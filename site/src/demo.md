# Live demo

[Open the demo wiki](./demo/index.html)

The two-host fixture in `tests/fixture/`, rendered by `nix build .#demo-docs`:
around sixty lines of NixOS config with a dozen `#:` comments produce every
page there.

The same render feeds `checks.reference` and supplies the diagrams on this
site, so fixture, tests, images and demo cannot drift apart.
