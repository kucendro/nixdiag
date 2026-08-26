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
| `serve.docs` | required | docs derivation, typically `lib.mkDocs { … }` |
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
