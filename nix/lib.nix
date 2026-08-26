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
        proj: cfgs:
        builtins.listToAttrs (
          map (n: {
            name = n;
            value = import proj cfgs.${n};
          }) (pick (builtins.attrNames cfgs))
        );
    in
    {
      schema = 1;
      hosts =
        project ./projections/nixos.nix (flake.nixosConfigurations or { })
        // project ./projections/darwin.nix (flake.darwinConfigurations or { });
    };

  # flake -> docs derivation. `nix build .#docs` stays pure and cached;
  # this replaces committing generated docs back from CI.
  mkDocs =
    {
      pkgs,
      flake,
      title ? "Infrastructure wiki",
      extraPages ? { },
      extraLinks ? { },
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
        ${lib.optionalString buildWiki ''
          mdbook build $out/wiki --dest-dir book
        ''}
      '';
}
