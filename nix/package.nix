{
  lib,
  rustPlatform,
  makeWrapper,
  d2,
}:

rustPlatform.buildRustPackage {
  pname = "nixdiag";
  version = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;
  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];
  postInstall = ''
    wrapProgram $out/bin/nixdiag --suffix PATH : ${lib.makeBinPath [ (import ./d2.nix d2) ]}
  '';

  meta = {
    description = "Static infrastructure docs";
    homepage = "https://github.com/kucendro/nixdiag";
    license = lib.licenses.mit;
    mainProgram = "nixdiag";
  };
}
