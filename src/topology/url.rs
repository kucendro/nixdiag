pub fn split(target: &str) -> (&str, Option<u32>) {
    let (scheme, rest) = match target.split_once("://") {
        Some((s, r)) => (Some(s), r),
        None => (None, target),
    };
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, port) = match authority.strip_prefix('[').and_then(|a| a.split_once(']')) {
        Some((h, p)) => (h, p.strip_prefix(':')),
        None => match authority.rsplit_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (authority, None),
        },
    };
    let default = match scheme {
        Some("https") => Some(443),
        Some("http") => Some(80),
        _ => None,
    };
    (host, port.and_then(|p| p.parse().ok()).or(default))
}

pub fn is_loopback(host: &str) -> bool {
    host.starts_with("127.") || host == "localhost" || host == "::1"
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
