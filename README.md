<img src="assets/logo.svg" align="left" width="180">

<samp>Static infrastructure docs from any Nix flake. nixdiag reads your
nixosConfigurations and darwinConfigurations and renders data-flow topology
diagrams, module trees and an mdBook wiki.</samp>

<br clear="left">

---

<picture><source media="(prefers-color-scheme: dark)" srcset="assets/topology.svg"><img alt="topology" src="assets/topology-light.svg"></picture>

<picture><source media="(prefers-color-scheme: dark)" srcset="assets/modules.svg"><img alt="modules" src="assets/modules-light.svg"></picture>

<picture><source media="(prefers-color-scheme: dark)" srcset="assets/inputs.svg"><img alt="flake inputs" src="assets/inputs-light.svg"></picture>

<picture><source media="(prefers-color-scheme: dark)" srcset="assets/inputs-timeline.svg"><img alt="input lock dates" src="assets/inputs-timeline-light.svg"></picture>

<picture><source media="(prefers-color-scheme: dark)" srcset="assets/closures.svg"><img alt="fleet closure sizes" src="assets/closures-light.svg"></picture>

<picture><source media="(prefers-color-scheme: dark)" srcset="assets/closures-sol.svg"><img alt="closure treemap" src="assets/closures-sol-light.svg"></picture>

---

## Docs

**<https://kucendro.github.io/nixdiag>**

- [Quickstart](https://kucendro.github.io/nixdiag/quickstart.html): first render in two commands
- [Annotations](https://kucendro.github.io/nixdiag/annotations.html): the full `#:` grammar
- [Cheat sheet](https://kucendro.github.io/nixdiag/syntax.html): the grammar on one page for your coding assistant, also `nixdiag syntax`
- [Build and serve](https://kucendro.github.io/nixdiag/build.html): `lib.mkDocs` as a pure derivation, the nginx module
- [CLI](https://kucendro.github.io/nixdiag/cli.html): `facts`, `render`, `gen`, `check`
- [Live demo](https://kucendro.github.io/nixdiag/demo/): the wiki this repo's test fixture renders

---

<a href="https://www.buymeacoffee.com/kucendro"><img src="https://img.buymeacoffee.com/button-api/?text=Buy%20me%20a%20matcha&emoji=&slug=kucendro&button_colour=5f5ca7&font_colour=ffffff&font_family=Cookie&outline_colour=ffffff&coffee_colour=FFDD00" alt="Buy me a matcha" height="50" /></a>
