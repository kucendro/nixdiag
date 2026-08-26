# Mode B: pure-derivation docs. The consumer flake evals its own
# configurations at eval time; the sandbox only renders.
{ self, lib }:
rec {
  # flake -> facts attrset (same shape as `nixdiag facts`).
  mkFacts =
    {
      flake,
      hosts ? null,
    }:
    let
      pick = names: if hosts == null then names else builtins.filter (n: builtins.elem n hosts) names;
      project =
        kind: cfgs:
        builtins.listToAttrs (
          map (n: {
            name = n;
            value = import ./projections/core.nix {
              host = cfgs.${n};
              inherit kind;
            };
          }) (pick (builtins.attrNames cfgs))
        );
    in
    {
      schema = 2;
      hosts =
        project "nixos" (flake.nixosConfigurations or { })
        // project "darwin" (flake.darwinConfigurations or { });
    };

  # flake -> docs derivation. `nix build .#docs` stays pure and cached;
  # this replaces committing generated docs back from CI.
  mkDocs =
    {
      pkgs,
      flake,
      title ? "Infrastructure wiki",
      # User-owned index page; replaces the seeded wiki/src/index.md.
      indexPage ? null,
      # User-owned mdBook config; replaces the seeded wiki/book.toml.
      bookToml ? null,
      extraPages ? { },
      extraLinks ? { },
      # Static files/dirs copied into wiki/src (path under src -> source),
      # e.g. images referenced by extra pages. Copied before mdbook runs.
      extraAssets ? { },
      hosts ? null,
      buildWiki ? true,
    }:
    let
      facts = mkFacts { inherit flake hosts; };
      factsJson = pkgs.writeText "nixdiag-facts.json" (builtins.toJSON facts);
      nixdiag = self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag;
      pageFlags = lib.mapAttrsToList (t: p: "--extra-page ${lib.escapeShellArg "${t}=${p}"}") extraPages;
      linkFlags = lib.mapAttrsToList (t: n: "--extra-link ${lib.escapeShellArg "${t}=${n}"}") extraLinks;
    in
    pkgs.runCommand "nixdiag-docs"
      {
        nativeBuildInputs = [
          nixdiag
          pkgs.d2
        ]
        ++ lib.optional buildWiki pkgs.mdbook;
      }
      ''
        nixdiag render --facts ${factsJson} --repo ${flake} --out $out \
          --title ${lib.escapeShellArg title} \
          ${lib.concatStringsSep " " (pageFlags ++ linkFlags)}
        ${lib.optionalString (indexPage != null) ''
          install -m 644 ${indexPage} $out/wiki/src/index.md
        ''}
        ${lib.optionalString (bookToml != null) ''
          install -m 644 ${bookToml} $out/wiki/book.toml
        ''}
        ${lib.concatStringsSep "\n" (
          lib.mapAttrsToList (dest: src: ''
            mkdir -p "$(dirname $out/wiki/src/${dest})"
            cp -r --no-preserve=mode ${src} "$out/wiki/src/${dest}"
          '') extraAssets
        )}
        ${lib.optionalString buildWiki ''
          # --dest-dir resolves relative paths against the cwd, not the book root
          mdbook build $out/wiki --dest-dir $out/wiki/book
        ''}
      '';
}
