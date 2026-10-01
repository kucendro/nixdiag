use std::fmt::Display;

pub const NONE: &str = "—";
pub const NOT_MEASURED: &str = "not measured";
pub const KV: [&str; 2] = ["", ""];

pub fn code(s: impl Display) -> String {
    format!("`{s}`")
}

pub fn link(text: &str, url: &str) -> String {
    format!("[{text}]({url})")
}

pub fn size_paths(size: impl Display, paths: impl Display) -> String {
    format!("{size} ({paths} paths)")
}

pub fn heading(title: &str) -> String {
    format!("# {title}")
}

pub fn generated(at: impl Display, revision: Option<&str>) -> String {
    match revision {
        Some(r) => format!("_Generated from the configuration as of {at}, `{r}`._"),
        None => format!("_Generated from the configuration as of {at}._"),
    }
}

pub fn diagram(light: &str, dark: &str) -> String {
    format!(
        "<div class=\"d2 d2-light\">{{{{#include {light}}}}}</div>\n<div class=\"d2 d2-dark\">{{{{#include {dark}}}}}</div>"
    )
}

pub const INDEX: &str = "\
# Infrastructure wiki

_Hand-written overview goes here_ — the big picture, and *why* things are the way they are. Every other page is generated from the Nix configuration; pass `indexPage` to `mkDocs` to own this one.";

pub mod architecture {
    pub const TITLE: &str = "Architecture";
    pub const TOPOLOGY: &str = "## Data-flow topology";
    pub const NETWORKS: &str = "## Networks";
    pub const OVERLAY: &str = "## Overlay";
}

pub mod summary {
    pub const TITLE: &str = "# Summary";
    pub const OVERVIEW: &str = "Overview";

    pub fn entry(title: &str, file: &str) -> String {
        format!("- [{title}](./{file})")
    }
}

pub mod hosts {
    pub const TITLE: &str = "Hosts";
    pub const DARWIN_INTRO: &str = "_nix-darwin host._";
    pub const LOCATION: &str = "Location";
    pub const PLATFORM: &str = "Platform";
    pub const UNKNOWN_PLATFORM: &str = "?";
    pub const STATE: &str = "State version";
    pub const USERS: &str = "Users";
    pub const PACKAGES: &str = "System packages";
    pub const CLOSURE: &str = "Closure";
    pub const FIREWALL: &str = "Firewall";
    pub const GATEWAY: &str = "Default gateway";
    pub const INTERFACES: &str = "**Interfaces:**";
    pub const INTERFACES_HEAD: [&str; 4] = ["Interface", "Kind", "Addresses", "Over"];
    pub const DHCP: &str = "dhcp";
    pub const SERVICES_COUNT: &str = "Repo-configured services";
    pub const DAEMONS: &str = "LaunchDaemons";
    pub const AGENTS: &str = "User agents";
    pub const CASKS: &str = "Homebrew casks";
    pub const TOPOLOGY: &str = "**Topology:**";
    pub const MODULES: &str = "**Modules:**";

    pub fn nixos(host: &str, anchor: &str) -> String {
        format!("## 🖥️ {host} {{#{anchor}}}")
    }

    pub fn darwin(host: &str, anchor: &str) -> String {
        format!("## 🍏 {host} {{#{anchor}}}")
    }

    pub fn via(address: &str, interface: &str) -> String {
        format!("{address} via {interface}")
    }

    pub fn kind(kind: &str, detail: Option<String>) -> String {
        match detail {
            Some(d) => format!("{kind} {d}"),
            None => kind.into(),
        }
    }

    pub fn listen(port: u32) -> String {
        format!("{port}/udp")
    }

    pub fn services(rows: &str) -> String {
        format!("**Services:**\n\n{rows}")
    }

    pub fn service(name: &str, files: &str) -> String {
        format!("- **{name}** — {files}")
    }

    pub fn list(title: &str, items: &str) -> String {
        format!("**{title}:** {items}")
    }
}

pub mod firewall {
    pub const TITLE: &str = "Firewall";
    pub const HEAD: [&str; 3] = ["Interface", "TCP", "UDP"];
    pub const ALL: &str = "every interface";
    pub const EVERY: &str = "every port";
    pub const OFF: &str = "**Off:** every port is open on every interface.";
    pub const FINDINGS: &str = "**Findings:**";
    pub const CLEAN: &str = "No findings.";
    pub const HOST: &str = "the host";

    pub fn count(n: usize) -> String {
        match n {
            0 => "no findings".into(),
            1 => "1 finding".into(),
            n => format!("{n} findings"),
        }
    }

    pub fn port(port: u32, udp: bool) -> String {
        format!("`{port}/{}`", if udp { "udp" } else { "tcp" })
    }

