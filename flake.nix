{
  description = "Static infrastructure docs";

  inputs.nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      eachSystem = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});

      fixtureSrc = builtins.path {
        path = ./tests/fixture;
        name = "source";
      };

      fixtureFlake = {
        outPath = fixtureSrc;
        nixosConfigurations = nixpkgs.lib.genAttrs [ "luna" "sol" ] (
          name:
          nixpkgs.lib.nixosSystem {
            modules = [
              "${fixtureSrc}/hosts/${name}"
              { nixpkgs.hostPlatform = "x86_64-linux"; }
            ];
          }
        );
      };
    in
    {
      lib = import ./nix/lib.nix {
        inherit self;
        lib = nixpkgs.lib;
      };

      packages = eachSystem (pkgs: rec {
        default = nixdiag;

        fixture-docs = self.lib.mkDocs {
          inherit pkgs;
          flake = fixtureFlake;
          buildWiki = false;
          domains.ts = "ts.example";
        };

        fixture-facts = pkgs.writeText "facts.json" (
          builtins.toJSON (self.lib.mkFacts { flake = fixtureFlake; })
        );

        fixture-docs-closures =
          pkgs.runCommand "nixdiag-fixture-closures"
            {
              nativeBuildInputs = [ nixdiag ];
              facts = fixture-facts;
            }
            ''
              nixdiag render --facts "$facts" --repo ${fixtureSrc} \
                --closures ${fixtureSrc}/closures.json \
                --domain ts=ts.example --out $out --no-svg
            '';

        demo-docs = self.lib.mkDocs {
          inherit pkgs;
          flake = fixtureFlake;
          title = "Example fleet";
          domains.ts = "ts.example";
        };

        site =
          pkgs.runCommand "nixdiag-site"
            {
              nativeBuildInputs = [ pkgs.mdbook ];
            }
            ''
              cp -r ${./site} book
              chmod -R u+w book
              cp ${./assets}/topology.svg ${./assets}/modules.svg book/src/
              cp ${./tests/reference}/wiki/src/closures.svg book/src/
              mdbook build book --dest-dir $out
              cp -r --no-preserve=mode ${demo-docs}/wiki/book $out/demo
            '';

        nixdiag = pkgs.callPackage ./nix/package.nix { };
      });

      overlays.default = final: prev: {
        nixdiag = self.packages.${final.stdenv.hostPlatform.system}.nixdiag;
      };

      nixosModules.default = import ./nix/module.nix { inherit self; };

      templates.default = {
        path = ./templates/default;
        description = "Flake with nixdiag docs as a pure derivation";
      };

      apps = eachSystem (pkgs: {
        default = {
          type = "app";
          program = nixpkgs.lib.getExe self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag;
        };
      });

      devShells = eachSystem (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            (import ./nix/d2.nix d2)
            mdbook
            just
            lefthook
            nixfmt
          ];
          shellHook = ''
            if [ -t 1 ]; then
              echo
              just --list --unsorted
            fi
          '';
        };
      });

      checks = eachSystem (
        pkgs:
        let
          # Snapshot list and layout live in tests/reference/MANIFEST, which
          # `just snapshots` writes from. One source, so adding a snapshot
          # cannot land in the refresher but not the gate.
          diffManifest = pkgs.writeShellScript "nixdiag-diff-manifest" ''
            set -euo pipefail
            want="$1"; reference="$2"; docs="$3"; seen=0
            while read -r build path; do
              case "$build" in "") continue ;; esac
              # Every line is validated by both checks, so a typo in the build
              # column fails loudly instead of silently skipping a snapshot.
              case "$build" in
                docs|closures) ;;
                *) echo "MANIFEST: unknown build '$build' for $path"; exit 1 ;;
              esac
              [ "$build" = "$want" ] || continue
              if [ ! -e "$docs/$path" ]; then
                echo "MANIFEST lists $path, but the build did not write it"; exit 1
              fi
              diff -u "$reference/$path" "$docs/$path"
              seen=$((seen + 1))
            done < <(sed 's/#.*//' "$reference/MANIFEST")
            [ "$seen" -gt 0 ] || { echo "no MANIFEST entries for '$want'"; exit 1; }
            echo "$want: $seen snapshots match"
          '';

          # Nix records a reference for every store path in an output, so a
          # listing of a system closure would make the docs retain it.
          noStorePaths = ''
            if grep -rIqE '/nix/store/[a-z0-9]{32}-' "$docs"; then
              echo "generated docs contain a store path; that would retain Nix references:"
              grep -rIoE '/nix/store/[a-z0-9]{32}-[^ `"]*' "$docs" | head
              exit 1
            fi
          '';
        in
        {
          build = self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag;
          site = self.packages.${pkgs.stdenv.hostPlatform.system}.site;
          closures-plumbing =
            let
              out =
                (import ./nix/closures.nix {
                  inherit pkgs;
                  lib = pkgs.lib;
                }).mkClosures
                  { demo = pkgs.hello; };
            in
            pkgs.runCommand "nixdiag-closures-plumbing" { nativeBuildInputs = [ pkgs.jq ]; } ''
              jq -e '.schema == 1' ${out} > /dev/null
              jq -e '.hosts.demo.paths | length > 0' ${out} > /dev/null
              jq -e '.hosts.demo.paths | all(has("path") and has("narSize"))' ${out} > /dev/null
              jq -e '.hosts.demo.paths == (.hosts.demo.paths | sort_by(.path))' ${out} > /dev/null
              touch $out
            '';

          closures-self =
            let
              out =
                (import ./nix/closures.nix {
                  inherit pkgs;
                  lib = pkgs.lib;
                }).mkClosures
                  { nixdiag = self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag; };
            in
            pkgs.runCommand "nixdiag-closures-self" { nativeBuildInputs = [ pkgs.jq ]; } ''
              paths=$(jq '.hosts.nixdiag.paths | length' ${out})
              bytes=$(jq '[.hosts.nixdiag.paths[].narSize] | add' ${out})
              echo "nixdiag runtime closure: $((bytes / 1048576)) MiB across $paths paths"

              if jq -e '[.hosts.nixdiag.paths[].path] | any(test("playwright"))' ${out} > /dev/null; then
                echo "playwright is back in nixdiag's runtime closure -- see nix/d2.nix"
                exit 1
              fi

              if [ "$bytes" -gt $((600 * 1024 * 1024)) ]; then
                echo "runtime closure passed 600 MiB; re-measure and raise the ceiling on purpose"
                exit 1
              fi
              touch $out
            '';

          closures =
            pkgs.runCommand "nixdiag-closures-reference"
              {
                docs = self.packages.${pkgs.stdenv.hostPlatform.system}.fixture-docs-closures;
                reference = ./tests/reference;
              }
              ''
                ${diffManifest} closures "$reference" "$docs"
                grep -q '| Closure |' "$docs/wiki/src/hosts.md"
                ${noStorePaths}
                touch $out
              '';

          reference =
            pkgs.runCommand "nixdiag-reference"
              {
                docs = self.packages.${pkgs.stdenv.hostPlatform.system}.fixture-docs;
                reference = ./tests/reference;
                nativeBuildInputs = [ pkgs.jq ];
              }
              ''
                ${diffManifest} docs "$reference" "$docs"

                # snapshot.json is deliberately not diffed: it carries the
                # revision, so it is Volatile and out of the drift gate. Its
                # shape is still asserted.
                jq -e '.meta.schema and .totals.hosts' "$docs/api/v1/snapshot.json" > /dev/null

                # Every generated JSON must carry the AUTO marker, or the next
                # render refuses to overwrite its own output.
                for f in "$docs"/api/v1/*.json; do
                  grep -q 'Auto-generated' "$f" || { echo "no marker: $f"; exit 1; }
                done

                # `hosts.json` and `services.json` carry defining files, which
                # arrive from eval as store paths — the one place a Repo
                # resolution failure would leak one into an installed file.
                ${noStorePaths}
                touch $out
              '';
        }
      );
    };
}
