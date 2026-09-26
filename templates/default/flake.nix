{
  description = "Flake documented by nixdiag";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    nixdiag.url = "github:kucendro/nixdiag";
  };

  outputs =
    {
      self,
      nixpkgs,
      nixdiag,
    }:
    {
      packages.x86_64-linux.docs = nixdiag.lib.mkDocs {
        pkgs = nixpkgs.legacyPackages.x86_64-linux;
        flake = self;
      };
    };
}
