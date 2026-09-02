{ self, lib }:
rec {
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

  mkDocs =
    {
      pkgs,
      flake,
      title ? "Infrastructure wiki",
      indexPage ? null,
      bookToml ? null,
      extraPages ? { },
      extraLinks ? { },
      extraAssets ? { },
      hosts ? null,
      buildWiki ? true,
      theme ? null,
      background ? null,
      colors ? { },
      domains ? { },
      grammar ? null,
      deny ? [ ],
      closures ? false,
      closuresExclude ? [ ],
      api ? true,
      # Identity for api/v1/snapshot.json, so accumulated snapshots can be
      # told apart. `self.rev` on a clean tree, `self.dirtyRev` otherwise,
      # neither on a plain directory — guarded with `or` so eval never throws.
      # nixdiag itself never invokes git; the value is always passed in.
      revision ? (flake.rev or flake.dirtyRev or null),
      # A fixed integer Nix computes from the source, not a clock read.
      revisionTime ? (flake.lastModified or null),
    }:
    let
      facts = mkFacts { inherit flake hosts; };

      nixosConfigs = lib.filterAttrs (n: _: hosts == null || builtins.elem n hosts) (
        flake.nixosConfigurations or { }
      );

      serving = builtins.attrNames (
        lib.filterAttrs (_: cfg: cfg.config.services.nixdiag.serve.enable or false) nixosConfigs
      );

      excluded =
        let
          known = builtins.attrNames (flake.nixosConfigurations or { });
          unknown = builtins.filter (n: !(builtins.elem n known)) closuresExclude;
        in
        if unknown != [ ] then
          throw "nixdiag: closuresExclude names unknown host(s): ${lib.concatStringsSep ", " unknown}"
        else
          closuresExclude;

      closureHosts =
        if lib.isList closures then
          if closuresExclude != [ ] then
            throw ''
              nixdiag: closuresExclude has nothing to subtract from an explicit closures list.
              The list already names exactly what to measure; drop one of the two.
            ''
          else
            let
              unknown = builtins.filter (n: !(nixosConfigs ? ${n})) closures;
            in
            if unknown != [ ] then
              throw "nixdiag: closures names unknown host(s): ${lib.concatStringsSep ", " unknown}"
            else
              closures
        else if closures then
          let
            wanted = builtins.filter (n: !(builtins.elem n excluded)) (builtins.attrNames nixosConfigs);
            cyclic = builtins.filter (n: builtins.elem n serving) wanted;
            kept = builtins.filter (n: !(builtins.elem n cyclic)) wanted;
          in
          if cyclic == [ ] then
            kept
          else
            lib.warn ''
              nixdiag: skipping closure metrics for ${lib.concatStringsSep ", " cyclic}.
              Those hosts run services.nixdiag.serve, so their system closure contains an
              nginx vhost rooted at a docs derivation; measuring them would make these docs
              depend on a system that contains docs, which Nix reports as infinite recursion.
              Say so and this warning stops, everything else stays automatic:
                closuresExclude = [ ${lib.concatMapStringsSep " " (n: ''"${n}"'') cyclic} ];
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
        ++ lib.optional (closureFile != null) "--closures ${closureFile}"
        # `cmd_render` uses FlakeConfig::default(), so mode B never sees the
        # documented flake's `nixdiag` attr — every setting must be passed.
        ++ lib.optional (!api) "--no-api"
        ++ lib.optional (api && revision != null) "--revision ${lib.escapeShellArg revision}"
        ++ lib.optional (api && revisionTime != null) "--revision-time ${toString revisionTime}";
    in
    pkgs.runCommand "nixdiag-docs"
      {
        nativeBuildInputs = [
          nixdiag
          (import ./d2.nix pkgs.d2)
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
