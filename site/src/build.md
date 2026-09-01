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
derivation. Your flake evaluates its own configurations at eval time, the
sandbox only renders, so nothing needs to be committed back and CI has nothing
to drift from. Add `checks.docs = self.packages.x86_64-linux.docs;` and a
broken annotation fails the build.

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

### Closure metrics

`closures` adds a Closures page (per-host totals, largest contributing
packages, and what the fleet shares) plus a closure row on each host.

The page opens with `closures.svg`, one stacked bar per host split into what
every host carries, what some of them carry, and what this host alone costs:

![Fleet closure sizes](./closures.svg)

nixdiag draws that chart itself rather than through d2, so it needs no binary
on PATH, ignores `--no-svg`, and is byte-identical run to run — which is why it
is the one picture `nixdiag check` can compare. Its colors follow `theme` and
can be overridden by name: `chartShared`, `chartPartial`, `chartUnique`,
`chartInk`, `chartMuted`, `chartTrack`.

Packages are listed by name and version, never by full store path. Nix records
a reference for every store path that appears in a build output, so printing
them would make the docs derivation retain the entire closure it describes —
and `services.nixdiag.serve` would then drag that into the serving host's own
system closure.

| value | measured |
|---|---|
| `false` | nothing (default) |
| `true` | every NixOS host that does not serve nixdiag docs |
| `[ "nas" "luna" ]` | exactly those hosts, taken at your word |

Nar sizes exist only for *realised* store paths, so each measured host's
`system.build.toplevel` becomes a build input: the docs build gets as expensive
as building those systems. Much of that is usually substituted rather than
compiled, but it is a real cost. NixOS hosts only — a darwin system cannot be
built from Linux.

A host that is not measured is still listed, with `—` in place of its numbers.
An opt-in list that silently dropped the rest would leave the page reading as
though it covered the whole fleet.

#### It measures; it does not build your fleet

The measurement is not a way to get systems built. Anything it realises is
unrooted by construction: the pages contain no store paths, so the docs
derivation holds no references to the systems it describes, and the next
`nix-collect-garbage` takes them. That is the same property that keeps
`services.nixdiag.serve` from dragging a fleet into the serving host's system,
so it is not going to change.

Turn it around and the cost disappears. Build the systems in your own
pipeline, root them there, and enable `closures` on a docs build that runs
afterwards:

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

`--out-link` is the GC root. By the time the docs build runs every path is
already realised, so the measurement is a few seconds of `jq` over data on
disk — and as a side effect a binary cache on that machine (harmonia,
nix-serve, attic) can serve the systems to the hosts that will deploy them.

If your docs derivation is *also* a flake check, the order flips: `nix flake
check` realises the measured systems on its own, and the build step is left
doing nothing but attaching roots. Put it after the check then, so only a
revision that passed gets pinned.

Run the docs build **on the machine that holds the systems**. The per-host
measurement derivation is `preferLocalBuild`, which pins it to whichever
machine invoked the build; invoking it from your laptop against a remote
builder copies every measured closure to the laptop.

#### Why serving hosts are skipped

`services.nixdiag.serve` roots an nginx vhost at a docs derivation, so that
host's system closure *contains* the docs. Measuring it from inside a docs
build would make the docs depend on a system that contains them —
`docs -> toplevel -> docs` — which Nix reports as infinite recursion, with a
trace that points nowhere useful.

`closures = true` detects this and skips those hosts, naming them in a warning.
The detection is safe because reading `serve.enable` does not force
`serve.docs`; it is forcing `serve.docs` that closes the loop.

If you serve one build and measure another, no cycle exists and you can ask for
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

Colours live in a `vars` block at the top of the generated d2, so you can edit
a rendered `.d2` by hand as well as override at build time. Names:
`appFill`, `appStroke`, `infraFill`, `infraStroke`, `baseFill`, `baseStroke`,
`hostFill`, `hostStroke`, `progFill`, `hostCloud`, `public`, `lan`, `mesh`.

```nix
theme = "light";
background = "#ffffff";
colors = { public = "#ff5555"; mesh = "#7fa7e8"; };
```

An unknown name is an error listing the palette, and a d2 compile failure
fails the render rather than shipping a stale SVG. Output is text only, no
icon or image assets, ever.

## What you may edit

`wiki/src/index.md` and `wiki/book.toml` are written once and never
overwritten: the index is your hand-written overview. Everything else carries
an auto-generated marker and is rewritten on every run. In mode B, pass
`indexPage` and `bookToml` to own them from the flake instead.

## Serving it

The NixOS module points an nginx vhost straight at the docs derivation, so the
wiki ships atomically with every deploy. No daemon, no timer, no checkout.

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

A binding this vhost carries would be read by nixdiag's own projection, so it
reads names, listen addresses and `proxyPass` only, never `root`, which would
force the docs derivation during eval.

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
