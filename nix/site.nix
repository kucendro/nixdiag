{
  pkgs,
  self,
  fixtureFlake,
}:
rec {
  demo-docs = self.lib.mkDocs {
    inherit pkgs;
    flake = fixtureFlake;
    title = "Example fleet";
    theme = "light";
  };

  site =
    pkgs.runCommand "nixdiag-site"
      {
        nativeBuildInputs = [ pkgs.mdbook ];
      }
      ''
        cp -r ${../site} book
        chmod -R u+w book
        cp ${../assets}/topology-light.svg book/src/topology.svg
        cp ${../assets}/modules-light.svg book/src/modules.svg
        cp ${../assets}/closures-light.svg book/src/closures.svg
        mdbook build book --dest-dir $out
        cp -r --no-preserve=mode ${demo-docs}/wiki/book $out/demo
      '';
}
