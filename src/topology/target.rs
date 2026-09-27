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

#[cfg(test)]
mod tests {
    use super::*;

    fn split(s: &str) -> (String, Option<u32>) {
        let t = Target::parse(s).unwrap();
        (t.host.to_string(), t.port)
    }

    #[test]
    fn urls_split_into_host_and_port_with_scheme_defaults() {
        assert_eq!(
            split("https://hs.ts.example"),
            ("hs.ts.example".into(), Some(443))
        );
        assert_eq!(
            split("http://luna.ts.example:3000/api"),
            ("luna.ts.example".into(), Some(3000))
        );
        assert_eq!(split("127.0.0.1:8080"), ("127.0.0.1".into(), Some(8080)));
        assert_eq!(split("db.example:5432"), ("db.example".into(), Some(5432)));
        assert_eq!(split("db.example"), ("db.example".into(), None));
    }

    #[test]
    fn loopback_is_any_local_address() {
        for s in [
            "127.0.0.2:80",
            "http://[::1]:9090",
            "localhost:3000",
            "postgres://127.0.0.1",
        ] {
            assert!(Target::parse(s).unwrap().is_loopback());
        }
        assert!(!Target::parse("10.0.0.1:80").unwrap().is_loopback());
    }
}
