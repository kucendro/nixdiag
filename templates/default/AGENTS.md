# Notes for coding agents

This flake is documented by nixdiag. The topology diagram and the Endpoints
page come from `#:` comment lines in the Nix modules, never from option
values. Before writing or changing a `#:` line, read the grammar:

    nix run github:kucendro/nixdiag -- syntax

or <https://kucendro.github.io/nixdiag/syntax.html>. Then build the docs with
`nix build .#docs`; a malformed line or an edge to something that does not
exist fails the build on purpose.
