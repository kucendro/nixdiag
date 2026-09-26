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
      packagesOf = pkgs: self.packages.${pkgs.stdenv.hostPlatform.system};
      fixture = import ./nix/fixture.nix { inherit self nixpkgs; };
    in
    {
      lib = import ./nix/lib.nix {
        inherit self;
        lib = nixpkgs.lib;
      };

      packages = eachSystem (
        pkgs:
        let
          nixdiag = pkgs.callPackage ./nix/package.nix { };
        in
        {
          default = nixdiag;
          inherit nixdiag;
        }
        // fixture.packages { inherit pkgs nixdiag; }
        // import ./nix/site.nix {
          inherit pkgs self;
          fixtureFlake = fixture.flake;
        }
      );

      overlays.default = final: prev: { nixdiag = (packagesOf final).nixdiag; };

      nixosModules.default = import ./nix/module.nix { inherit self; };

      templates.default = {
        path = ./templates/default;
        description = "Flake with nixdiag docs as a pure derivation";
      };

      apps = eachSystem (pkgs: {
        default = {
          type = "app";
          program = nixpkgs.lib.getExe (packagesOf pkgs).nixdiag;
        };
      });

      devShells = eachSystem (pkgs: {
        default = import ./nix/shell.nix { inherit pkgs; };
      });

      checks = eachSystem (
        pkgs:
        import ./nix/checks.nix {
          inherit pkgs;
          packages = packagesOf pkgs;
        }
      );
    };
}
