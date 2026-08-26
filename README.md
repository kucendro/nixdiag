<img src="assets/logo.svg" align="left" width="180">

<samp>Static infrastructure docs from any Nix flake. nixdiag reads your
nixosConfigurations and darwinConfigurations and renders data-flow topology
diagrams, module trees and an mdBook wiki.</samp>

<br clear="left">

```sh
nix run github:kucendro/nixdiag -- gen --flake .
```

<sub><samp>Unlike nix-topology (module-based, network-layer), nixdiag is
zero-touch and draws the data flow.</samp></sub>
