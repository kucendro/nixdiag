use url::{Host, Url};

pub struct Target {
    pub host: Host,
    pub port: Option<u32>,
}

impl Target {
    pub fn parse(s: &str) -> Option<Target> {
        let url = Url::parse(s)
            .ok()
            .filter(Url::has_host)
            .or_else(|| Url::parse(&format!("x://{s}")).ok())?;
        Some(Target {
            host: Host::parse(url.host_str()?).ok()?,
            port: url.port_or_known_default().map(u32::from),
        })
    }

    pub fn is_loopback(&self) -> bool {
        match &self.host {
            Host::Domain(d) => d == "localhost",
            Host::Ipv4(ip) => ip.is_loopback(),
            Host::Ipv6(ip) => ip.is_loopback(),
        }
    }
}
