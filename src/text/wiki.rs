pub const NONE: &str = "—";
pub const CODE: &str = "`{code}`";
pub const NOT_MEASURED: &str = "not measured";
pub const KV: [&str; 2] = ["", ""];
pub const SIZE_PATHS: &str = "{size} ({paths} paths)";

pub const INDEX: &str = "\
# Infrastructure wiki

_Hand-written overview goes here_ — the big picture, and *why* things are the way they are. Every other page is generated from the Nix configuration; pass `indexPage` to `mkDocs` to own this one.";

pub const ARCHITECTURE: &str = "\
# Architecture

## Data-flow topology

![Data-flow topology](./topology.svg)

## Module tree

![Module tree](./modules.svg)";

pub mod summary {
    pub const TITLE: &str = "# Summary";
    pub const FIXED: &str = "\
- [Overview](./index.md)
- [Architecture](./architecture.md)
- [Hosts](./hosts.md)
- [Services](./services.md)
- [Endpoints](./endpoints.md)";
    pub const INPUTS: &str = "- [Inputs](./inputs.md)";
    pub const CLOSURES: &str = "- [Closures](./closures.md)";
    pub const EXTRA: &str = "- [{title}](./{file})";
}

pub mod hosts {
    pub const TITLE: &str = "# Hosts";
    pub const NIXOS: &str = "## 🖥️ {host}";
    pub const DARWIN: &str = "## 🍏 {host}";
    pub const DARWIN_INTRO: &str = "_nix-darwin host._";
    pub const PLATFORM: &str = "Platform";
    pub const UNKNOWN_PLATFORM: &str = "?";
    pub const STATE: &str = "State version";
    pub const USERS: &str = "Users";
    pub const PACKAGES: &str = "System packages";
    pub const CLOSURE: &str = "Closure";
    pub const TCP: &str = "Open TCP ports";
    pub const UDP: &str = "Open UDP ports";
    pub const SERVICES_COUNT: &str = "Repo-configured services";
    pub const SERVICES: &str = "**Services:**\n\n{rows}";
    pub const SERVICE: &str = "- **{name}** — {files}";
    pub const LIST: &str = "**{title}:** {items}";
    pub const DAEMONS: &str = "LaunchDaemons";
    pub const AGENTS: &str = "User agents";
    pub const CASKS: &str = "Homebrew casks";
}

pub mod services {
    pub const TITLE: &str = "# Services";
    pub const HEAD: [&str; 3] = ["Service", "Hosts", "Defined in"];
    pub const NAME: &str = "**{name}**";
    pub const UNIT: &str = "## {name}\n\n{description}";
}

pub mod endpoints {
    pub const TITLE: &str = "# Endpoints";
    pub const HEAD: [&str; 5] = ["Endpoint", "Port", "Scope", "Host", "Service"];
    pub const LINK: &str = "[`{name}`]({scheme}://{name}{port})";
    pub const HTTP: &str = "http";
    pub const HTTPS: &str = "https";
    pub const UNNAMED: &str = "{host}:{port}";
    pub const UDP: &str = "/udp";
}

pub mod inputs {
    pub const TITLE: &str = "# Inputs";
    pub const INTRO: &str = "\
Dashed edges are `follows`, which *removes* a duplicate.

![Input graph](./inputs.svg)";
    pub const HEAD: [&str; 4] = ["Input", "Source", "Rev", "Locked"];
    pub const DATES_CAPTION: &str = "Locked inputs by date, oldest first";
    pub const DATES: &str = "\
## Lock dates

![Input dates](./inputs-timeline.svg)

`lastModified` is a fixed integer in the lock, not a clock read: this is the *spread*, not a claim about today.";
    pub const SPAN: &str = "**{days} days** separate the oldest input from the newest.";
    pub const DIAMONDS: &str = "## Duplicate inputs";
    pub const DIAMOND: &str = "`{source}` is locked at **{revisions} revisions**, so every copy is fetched and evaluated separately:";
    pub const DIAMOND_HEAD: [&str; 3] = ["Rev", "Node", "Pulled in by"];
    pub const THIS_FLAKE: &str = "this flake";
    pub const PARENT_AS: &str = "`{parent}` (as `{input}`)";
    pub const FIX: &str = "Point the extra copies at `{target}`:\n\n```nix\n{lines}\n```";
    pub const FIX_LINE: &str = "inputs.{parent}.inputs.{input}.follows = \"{target}\";";
    pub const REDUNDANT: &str = "\
## Redundant inputs

One revision under several node names. Harmless; a `follows` drops the extra fetch.

{rows}";
    pub const REDUNDANT_ROW: &str = "- `{source}` — {nodes}";
}

pub mod closures {
    pub const TITLE: &str = "# Closures";
    pub const CHART_CAPTION: &str = "System closure size by host";
    pub const CHART: &str = "![{caption}](./closures.svg)";
    pub const HEAD: [&str; 4] = ["Host", "Closure", "Paths", "Unique"];
    pub const FLEET: &str = "## Fleet";
    pub const SHARED: &str = "Shared by every host";
    pub const DEDUPED: &str = "Fleet total, deduplicated";
    pub const SUM: &str = "Sum of per-host closures";
    pub const SAVED: &str = "Saved by sharing";
    pub const HOST: &str = "## {host}";
    pub const SERVED: &str = "Measured without the docs it serves.";
    pub const TREEMAP_CAPTION: &str = "{host} closure by package";
    pub const TREEMAP: &str = "![{caption}](./{file})";
    pub const MORE: &str = "{count} more";
    pub const LARGEST: &str = "Largest single paths:";
    pub const LARGEST_HEAD: [&str; 2] = ["Package", "Size"];
}
