# Build and serve

```nix
inputs.nixdiag.url = "github:kucendro/nixdiag";

packages.x86_64-linux.docs = nixdiag.lib.mkDocs {
  pkgs = nixpkgs.legacyPackages.x86_64-linux;
  flake = self;
  title = "my infrastructure wiki";
};
```

`nix build .#docs`: your flake evaluates its configurations, the sandbox
renders, nothing is committed back. `nix run github:kucendro/nixdiag` does
the same for the current directory without touching the flake.

| Argument | Default | Effect |
|---|---|---|
| `pkgs` | required | nixpkgs instance to build with |
| `flake` | required | flake to document, usually `self` |
| `title` | `"Infrastructure wiki"` | book title |
| `hosts` | all | list of host names to restrict to |
| `indexPage` | seeded stub | your own `wiki/src/index.md` |
| `bookToml` | seeded default | your own `wiki/book.toml` |
| `extraPages` | `{ }` | `{ Runbooks = ./runbooks.md; }`, copied in and linked |
| `extraLinks` | `{ }` | `{ Termux = "termux.md"; }`, SUMMARY entry for a page another tool writes |
| `extraAssets` | `{ }` | `{ "img/rack.png" = ./rack.png; }`, copied into `wiki/src` before mdbook |
| `buildWiki` | `true` | set `false` for diagrams and markdown only, no mdbook |
| `theme` | `"dark"` | `"light"` or `"dark"` |
| `background` | `"transparent"` | diagram canvas fill, any d2 fill |
| `colors` | `{ }` | palette overrides, see below |
| `closures` | `false` | per-host closure sizes: `true`, or a list of hosts; **builds those systems** |
| `closuresExclude` | `[ ]` | hosts to leave out of `closures = true` |

### Closure metrics

> **`closures` builds every host it measures.** Nar sizes exist only for
> realised paths, so each measured `system.build.toplevel` becomes a build
> input. Mostly substituted, but budget for it. NixOS only.

| value | measured |
|---|---|
| `false` | nothing |
| `true` | every NixOS host, minus `closuresExclude` |
| `[ "nas" "luna" ]` | exactly those |

A host that serves the docs is measured with an empty directory in the docs'
place, so a build may document the host that serves it. The Closures page
says so under that host.

![Fleet closure sizes](./closures.svg)

Each bar is one host's closure split into what every host carries, what some
carry, and what this host alone costs; each measured host also gets a treemap
of its packages. An unmeasured host keeps its row with `—`. Packages are
named without their store path, so the docs never retain the closures they
describe.

Chart colors: `chartShared`, `chartPartial`, `chartUnique`, `chartMark`,
`chartInk`, `chartMuted`, `chartTrack`, `chartTileInk`.

Build and root the systems in your own pipeline first, on the machine that
runs the docs build; the measurement then finds everything realised and costs
seconds of `jq`:

```yaml
- run: |
    for host in nas edge nixbook; do
      nix build ".#nixosConfigurations.$host.config.system.build.toplevel" \
        --out-link "/var/lib/ci/gcroots/$host"
    done
- run: nix build .#docs
```

The docs hold no references to the systems, so anything unrooted goes at the
next `nix-collect-garbage`. The measurement is `preferLocalBuild`, so
invoking it elsewhere copies every closure to the invoking machine.

## Diagram styling

```nix
theme = "light";
background = "#ffffff";
colors = { public = "#ff5555"; mesh = "#7fa7e8"; };
```

Names: `appFill`, `appStroke`, `infraFill`, `infraStroke`, `baseFill`,
`baseStroke`, `hostFill`, `hostStroke`, `progFill`, `hostCloud`, `public`,
`lan`, `mesh`. An unknown name is an error listing the palette. Colours live
in a `vars` block at the top of each `.d2`, editable by hand.

`theme` also picks the mdBook theme the seeded `book.toml` starts with:
`navy` for dark, `light` with `coal` for dark-preferring viewers otherwise.

## Serving it

```nix
{
  imports = [ inputs.nixdiag.nixosModules.default ];

  services.nixdiag.serve = {
    enable = true;
    docs = inputs.self.packages.x86_64-linux.docs;
    virtualHost = "wiki.example.com";
    virtualHostExtra = {
      listenAddresses = [ "100.64.0.1" ];
      useACMEHost = "example.com";
      forceSSL = true;
    };
  };
}
```

The vhost points straight at the docs derivation, so the wiki ships with every
deploy. No daemon, no timer, no checkout.

| Option | Default | Effect |
|---|---|---|
| `serve.enable` | `false` | create the vhost |
| `serve.docs` | required | docs derivation, typically `lib.mkDocs { … }` |
| `serve.virtualHost` | required | nginx vhost name |
| `serve.subpath` | `"wiki/book"` | path inside the derivation used as web root |
| `serve.virtualHostExtra` | `{ }` | merged into the vhost: TLS, listen addresses |

The module also declares the `nixdiag.*` options; `mkFacts` adds it itself
when a host does not import it. Keep one `nixdiag` input for both, a module
from another revision fails with a schema mismatch.

## Printing

mdBook's print icon lays the book out on one page, and the browser's "Save as
PDF" turns that into a vector PDF, diagrams included. Choose landscape or a
larger paper size where a dense diagram has to stay legible.
