{
  lib,
  rustPlatform,
  makeWrapper,
  callPackage,
}:

rustPlatform.buildRustPackage {
  pname = "nixdiag";
  version = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;
  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
    ];
  };
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ makeWrapper ];
  postInstall = ''
    wrapProgram $out/bin/nixdiag --suffix PATH : ${lib.makeBinPath [ (callPackage ./d2.nix { }) ]}
  '';

  meta = {
    description = "Static infrastructure docs";
    homepage = "https://github.com/kucendro/nixdiag";
    license = lib.licenses.mit;
    mainProgram = "nixdiag";
  };
}