    pub fn unused(port: &str, on: &str) -> String {
        format!("- {port} is open on {on}, but nothing here uses it")
    }

    pub fn closed(unit: &str, port: &str, scope: &str, on: &str) -> String {
        format!("- {unit} exposes {port} to {scope}, but it is closed on {on}")
    }

    pub fn open_all(interface: &str) -> String {
        format!("- `{interface}` is trusted: every port on it is open")
    }

    pub fn blocked(from: &str, port: &str, on: &str) -> String {
        format!("- `{from}` connects to {port}, but it is closed on {on}")
    }

    pub fn host(host: &str, anchor: &str) -> String {
        format!("## {host} {{#{anchor}}}")
    }

    pub fn trusted(interface: &str) -> String {
        format!("`{interface}` (trusted)")
    }

    pub fn range(from: u32, to: u32) -> String {
        format!("{from}–{to}")
    }
}

pub mod services {
    pub const TITLE: &str = "Services";
    pub const HEAD: [&str; 3] = ["Service", "Hosts", "Defined in"];

    pub fn name(name: &str) -> String {
        format!("**{name}**")
    }

    pub fn unit(name: &str, description: &str) -> String {
        format!("## {name}\n\n{description}")
    }
}

pub mod endpoints {
    pub const TITLE: &str = "Endpoints";
    pub const HEAD: [&str; 5] = ["Endpoint", "Port", "Scope", "Host", "Service"];
    pub const HTTP: &str = "http";
    pub const HTTPS: &str = "https";
    pub const UDP: &str = "/udp";

    pub fn link(name: &str, scheme: &str, port: &str) -> String {
        format!("[`{name}`]({scheme}://{name}{port})")
    }

    pub fn unnamed(host: &str, port: u32) -> String {
        format!("{host}:{port}")
    }
}

pub mod inputs {
    pub const TITLE: &str = "Inputs";
    pub const INTRO: &str = "Dashed edges are `follows`, which *removes* a duplicate.";
    pub const HEAD: [&str; 4] = ["Input", "Source", "Rev", "Locked"];
    pub const DATES: &str = "\
## Lock dates

![Input dates](./inputs-timeline.svg)

`lastModified` is a fixed integer in the lock, not a clock read: this is the *spread*, not a claim about today.";
    pub const DIAMONDS: &str = "## Duplicate inputs";
    pub const DIAMOND_HEAD: [&str; 3] = ["Rev", "Node", "Pulled in by"];
    pub const THIS_FLAKE: &str = "this flake";
    pub const REDUNDANT: &str = "\
## Redundant inputs

One revision under several node names. Harmless; a `follows` drops the extra fetch.";

    pub fn span(days: i64) -> String {
        format!("**{days} days** separate the oldest input from the newest.")
    }

    pub fn diamond(source: &str, revisions: usize) -> String {
        format!("`{source}` is locked at **{revisions} revisions**, so every copy is fetched and evaluated separately:")
    }

    pub fn parent_as(parent: &str, input: &str) -> String {
        format!("`{parent}` (as `{input}`)")
    }

    pub fn fix(target: &str, lines: &str) -> String {
        format!("Point the extra copies at `{target}`:\n\n```nix\n{lines}\n```")
    }

    pub fn fix_line(parent: &str, input: &str, target: &str) -> String {
        format!("inputs.{parent}.inputs.{input}.follows = \"{target}\";")
    }

    pub fn redundant(source: &str, nodes: &str) -> String {
        format!("- `{source}` — {nodes}")
    }
}

pub mod closures {
    pub const TITLE: &str = "Closures";
    pub const CHART_CAPTION: &str = "System closure size by host";
    pub const HEAD: [&str; 4] = ["Host", "Closure", "Paths", "Unique"];
    pub const FLEET: &str = "## Fleet";
    pub const SHARED: &str = "Shared by every host";
    pub const DEDUPED: &str = "Fleet total, deduplicated";
    pub const SUM: &str = "Sum of per-host closures";
    pub const SAVED: &str = "Saved by sharing";
    pub const SERVED: &str = "Measured without the docs it serves.";
    pub const LARGEST: &str = "Largest single paths:";
    pub const LARGEST_HEAD: [&str; 2] = ["Package", "Size"];

    pub fn image(caption: &str, file: &str) -> String {
        format!("![{caption}](./{file})")
    }

    pub fn host(host: &str) -> String {
        format!("## {host}")
    }

    pub fn treemap_caption(host: &str) -> String {
        format!("{host} closure by package")
    }

    pub fn more(count: impl std::fmt::Display) -> String {
        format!("{count} more")
    }
}
