<img src="assets/logo.svg" align="left" width="180">

<samp>Static infrastructure docs from any Nix flake. nixdiag reads your
nixosConfigurations and darwinConfigurations and renders data-flow topology
diagrams, module trees and an mdBook wiki.</samp>

<br clear="left">

---

![topology](assets/topology.svg)

![modules](assets/modules.svg)

---

## Docs

**<https://kucendro.github.io/nixdiag>**

- [Quickstart](https://kucendro.github.io/nixdiag/quickstart.html): first render in two commands
- [Annotations](https://kucendro.github.io/nixdiag/annotations.html): the full `#:` grammar
- [Build and serve](https://kucendro.github.io/nixdiag/build.html): `lib.mkDocs` as a pure derivation, the nginx module
- [CLI](https://kucendro.github.io/nixdiag/cli.html): `facts`, `render`, `gen`, `check`
- [Live demo](https://kucendro.github.io/nixdiag/demo/): the wiki this repo's test fixture renders
