# Data API

Everything the wiki renders is also published as JSON, so a dashboard can read
what the pages show.

```sh
curl https://wiki.example.com/api/v1/hosts.json
```

| Path | Contents |
|---|---|
| `/api/v1/index.json` | the endpoints this build published |
| `/api/v1/hosts.json` | hosts, platform, open ports, users, package counts |
| `/api/v1/services.json` | services this repo configures, their hosts and files |
| `/api/v1/topology.json` | annotated nodes, edges and endpoints |
| `/api/v1/inputs.json` | lock graph, dates, diamonds and redundancy |
| `/api/v1/closures.json` | per-host sizes by package, with the sharing split |
| `/api/v1/snapshot.json` | totals plus revision identity |
| `/api/v1/openapi.json` | an OpenAPI 3.1 document describing all of the above |

`inputs.json` needs a `flake.lock`; `closures.json` needs
[`closures`](./build.md#closure-metrics). An endpoint this build did not write
is absent from `index.json` and from the spec too. No viewer is bundled — point
whichever one you prefer at `openapi.json`.

Node ids are spelled as in your annotations — `sol/nginx`, the same string you
write in `#: -> sol/nginx`. The identifiers in the diagrams are mangled to suit
d2 and are not addresses. Closure figures are per package, never per store
path.

`v1` is in the URL, so a future `v2` is served beside it rather than replacing
it. Every document carries `meta.schema`: adding a key does not bump it,
removing or renaming one does. Tolerate unknown keys, and treat an unrecognised
schema as newer than you understand.

Every document except `snapshot.json` takes part in `nixdiag check`.
`snapshot.json` carries the revision, which changes on every commit.

Turn the tree off with `mkDocs { api = false; }` or `nixdiag gen --no-api`.

## Serving it

```nix
services.nixdiag.serve = {
  enable = true;
  docs = inputs.self.packages.x86_64-linux.docs;
  virtualHost = "wiki.example.com";
  allowOrigins = [ "https://dash.example.com" ];
  history = true;
};
```

| Option | Default | Effect |
|---|---|---|
| `serve.api` | `true` | serve `/api/` from the docs derivation |
| `serve.allowOrigins` | `[ ]` | origins allowed to read this vhost cross-origin |
| `serve.history` | `false` | keep every deployed revision's snapshot, served at `/api/v1/history/` |
| `serve.historyLimit` | `null` | keep at most this many snapshots, oldest dropped first |

`allowOrigins` takes named origins rather than a wildcard: this vhost is
usually reachable only on a mesh, and `*` would turn "reachable from my
tailnet" into "readable by any page a browser on my tailnet visits".

## History

`history = true` files each deployed revision's `snapshot.json` on activation
and rewrites the index beside it:

```sh
curl https://wiki.example.com/api/v1/history/index.json
curl https://wiki.example.com/api/v1/history/a1b2c3d.json
```

Snapshots are keyed by revision, so rolling back re-files the same one and a
build without a revision is skipped. `mkDocs` takes the revision from your
flake. In mode A, pass it from CI:

```sh
nixdiag gen --revision "$(git rev-parse HEAD)"
```

or declare it in the flake, where an output may reference `self`:

```nix
nixdiag.revision = self.rev or self.dirtyRev or null;
```

A `-dirty` suffix sets `revision.dirty`, so a dashboard can drop those points
from a trend.

To keep no state on the serving host, leave `history` off and have your
pipeline copy `api/v1/snapshot.json` into object storage after each build, or
let the dashboard poll and keep its own history.
