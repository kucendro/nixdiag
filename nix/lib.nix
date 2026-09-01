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
      # theme "dark" (default) or "light"; background any fill (default
      # "transparent"); colors overrides palette names from the d2 vars
      # block, e.g. { public = "#ff5555"; }.
      theme ? null,
      background ? null,
      colors ? { },
      # `@key` -> domain suffix for fqdn positions in annotations, e.g.
      # { home = "home.example.com"; } lets `name=vault@home` render as
      # vault.home.example.com without the literal appearing in the repo.
      domains ? { },
      # Annotation grammar edition the flake's modules are written against.
      # null means "whatever the nixdiag binary implements"; a declaration
      # newer than the binary implements is a hard error.
      grammar ? null,
      # Warning categories promoted to errors, e.g. [ "deprecated" ].
      deny ? [ ],
      # Per-host system closure sizes, adding a Closures wiki page and a row
      # on each host. Requires every measured host's system to be BUILT (or
      # substituted), so this makes `nix build .#docs` as expensive as
      # building those systems. NixOS hosts only: darwin cannot be built
      # from Linux.
      #
      #   false          off (default)
      #   true           every NixOS host that does not serve nixdiag docs
      #   [ "nas" … ]    exactly these hosts, taken at your word
      #
      # This MEASURES; it is not a way to build a fleet. Nothing here keeps
      # what it realises alive: the pages carry no store paths (printing one
      # would make the docs retain the closure they describe), so the docs
      # hold no references to those systems and the next garbage collection
      # removes anything your own pipeline did not root. Build and root the
      # systems first -- a deploy step, a CI job, `nix build
      # .#nixosConfigurations.<h>.config.system.build.toplevel --out-link ...`
      # -- and enable this on a docs build that runs after. Everything is then
      # already realised and the measurement costs a few seconds of jq.
      #
      # Run that docs build on the machine that holds the systems: the
      # per-host derivation is preferLocalBuild, so invoking it elsewhere
      # copies every measured closure to the invoking machine.
      #
      # A host running services.nixdiag.serve is skipped under `true` because
      # its system closure contains an nginx vhost rooted at a docs
      # derivation. Measuring it would make these docs depend on a system
      # that contains docs, and Nix reports that as infinite recursion rather
      # than anything legible. See `serving` below.
      closures ? false,
    }:
    let
      facts = mkFacts { inherit flake hosts; };

      nixosConfigs = lib.filterAttrs (n: _: hosts == null || builtins.elem n hosts) (
        flake.nixosConfigurations or { }
      );

      # Reading `serve.enable` is safe; reading `serve.docs` is not. The
      # latter is the vhost root, and forcing it from within a docs build is
      # precisely the cycle. Verified: `serve.enable` evaluates without
      # forcing `serve.docs`, while `system.build.toplevel` does force it.
      serving = builtins.attrNames (
        lib.filterAttrs (_: cfg: cfg.config.services.nixdiag.serve.enable or false) nixosConfigs
      );

      closureHosts =
        if lib.isList closures then
          let
            unknown = builtins.filter (n: !(nixosConfigs ? ${n})) closures;
          in
          if unknown != [ ] then
            throw "nixdiag: closures names unknown host(s): ${lib.concatStringsSep ", " unknown}"
          else
            closures
        else if closures then
          let
            kept = builtins.filter (n: !(builtins.elem n serving)) (builtins.attrNames nixosConfigs);
          in
          if serving == [ ] then
            kept
          else
            lib.warn ''
              nixdiag: skipping closure metrics for ${lib.concatStringsSep ", " serving}.
              Those hosts run services.nixdiag.serve, so their system closure contains an
              nginx vhost rooted at a docs derivation; measuring them would make these docs
              depend on a system that contains docs, which Nix reports as infinite recursion.
              If this build is not the one being served, ask for the hosts by name:
                closures = [ ${lib.concatMapStringsSep " " (n: ''"${n}"'') (builtins.attrNames nixosConfigs)} ];
            '' kept
        else
          [ ];

      closureFile =
        if closureHosts == [ ] then
          null
        else
          (import ./closures.nix { inherit pkgs lib; }).mkClosures (
            lib.mapAttrs (_: cfg: cfg.config.system.build.toplevel) (lib.getAttrs closureHosts nixosConfigs)
          );
      factsJson = pkgs.writeText "nixdiag-facts.json" (builtins.toJSON facts);
      nixdiag = self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag;
      pageFlags = lib.mapAttrsToList (t: p: "--extra-page ${lib.escapeShellArg "${t}=${p}"}") extraPages;
      linkFlags = lib.mapAttrsToList (t: n: "--extra-link ${lib.escapeShellArg "${t}=${n}"}") extraLinks;
      styleFlags =
        lib.optional (theme != null) "--theme ${theme}"
        ++ lib.optional (background != null) "--background ${lib.escapeShellArg background}"
        ++ lib.mapAttrsToList (n: v: "--color ${lib.escapeShellArg "${n}=${v}"}") colors
        ++ lib.mapAttrsToList (k: v: "--domain ${lib.escapeShellArg "${k}=${v}"}") domains
        ++ lib.optional (grammar != null) "--grammar ${toString grammar}"
        ++ map (d: "--deny ${lib.escapeShellArg d}") deny
        ++ lib.optional (closureFile != null) "--closures ${closureFile}";
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
          ${lib.concatStringsSep " " (pageFlags ++ linkFlags ++ styleFlags)}
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
