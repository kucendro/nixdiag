pub mod topology {
    pub const INTERNET: &str = "🌐 Internet";
    pub const LAN: &str = "🏠 LAN";
    pub const NIXOS_ICON: &str = "🖥️";
    pub const DARWIN_ICON: &str = "🍏";
    pub const PORTS_SEP: &str = " · ";
    pub const UDP: &str = "/udp";

    pub fn host(icon: &str, host: &str) -> String {
        format!("{icon} {host}")
    }

    pub fn role(role: &str) -> String {
        format!("({})", role.replace('-', " "))
    }

    pub fn tcp(ports: &str) -> String {
        format!("tcp {ports}")
    }

    pub fn udp(ports: &str) -> String {
        format!("udp {ports}")
    }

    pub fn base(count: usize) -> String {
        format!("+ {count} system services")
    }

    pub fn expose(name: Option<&str>, port: Option<u32>, udp: bool) -> String {
        let port = port.map(|p| p.to_string()).unwrap_or_default();
        let proto = if udp { UDP } else { "" };
        match name {
            Some(n) => format!("{n} :{port}{proto}"),
            None => format!(":{port}{proto}"),
        }
    }
}

pub mod inputs {
    pub const ROOT: &str = "this flake";

    pub fn flagged(name: &str, rev: &str) -> String {
        format!("{name} {rev}")
    }
}
