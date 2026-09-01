<img src="assets/logo.svg" align="left" width="180">

<samp>Static infrastructure docs from any Nix flake. nixdiag reads your
nixosConfigurations and darwinConfigurations and renders data-flow topology
diagrams, module trees and an mdBook wiki.</samp>

<br clear="left">

---

![topology](assets/topology.svg)

![modules](assets/modules.svg)

Every flake input placed by the date it is locked at, straight out of
`flake.lock` — no eval, no clock, and blue for the inputs your own flake
declares:

![lock dates](assets/inputs-timeline.svg)

Opt in to closure metrics and each host is measured against the rest of the
fleet, then broken down by package:

![fleet closure sizes](assets/closures.svg)

![closure treemap](assets/closures-sol.svg)

nixdiag draws those three itself rather than through d2, so they need no
binary on PATH and come out byte-identical run to run — which is what lets
`nixdiag check` hold a picture to a diff.

---

## Docs

**<https://kucendro.github.io/nixdiag>**

- [Quickstart](https://kucendro.github.io/nixdiag/quickstart.html): first render in two commands
- [Annotations](https://kucendro.github.io/nixdiag/annotations.html): the full `#:` grammar
- [Build and serve](https://kucendro.github.io/nixdiag/build.html): `lib.mkDocs` as a pure derivation, the nginx module
- [CLI](https://kucendro.github.io/nixdiag/cli.html): `facts`, `render`, `gen`, `check`
- [Live demo](https://kucendro.github.io/nixdiag/demo/): the wiki this repo's test fixture renders
