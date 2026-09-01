{
  lib,
  rustPlatform,
  makeWrapper,
  d2,
  nix,
}:

let
  d2-svg = import ./d2.nix d2;
in
rustPlatform.buildRustPackage {
  pname = "nixdiag";
  version = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;
  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];
  postInstall = ''
    wrapProgram $out/bin/nixdiag \
      --suffix PATH : ${
        lib.makeBinPath [
          d2-svg
          nix
        ]
      }
  '';

  meta = {
    description = "Static infrastructure docs from any Nix flake";
    homepage = "https://github.com/kucendro/nixdiag";
    license = lib.licenses.mit;
    mainProgram = "nixdiag";
  };
}
