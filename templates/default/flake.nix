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
      nixdiag = {
        out = "docs";
        # title = "my infrastructure wiki";
        # extraPages.Runbooks = "./docs-src/runbooks.md";
        # extraLinks.Termux = "termux.md";
      };

      # nix build .#docs
      packages.x86_64-linux.docs = nixdiag.lib.mkDocs {
        pkgs = nixpkgs.legacyPackages.x86_64-linux;
        flake = self;
        # title = "my infrastructure wiki";
        # extraPages.Runbooks = ./docs-src/runbooks.md;
      };
    };
}
