# Data API

Everything the wiki renders is also published as JSON, so a dashboard can read
what the pages show.

```sh
curl https://wiki.example.com/api/v1/hosts.json
```

The reference lives at `/api/`, rendered by Scalar from a generated OpenAPI 3.1
document.

## Endpoints

| Path | Contents |
|---|---|
| `/api/v1/index.json` | the endpoints this build published |
| `/api/v1/hosts.json` | hosts, platform, open ports, users, package counts |
| `/api/v1/services.json` | services this repo configures, their hosts and files |
| `/api/v1/topology.json` | annotated nodes, edges and endpoints |
| `/api/v1/inputs.json` | lock graph, dates, diamonds and redundancy |
| `/api/v1/closures.json` | per-host sizes by package, with the sharing split |
| `/api/v1/snapshot.json` | totals plus revision identity |
| `/api/v1/openapi.json` | the document Scalar renders |

`inputs.json` needs a `flake.lock`; `closures.json` needs
[`closures`](./build.md#closure-metrics). Absent endpoints are absent from
`index.json` and from the spec too, so the reference never describes something
this build did not publish.

`topology.json` is the one a reader could not compute for itself: resolving
`#:` annotations needs rnix over the repo source.

Node ids are spelled as in your annotations — `sol/nginx`, the same string you
write in `#: -> sol/nginx`. The diagram's own identifiers are mangled to suit
d2 and are not identities.

Closure figures are per *package*, never per store path. That is a necessity,
not a summary: Nix records a reference for every store path appearing in a
build output, so naming one would make the docs retain the closure they
describe. Since the document cannot name a path, it cannot report one.

## Versioning

`v1` is in the URL, so a future `v2` is served beside it rather than replacing
it. Every document carries `meta.schema`. Adding a key does not bump it;
removing or renaming one does, with a changelog entry.

Readers should **tolerate unknown keys** and treat an unrecognised
`meta.schema` as newer than they understand. nixdiag only ever writes this
data — it never validates what a client does with it, and it cannot tell a
third-party reader to re-run anything.

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

`allowOrigins` is deliberately a list rather than a wildcard switch. This vhost
is usually reachable only on a mesh, and `*` would turn "reachable from my
tailnet" into "readable by any page a browser on my tailnet visits".

## History

`snapshot.json` is small on purpose — a few hundred bytes plus one number per
host — because a trend means fetching many of them.

`history = true` adds a systemd oneshot that runs on activation, files this
build's snapshot under `/var/lib/nixdiag/history/<rev>.json`, and rewrites the
index beside it:

```sh
curl https://wiki.example.com/api/v1/history/index.json
curl https://wiki.example.com/api/v1/history/a1b2c3d.json
```

This is the only mutable state nixdiag has, and it lives in the module rather
than in the docs because a derivation is immutable and cannot accumulate
across deploys. nixdiag never reads history back; it only ever emits the
current snapshot. Rolling back re-files the same revision and changes nothing
else.

Snapshots are keyed by revision, so a build without one is skipped. In mode B
the revision comes from the flake automatically. In mode A there is nothing to
discover — the revision of the commit that will contain `docs/` cannot be known
while writing it — so pass it from CI:

```sh
nixdiag gen --revision "$(git rev-parse HEAD)"
```

or declare it in the flake, where an output may reference `self`:

```nix
nixdiag.revision = self.rev or self.dirtyRev or null;
```

nixdiag itself never invokes git and never reads a clock. A `-dirty` suffix,
which is what `self.dirtyRev` produces, sets `revision.dirty` so a dashboard
can drop those points from a trend.

If you would rather not keep state on the serving host, leave `history` off and
have your pipeline copy `api/v1/snapshot.json` into object storage after each
build, or let the dashboard poll and keep its own history.

## What `check` compares

Every document except `snapshot.json` is a pure function of the repo and takes
part in the drift gate, the spec included. `snapshot.json` carries the revision,
so gating it would turn CI red on every commit.

Turn the whole tree off with `mkDocs { api = false; }` or `nixdiag gen
--no-api`; `scalar = false` keeps the JSON and the spec but drops the reference
page and its bundle.

The Scalar bundle is vendored as a pinned fixed-output derivation, so the
reference works with no network at view time and no viewer's browser calls out
to a CDN. It is ~3.6 MB, which is most of what a docs derivation weighs — turn
`scalar` off if that matters more than the browsable reference. The bundled
viewer is mode B only: `nixdiag gen --api` writes the JSON and the spec, but
the CLI has no bundle and must not fetch one.
