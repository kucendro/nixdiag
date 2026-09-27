use std::net::IpAddr;

pub struct Target<'a> {
    pub host: &'a str,
    pub port: Option<u32>,
}

impl<'a> Target<'a> {
    pub fn parse(s: &'a str) -> Self {
        let (scheme, rest) = s.split_once("://").map_or((None, s), |(a, b)| (Some(a), b));
        let authority = rest.split('/').next().unwrap_or(rest);
        let (host, port) = match authority.strip_prefix('[').and_then(|a| a.split_once(']')) {
            Some((h, p)) => (h, p.strip_prefix(':')),
            None => authority
                .rsplit_once(':')
                .map_or((authority, None), |(h, p)| (h, Some(p))),
        };
        let default = match scheme {
            Some("https") => Some(443),
            Some("http") => Some(80),
            _ => None,
        };
        let port = port.and_then(|p| p.parse().ok()).or(default);
        Target { host, port }
    }

    pub fn is_loopback(&self) -> bool {
        self.host == "localhost" || self.host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(s: &str) -> (&str, Option<u32>) {
        let t = Target::parse(s);
        (t.host, t.port)
    }

    #[test]
    fn urls_split_into_host_and_port_with_scheme_defaults() {
        assert_eq!(split("https://hs.ts.example"), ("hs.ts.example", Some(443)));
        assert_eq!(
            split("http://luna.ts.example:3000/api"),
            ("luna.ts.example", Some(3000))
        );
        assert_eq!(split("127.0.0.1:8080"), ("127.0.0.1", Some(8080)));
        assert_eq!(split("http://[::1]:9090"), ("::1", Some(9090)));
        assert_eq!(split("db.example"), ("db.example", None));
    }

    #[test]
    fn loopback_is_any_local_address() {
        for s in ["127.0.0.2:80", "http://[::1]:9090", "localhost:3000"] {
            assert!(Target::parse(s).is_loopback());
        }
        assert!(!Target::parse("10.0.0.1:80").is_loopback());
    }
}
