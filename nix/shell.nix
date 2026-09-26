{ pkgs }:
pkgs.mkShell {
  packages = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
    (import ./d2.nix d2)
    mdbook
    just
    lefthook
    nixfmt
  ];
  shellHook = ''
    if [ -t 1 ]; then
      echo
      just --list --unsorted
    fi
  '';
}
