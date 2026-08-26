use regex::Regex;

/// d2 identifier from an arbitrary segment.
pub fn sanitize(seg: &str) -> String {
    seg.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// "host:port" -> (host, port); no colon means the whole string is the host.
pub fn split_host_port(hostport: &str) -> (&str, &str) {
    match hostport.rfind(':') {
        Some(i) if i > 0 => (&hostport[..i], &hostport[i + 1..]),
        _ => (hostport, ""),
    }
}

/// proxyPass + extraConfig -> "host:port" (follows `set $var target;`).
pub fn resolve_upstream(pass: Option<&str>, extra: &str) -> Option<String> {
    let pass = pass?;
    let var_re = Regex::new(r"^https?://\$([A-Za-z0-9_]+)").unwrap();
    if let Some(m) = var_re.captures(pass) {
        let set_re = Regex::new(&format!(
            r"set\s+\${}\s+([^;\s]+)\s*;",
            regex::escape(&m[1])
        ))
        .unwrap();
        return set_re.captures(extra).map(|c| c[1].to_string());
    }
    Regex::new(r"^https?://([^/]+)")
        .unwrap()
        .captures(pass)
        .map(|c| c[1].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_resolution() {
        assert_eq!(
            resolve_upstream(Some("http://127.0.0.1:3000"), ""),
            Some("127.0.0.1:3000".into())
        );
        assert_eq!(
            resolve_upstream(Some("http://nas:80/x"), ""),
            Some("nas:80".into())
        );
        assert_eq!(
            resolve_upstream(Some("http://$up"), "set $up 10.0.0.2:9090;"),
            Some("10.0.0.2:9090".into())
        );
        assert_eq!(resolve_upstream(Some("http://$up"), ""), None);
        assert_eq!(resolve_upstream(None, ""), None);
    }
}
