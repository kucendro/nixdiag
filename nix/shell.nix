{ pkgs }:
pkgs.mkShell {
  packages = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
    (callPackage ./d2.nix { })
    graphviz
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
