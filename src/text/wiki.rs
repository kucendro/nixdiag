pub const NONE: &str = "—";

pub const INDEX: &str = "\
# Infrastructure wiki

_Hand-written overview goes here_ — the big picture, and *why* things are the way they are. Every other page is generated from the Nix configuration; this is the one you edit.";

pub const BOOK: &str = "\
[book]
title = \"{title}\"
src = \"src\"

[output.html]
default-theme = \"{default}\"
preferred-dark-theme = \"{dark}\"
no-section-label = true";

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
    pub const TABLE: &str = "| | |\n|---|---|\n{rows}";
    pub const PLATFORM: &str = "| Platform | `{platform}` |";
    pub const UNKNOWN_PLATFORM: &str = "?";
    pub const STATE: &str = "| State version | `{state}` |";
    pub const USERS: &str = "| Users | {users} |";
    pub const PACKAGES: &str = "| System packages | {count} |";
    pub const CLOSURE: &str = "| Closure | {closure} |";
    pub const CLOSURE_SIZE: &str = "{size} ({paths} paths)";
    pub const NOT_MEASURED: &str = "not measured";
    pub const TCP: &str = "| Open TCP ports | {ports} |";
    pub const UDP: &str = "| Open UDP ports | {ports} |";
    pub const SERVICES_COUNT: &str = "| Repo-configured services | {count} |";
    pub const SERVICES: &str = "**Services:**\n\n{rows}";
    pub const SERVICE: &str = "- **{name}** — {files}";
    pub const FILE: &str = "`{file}`";
    pub const LIST: &str = "**{title}:** {items}";
    pub const DAEMONS: &str = "LaunchDaemons";
    pub const AGENTS: &str = "User agents";
    pub const CASKS: &str = "Homebrew casks";
}

pub mod services {
    pub const TITLE: &str = "# Services";
    pub const TABLE: &str = "| Service | Hosts | Defined in |\n|---|---|---|\n{rows}";
    pub const ROW: &str = "| **{name}** | {hosts} | {files} |";
    pub const FILE: &str = "`{file}`";
    pub const EMPTY: &str = "| — | — | — |";
    pub const UNIT: &str = "## {name}\n\n{description}";
}

pub mod endpoints {
    pub const TITLE: &str = "# Endpoints";
    pub const TABLE: &str =
        "| Endpoint | Port | Scope | Host | Service |\n|---|---|---|---|---|\n{rows}";
    pub const ROW: &str = "| `{endpoint}` | {port} | {scope} | {host} | {service} |";
    pub const EMPTY: &str = "| — | — | — | — | — |";
    pub const UNNAMED: &str = "{host}:{port}";
    pub const UDP: &str = "/udp";
    pub const INTERNET: &str = "internet";
    pub const LAN: &str = "lan";
}

pub mod inputs {
    pub const TITLE: &str = "# Inputs";
    pub const INTRO: &str = "\
Dashed edges are `follows`, which *removes* a duplicate.

![Input graph](./inputs.svg)";
    pub const TABLE: &str = "| Input | Source | Rev | Locked |\n|---|---|---|---|\n{rows}";
    pub const ROW: &str = "| `{name}` | `{source}` | `{rev}` | {date} |";
    pub const EMPTY: &str = "| — | — | — | — |";
    pub const DATES_CAPTION: &str = "Locked inputs by date, oldest first";
    pub const DATES: &str = "\
## Lock dates

![Input dates](./inputs-timeline.svg)

`lastModified` is a fixed integer in the lock, not a clock read: this is the *spread*, not a claim about today.";
    pub const SPAN: &str = "**{days} days** separate the oldest input from the newest.";
    pub const DIAMONDS: &str = "## Duplicate inputs";
    pub const DIAMOND: &str = "\
`{source}` is locked at **{revisions} revisions**, so every copy is fetched and evaluated separately:

| Rev | Node | Pulled in by |
|---|---|---|
{rows}";
    pub const DIAMOND_ROW: &str = "| `{rev}` | `{node}` | {parents} |";
    pub const THIS_FLAKE: &str = "this flake";
    pub const PARENT: &str = "`{parent}`";
    pub const PARENT_AS: &str = "`{parent}` (as `{input}`)";
    pub const FIX: &str = "Point the extra copies at `{target}`:\n\n```nix\n{lines}\n```";
    pub const FIX_LINE: &str = "inputs.{parent}.inputs.{input}.follows = \"{target}\";";
    pub const REDUNDANT: &str = "\
## Redundant inputs

One revision under several node names. Harmless; a `follows` drops the extra fetch.

{rows}";
    pub const REDUNDANT_ROW: &str = "- `{source}` — {nodes}";
    pub const NODE: &str = "`{node}`";
}

pub mod closures {
    pub const TITLE: &str = "# Closures";
    pub const CHART_CAPTION: &str = "System closure size by host";
    pub const CHART: &str = "![{caption}](./closures.svg)";
    pub const TABLE: &str = "| Host | Closure | Paths | Unique |\n|---|---|---|---|\n{rows}";
    pub const ROW: &str = "| `{host}` | {closure} | {paths} | {unique} |";
    pub const ROW_UNMEASURED: &str = "| `{host}` | — | — | — |";
    pub const EMPTY: &str = "| — | — | — | — |";
    pub const NOT_MEASURED: &str = "not measured";
    pub const FLEET: &str = "\
## Fleet

| | |
|---|---|
| Shared by every host | {shared} ({shared_paths} paths) |
| Fleet total, deduplicated | {deduped} ({deduped_paths} paths) |
| Sum of per-host closures | {sum} |
| Saved by sharing | {saved} |";
    pub const HOST: &str = "## {host}";
    pub const TREEMAP_CAPTION: &str = "{host} closure by package";
    pub const TREEMAP: &str = "![{caption}](./{file})";
    pub const MORE: &str = "{count} more";
    pub const LARGEST: &str = "Largest single paths:\n\n| Package | Size |\n|---|---|\n{rows}";
    pub const LARGEST_ROW: &str = "| `{package}` | {size} |";
    pub const LARGEST_EMPTY: &str = "| — | — |";
}
