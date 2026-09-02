# Build and serve

Two modes. The CLI writes files into your repo, the derivation builds them in
the sandbox and commits nothing.

## Mode A: `nixdiag gen`

```sh
nixdiag gen --flake .
```

Defaults can live in the flake, under a `nixdiag` output, and flags override
them:

```nix
{
  nixdiag = {
    out = "docs";
    title = "my infrastructure wiki";
    extraPages.Runbooks = "./docs-src/runbooks.md";
    extraLinks.Termux = "termux.md";
    theme = "light";
    background = "#ffffff";
    colors.public = "#ff5555";
    domains.home = "home.example.com";
    grammar = 1;
    deny = [ "deprecated" ];
  };
}
```

Use this when you want the output in git, reviewed in diffs, with
`nixdiag check` as the CI gate.

## Mode B: `lib.mkDocs`

```nix
inputs.nixdiag.url = "github:kucendro/nixdiag";

packages.x86_64-linux.docs = nixdiag.lib.mkDocs {
  pkgs = nixpkgs.legacyPackages.x86_64-linux;
  flake = self;
  title = "my infrastructure wiki";
  domains.home = "home.example.com";
};
```

`nix build .#docs` and the whole thing, mdBook included, is a cached
derivation. Your flake evaluates its own configurations at eval time and the
sandbox only renders, so nothing is committed back and there is nothing to
drift from. Add `checks.docs = self.packages.x86_64-linux.docs;` and a broken
annotation fails the build.

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
| `domains` | `{ }` | `@key` suffixes for annotation fqdns |
| `grammar` | binary's own | annotation grammar edition your modules are written against, see [editions](./annotations.md#grammar-editions) |
| `deny` | `[ ]` | warning categories promoted to errors, e.g. `[ "deprecated" ]` |
| `closures` | `false` | per-host closure sizes: `true`, or a list of hosts; **requires those systems built** — see below |
| `api` | `true` | publish the JSON [data API](./api.md) and its OpenAPI document |
| `revision` | flake's own | revision recorded in `api/v1/snapshot.json`, for history |
| `revisionTime` | flake's own | unix time of that revision |
| `closuresExclude` | `[ ]` | hosts to leave out of `closures = true` |

### Closure metrics

> **`closures` builds every host it measures.** Nar sizes exist only for
> realised paths, so each measured host's `system.build.toplevel` becomes a
> build input and `nix build .#docs` costs as much as building those systems.
> Mostly substituted rather than compiled, but budget for it. NixOS only —
> darwin cannot be built from Linux.

| value | measured |
|---|---|
| `false` | nothing (default) |
| `true` | every NixOS host that does not serve nixdiag docs |
| `[ "nas" "luna" ]` | exactly those hosts, taken at your word |

`closuresExclude` subtracts from `true`, so the fleet is described by its
exceptions and a host added later is measured the day it lands:

```nix
closures = true;
closuresExclude = [ "vps" ];
```

It is an error beside an explicit `closures` list, which already names the
whole set.

You get a Closures page and a closure row on every host. Each bar is one host's
closure split into what every host carries, what some carry, and what this host
alone costs; each measured host also gets a treemap of its packages.

![Fleet closure sizes](./closures.svg)

A host that is not measured keeps its row, with `—` for its numbers. Packages
are named without their store path, so the docs never retain the closures they
describe.

Chart colors follow `theme` and are overridable by name: `chartShared`,
`chartPartial`, `chartUnique`, `chartMark`, `chartInk`, `chartMuted`,
`chartTrack`, `chartTileInk`.

#### Avoiding the build cost

Build the systems in your own pipeline, root them there, and run the docs build
afterwards — it then finds everything realised and the measurement is a few
seconds of `jq`:

```yaml
- name: BUILD
  run: |
    for host in nas edge nixbook; do
      nix build ".#nixosConfigurations.$host.config.system.build.toplevel" \
        --out-link "/var/lib/ci/gcroots/$host"
    done

- name: DOCS
  run: nix build .#docs        # closures = [ "nas" "nixbook" ]
```

`--out-link` is the GC root — the docs hold no references to the systems they
describe, so anything unrooted goes at the next `nix-collect-garbage`. A binary
cache on that machine can then serve the systems to the hosts that deploy them.

If your docs derivation is *also* a flake check, `nix flake check` realises the
systems itself; put the rooting step after it, so only a revision that passed
gets pinned.

Run the docs build **on the machine that holds the systems**. The measurement
is `preferLocalBuild`, so invoking it from your laptop against a remote builder
copies every measured closure to the laptop.

#### Why serving hosts are skipped

`services.nixdiag.serve` roots an nginx vhost at a docs derivation, so that
host's system closure *contains* the docs. Measuring it would give
`docs -> toplevel -> docs`, which Nix reports as infinite recursion.

`closures = true` skips those hosts and names them in a warning. Name one in
`closuresExclude` and the warning stops while every other host stays automatic.

Serve one build and measure another and there is no cycle, so you can ask for
every host by name:

```nix
packages.docs      = mkDocs { inherit pkgs; flake = self; };
packages.docs-full = mkDocs {
  inherit pkgs;
  flake = self;
  closures = [ "edge" "nas" ];   # edge serves `docs`, not `docs-full`
};
services.nixdiag.serve.docs = self.packages.x86_64-linux.docs;
```

An explicit list is taken at your word and is never filtered, so naming the
host that serves *this* build will still recurse.

## Diagram styling

Colours live in a `vars` block at the top of the generated d2, so a rendered
`.d2` can be edited by hand as well as overridden at build time. Names:
`appFill`, `appStroke`, `infraFill`, `infraStroke`, `baseFill`, `baseStroke`,
`hostFill`, `hostStroke`, `progFill`, `hostCloud`, `public`, `lan`, `mesh`.

```nix
theme = "light";
background = "#ffffff";
colors = { public = "#ff5555"; mesh = "#7fa7e8"; };
```

An unknown name is an error listing the palette. Output is text only, no icon
or image assets, ever.

## What you may edit

`wiki/src/index.md` and `wiki/book.toml` are written once and never
overwritten; everything else is rewritten on every run. In mode B, pass
`indexPage` and `bookToml` to own the two from the flake instead.

## Serving it

The vhost points straight at the docs derivation, so the wiki ships atomically
with every deploy. No daemon, no timer, no checkout.

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

| Option | Default | Effect |
|---|---|---|
| `serve.enable` | `false` | create the vhost |
| `serve.docs` | required | docs derivation, typically `lib.mkDocs { … }`. Safe with `closures = true`, which skips serving hosts; only an explicit `closures = [ … ]` naming this host recurses |
| `serve.virtualHost` | required | nginx vhost name |
| `serve.subpath` | `"wiki/book"` | path inside the derivation used as web root |
| `serve.virtualHostExtra` | `{ }` | merged into the vhost: TLS, listen addresses |
| `serve.api` | `true` | serve the [data API](./api.md) at `/api/` |
| `serve.allowOrigins` | `[ ]` | origins allowed to read this vhost cross-origin |
| `serve.history` | `false` | keep each deployed revision's snapshot for trends |
| `serve.historyLimit` | `null` | cap the number of snapshots kept |

nixdiag's own projection reads vhost names, listen addresses and `proxyPass`
only, never `root` — reading `root` would force the docs derivation during
eval.

For repos that want a checkout regenerated on a schedule instead:

```nix
services.nixdiag.timer = {
  enable = true;
  flake = "/var/lib/config";
  out = "/var/lib/config/docs";
  onCalendar = "daily";
  flags = [ "--no-svg" ];
};
```
