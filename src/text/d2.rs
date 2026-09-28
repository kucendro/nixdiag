pub const IMPORT: &str = "...@theme";

pub fn var(name: &str, value: &str) -> String {
    format!("  {name}: {value}")
}

pub fn vars(lines: &str) -> String {
    format!("vars: {{\n{lines}\n}}")
}

pub fn fill(color: &str) -> String {
    format!("style.fill: {color}")
}

pub mod topology {
    pub const INTERNET: &str = "🌐 Internet";
    pub const LAN: &str = "🏠 LAN";
    pub const MESH: &str = "🔒 Mesh";
    pub const NIXOS_ICON: &str = "🖥️";
    pub const DARWIN_ICON: &str = "🍏";
    pub const NO_PORTS: &str = "—";
    pub const PORTS_SEP: &str = ", ";
    pub const TCP: &str = "tcp";
    pub const UDP: &str = "udp";

    pub fn host(icon: &str, host: &str) -> String {
        format!("{icon} {host}")
    }

    pub fn role(role: &str) -> String {
        role.replace('-', " ")
    }

    pub fn port(port: u32, udp: bool) -> String {
        format!("{port}/{}", if udp { UDP } else { TCP })
    }

    pub fn unit(lines: &[&str]) -> String {
        lines.join("\n")
    }

    pub fn ghost(host: &str, unit: &str) -> String {
        format!("{host} / {unit}")
    }

    pub fn location(name: &str) -> String {
        format!("📍 {name}")
    }

    pub fn flows(count: usize) -> String {
        format!("{count} flows")
    }

    pub fn expose(name: Option<&str>, port: Option<u32>, udp: bool) -> String {
        let port = port.map(|p| p.to_string()).unwrap_or_default();
        let proto = if udp { "/udp" } else { "" };
        match name {
            Some(n) => format!("{n} :{port}{proto}"),
            None => format!(":{port}{proto}"),
        }
    }
}

pub mod networks {
    use std::fmt::Display;

    pub fn network(name: Option<&str>, cidrs: &str) -> String {
        match name {
            Some(n) => format!("{n}\n{cidrs}"),
            None => cidrs.into(),
        }
    }

    pub fn attach(interface: &str, address: Option<impl Display>) -> String {
        match address {
            Some(a) => format!("{interface} {a}"),
            None => interface.into(),
        }
    }

    pub const SERVER: &str = "control server";

    pub fn route(interface: &str) -> String {
        format!("routes via {interface}")
    }

    pub fn server(label: &str) -> String {
        format!("{label}\n{SERVER}")
    }

    pub fn gateway(address: impl Display) -> String {
        format!("gateway {address}")
    }
}

pub mod modules {
    pub const SERVICE: &str = "service";
    pub const PROGRAM: &str = "program";
}

pub mod inputs {
    pub const ROOT: &str = "this flake";

    pub fn flagged(name: &str, rev: &str) -> String {
        format!("{name}\n{rev}")
    }
}
