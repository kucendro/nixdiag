# Quickstart

## Render an existing flake

```sh
cd ~/my-nix-config
nix run github:kucendro/nixdiag -- gen --flake .
```

That writes `docs/` next to your flake: two diagrams and an mdBook source
tree. Nothing is required in the flake itself, and with zero annotations you
still get host boxes with their open ports, the module tree and the whole
wiki.

Build the book with mdbook, or let [mkDocs](./build.md) do it in a derivation:

```sh
nix run nixpkgs#mdbook -- serve docs/wiki
```

## Start a new flake

```sh
nix flake init -t github:kucendro/nixdiag
```

The template flake has a `nixdiag` output for CLI defaults and a
`packages.x86_64-linux.docs` built with `nixdiag.lib.mkDocs`.

## Add the first annotation

Edges and endpoints come from comments in your own module files:

```nix
{
  #: mesh-control
  #: name hs.example.com
  #: expose 443 public name=hs.example.com
  services.headscale = {
    enable = true;
    port = 8080;
  };
}
```

Re-run `nixdiag gen` and headscale is a node in the topology, drawn with the
`mesh-control` role, linked to the internet cloud, with a row on the Endpoints
page. Point something at it:

```nix
{
  #: proxy
  services.nginx = {
    enable = true;
    #: -> headscale hs :8080
    virtualHosts."hs.example.com".locations."/".proxyPass = "http://127.0.0.1:8080";
  };
}
```

Full grammar: [Annotations](./annotations.md).

## Keep it honest in CI

```sh
nixdiag check --flake .
```

`check` re-renders to a temp dir and diffs against the committed output, so a
config change that outdates the docs fails the build. If you would rather not
commit generated files at all, build the docs as a derivation instead and skip
`check` entirely: see [Build and serve](./build.md).
