# Live demo

[Open the demo wiki](./demo/index.html)

That is the two-host test fixture in `tests/fixture/`, rendered by
`nix build .#demo-docs` and published next to this book. Around sixty lines of
NixOS config with a dozen `#:` comments produce every page you see there:
architecture with both diagrams, hosts with their ports and services, the
service index, and the endpoints table.

The same render feeds `checks.reference`, which diffs it against committed
snapshots, and supplies the diagrams on this site. Fixture, tests, images and
demo cannot drift apart.
