{ self, lib }:
rec {
  mkFacts =
    {
      flake,
      hosts ? null,
    }:
    let
      pick = names: if hosts == null then names else builtins.filter (n: builtins.elem n hosts) names;
      factsOf =
        cfg:
        if cfg.options ? nixdiag then
          cfg.config.nixdiag.facts
        else
          (cfg.extendModules { modules = [ ./module/facts.nix ]; }).config.nixdiag.facts;
      project = cfgs: lib.genAttrs (pick (builtins.attrNames cfgs)) (n: factsOf cfgs.${n});
    in
    {
      schema = 3;
      hosts = project (flake.nixosConfigurations or { }) // project (flake.darwinConfigurations or { });
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
      closures ? false,
      closuresExclude ? [ ],
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
          builtins.filter (n: !(builtins.elem n excluded)) (builtins.attrNames nixosConfigs)
        else
          [ ];

      withoutDocs =
        cfg:
        cfg.extendModules {
          modules = [ { services.nixdiag.serve.docs = lib.mkForce pkgs.emptyDirectory; } ];
        };

      toplevelOf =
        name: cfg:
        (if builtins.elem name serving then withoutDocs cfg else cfg).config.system.build.toplevel;

      closureFile =
        if closureHosts == [ ] then
          null
        else
          (import ./closures.nix { inherit pkgs lib; }).mkClosures {
            toplevels = lib.mapAttrs toplevelOf (lib.getAttrs closureHosts nixosConfigs);
            served = builtins.filter (n: builtins.elem n serving) closureHosts;
          };
      factsJson = pkgs.writeText "nixdiag-facts.json" (builtins.toJSON facts);
      nixdiag = self.packages.${pkgs.stdenv.hostPlatform.system}.nixdiag;
      pageFlags = lib.mapAttrsToList (t: p: "--extra-page ${lib.escapeShellArg "${t}=${p}"}") extraPages;
      linkFlags = lib.mapAttrsToList (t: n: "--extra-link ${lib.escapeShellArg "${t}=${n}"}") extraLinks;
      styleFlags =
        lib.optional (theme != null) "--theme ${theme}"
        ++ lib.optional (background != null) "--background ${lib.escapeShellArg background}"
        ++ lib.mapAttrsToList (n: v: "--color ${lib.escapeShellArg "${n}=${v}"}") colors
        ++ lib.optional (closureFile != null) "--closures ${closureFile}";
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
          mdbook build $out/wiki --dest-dir $out/wiki/book
        ''}
      '';
}
