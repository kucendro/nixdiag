{
  description = "Static infrastructure docs from any Nix flake";

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

        # Hoisted out of fixture-docs-closures so `just wiki` can render the
        # fixture with a working-tree binary. tests/fixture/flake.nix is a
        # plain attrset that is never evaluated as a flake, so mode A cannot
        # produce these itself.
        fixture-facts = pkgs.writeText "facts.json" (
          builtins.toJSON (self.lib.mkFacts { flake = fixtureFlake; })
        );

        # The fixture rendered with hand-written closure data. Real closures
        # would mean building two NixOS systems in CI; this exercises the
        # model, the fleet analysis and the page at zero build cost.
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

        # The fixture's wiki, published under /demo on the docs site.
        demo-docs = self.lib.mkDocs {
          inherit pkgs;
          flake = fixtureFlake;
          title = "Example fleet";
          domains.ts = "ts.example";
        };

        # Hand-written book in site/, with the demo wiki nested under /demo
        # and the README diagrams doubling as the book's images.
        site =
          pkgs.runCommand "nixdiag-site"
            {
              nativeBuildInputs = [ pkgs.mdbook ];
            }
            ''
              cp -r ${./site} book
              chmod -R u+w book
              cp ${./assets}/topology.svg ${./assets}/modules.svg book/src/
              # The closure chart on build.md is the reference snapshot itself,
              # so the page cannot drift from what the renderer emits.
              cp ${./tests/reference}/closures.svg book/src/
              # --dest-dir resolves relative paths against the cwd, not the book root
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
          # Show the task list on entry rather than starting anything: a
          # server spawned by a shellHook leaks past the shell, fights over
          # ports with the last one, and fires in CI and editor shells too.
          # `just site` / `just wiki` are one word away.
          shellHook = ''
            if [ -t 1 ]; then
              echo
              just --list --unsorted
            fi
          '';
        };
      });

      checks = eachSystem (pkgs: {
        build = self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag;
        site = self.packages.${pkgs.stdenv.hostPlatform.system}.site;
        # The real exportReferencesGraph plumbing, over a package small enough
        # to substitute in CI — proves __structuredAttrs + jq without building
        # a NixOS system.
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

        # nixdiag measuring nixdiag. Cheap (the package is realised by
        # `checks.build` anyway) and it guards the one thing a closure report
        # is for: knowing when the closure quietly grew.
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

            # d2's withImageSupport pulls playwright-driver.browsers for a PNG
            # exporter nixdiag never calls -- 2.2 GiB of Chromium, Firefox and
            # WebKit. nix/d2.nix turns it off; this is the tripwire.
            if jq -e '[.hosts.nixdiag.paths[].path] | any(test("playwright"))' ${out} > /dev/null; then
              echo "playwright is back in nixdiag's runtime closure -- see nix/d2.nix"
              exit 1
            fi

            # Measured 228 MiB on x86_64-linux. The ceiling is deliberately
            # loose: it is not a budget to optimise against, it is a tripwire
            # for something heavy joining the wrapper. Raise it deliberately,
            # the same way reference snapshots move.
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
              diff -u "$reference/closures.md" "$docs/wiki/src/closures.md"
              # nixdiag draws the fleet bar chart itself, so unlike anything d2
              # produces it is byte-deterministic and survives --no-svg — which
              # is what lets a picture be diffed here at all.
              diff -u "$reference/closures.svg" "$docs/wiki/src/closures.svg"
              grep -q '| Closure |' "$docs/wiki/src/hosts.md"
              # Nix records a reference for every store path appearing in an
              # output, so printing one would make the docs retain the whole
              # closure it describes. Keep the pages free of them.
              if grep -rIqE '/nix/store/[a-z0-9]{32}-' "$docs"; then
                echo "generated docs contain a store path; that would retain Nix references:"
                grep -rIoE '/nix/store/[a-z0-9]{32}-[^ `"]*' "$docs" | head
                exit 1
              fi
              touch $out
            '';

        reference =
          pkgs.runCommand "nixdiag-reference"
            {
              docs = self.packages.${pkgs.stdenv.hostPlatform.system}.fixture-docs;
              reference = ./tests/reference;
            }
            ''
              diff -u "$reference/topology.d2" "$docs/topology.d2"
              diff -u "$reference/modules.d2" "$docs/modules.d2"
              diff -u "$reference/inputs.d2" "$docs/inputs.d2"
              diff -u "$reference/hosts.md" "$docs/wiki/src/hosts.md"
              diff -u "$reference/endpoints.md" "$docs/wiki/src/endpoints.md"
              diff -u "$reference/inputs.md" "$docs/wiki/src/inputs.md"
              touch $out
            '';
      });
    };
}
