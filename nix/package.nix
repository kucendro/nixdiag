{
  lib,
  rustPlatform,
  makeWrapper,
  d2,
  nix,
}:

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
          d2
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
