//! The statement grammar: one statement per line, a malformed line is a
//! reported error and never silently ignored. One statement per line is also
//! what keeps `nixdiag migrate` mechanical at an edition bump.

use super::model::{Expose, Scope};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) enum Stmt {
    Role(String),
    Expose(Expose),
    Edge {
        rev: bool,
        target: String,
        label: String,
        /// `name=<fqdn>[:port]` — the endpoint the annotated node fronts.
        name: Option<String>,
        port: Option<u32>,
    },
    Name(String),
    Scope(Scope),
    /// Declares a node the projection can't see (a container, a raw systemd
    /// unit); the contiguous `#:` block it sits in attaches to it.
    Unit(String),
}

pub(super) fn parse_stmt(body: &str) -> Result<Stmt, String> {
    let toks: Vec<&str> = body.split_whitespace().collect();
    match toks.as_slice() {
        [] => Err("empty annotation".into()),
        [arrow @ ("->" | "<-"), target, rest @ ..] => {
            let mut name: Option<String> = None;
            let mut label: Vec<&str> = Vec::new();
            for t in rest {
                if let Some(n) = t.strip_prefix("name=") {
                    if n.is_empty() {
                        return Err("empty name= on edge".into());
                    }
                    if name.replace(n.to_string()).is_some() {
                        return Err("duplicate name= on edge".into());
                    }
                } else {
                    label.push(t);
                }
            }
            let (name, port) = match name {
                None => (None, None),
                Some(n) => match n.rsplit_once(':') {
                    None => (Some(n), None),
                    Some((fqdn, p)) => {
                        if fqdn.is_empty() {
                            return Err("empty fqdn in name= on edge".into());
                        }
                        let p: u32 = p
                            .parse()
                            .map_err(|_| format!("`{p}` is not a port number in name="))?;
                        (Some(fqdn.to_string()), Some(p))
                    }
                },
            };
            Ok(Stmt::Edge {
                rev: *arrow == "<-",
                target: (*target).to_string(),
                label: label.join(" "),
                name,
                port,
            })
        }
        ["->" | "<-"] => Err("edge needs a target: `-> <host[/service] | fqdn> [label]`".into()),
        ["expose", port, rest @ ..] => parse_expose(port, rest),
        ["expose"] => Err("expose needs a port: `expose <port>[/udp] [scope] [name=<fqdn>]`".into()),
        ["name", fqdn] => Ok(Stmt::Name((*fqdn).to_string())),
        ["name", ..] => Err("name takes exactly one fqdn: `name <fqdn>`".into()),
        ["unit", name] if is_unit_token(name) => Ok(Stmt::Unit((*name).to_string())),
        ["unit", ..] => Err("unit takes exactly one name: `unit <name>` or `unit <host>/<name>`".into()),
        ["scope", s] => Scope::parse(s)
            .map(Stmt::Scope)
            .ok_or_else(|| format!("unknown scope `{s}` (public|mesh|lan)")),
        ["scope", ..] => Err("scope takes exactly one of public|mesh|lan".into()),
        [role]
            if role
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') =>
        {
            Ok(Stmt::Role((*role).to_string()))
        }
        _ => Err(format!(
            "unrecognized statement `{body}` — expected a role, `expose`, `name`, `scope`, `->` or `<-`"
        )),
    }
}

fn parse_expose(port: &str, rest: &[&str]) -> Result<Stmt, String> {
    let (port_s, udp) = match port.split_once('/') {
        Some((p, "udp")) => (p, true),
        Some((p, "tcp")) => (p, false),
        Some((_, proto)) => return Err(format!("unknown protocol `{proto}` (tcp|udp)")),
        None => (port, false),
    };
    let port: u32 = port_s
        .parse()
        .map_err(|_| format!("`{port_s}` is not a port number"))?;
    let mut scope = None;
    let mut name = None;
    for t in rest {
        if let Some(s) = Scope::parse(t) {
            if scope.replace(s).is_some() {
                return Err("duplicate scope on expose".into());
            }
        } else if let Some(n) = t.strip_prefix("name=") {
            if name.replace(n.to_string()).is_some() {
                return Err("duplicate name= on expose".into());
            }
        } else {
            return Err(format!(
                "unexpected `{t}` in expose — allowed: public|mesh|lan, name=<fqdn>"
            ));
        }
    }
    Ok(Stmt::Expose(Expose {
        port,
        udp,
        scope,
        name,
    }))
}

/// `<name>` or `<host>/<name>`: a slash pins the declared unit to one host
/// (needed when several hosts' import graphs reach the file).
fn is_unit_token(s: &str) -> bool {
    let ident = |p: &str| {
        !p.is_empty()
            && p.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    };
    match s.split_once('/') {
        None => ident(s),
        Some((h, u)) => ident(h) && ident(u),
    }
}

/// `<sub>@<key>` in an fqdn position: the domain map supplies the suffix at
/// render time, so domain literals stay out of the repo source. A bare
/// `@<key>` is the domain itself; a token without `@` passes through.
pub(super) fn expand_fqdn(
    token: &str,
    domains: &BTreeMap<String, String>,
) -> Result<String, String> {
    let Some((sub, key)) = token.rsplit_once('@') else {
        return Ok(token.to_string());
    };
    let Some(domain) = domains.get(key) else {
        let known: Vec<&str> = domains.keys().map(String::as_str).collect();
        let hint = if known.is_empty() {
            "declare one via the flake's `nixdiag.domains`, mkDocs `domains`, or `--domain KEY=DOMAIN`".into()
        } else {
            format!("known keys: {}", known.join(", "))
        };
        return Err(format!("unknown domain key `@{key}` — {hint}"));
    };
    Ok(if sub.is_empty() {
        domain.clone()
    } else {
        format!("{sub}.{domain}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statements() {
        assert!(matches!(parse_stmt("proxy"), Ok(Stmt::Role(r)) if r == "proxy"));
        assert!(matches!(
            parse_stmt("expose 443 public name=vpn.example.com"),
            Ok(Stmt::Expose(Expose {
                port: 443,
                udp: false,
                scope: Some(Scope::Public),
                name: Some(_)
            }))
        ));
        assert!(matches!(
            parse_stmt("expose 51820/udp"),
            Ok(Stmt::Expose(Expose {
                port: 51820,
                udp: true,
                scope: None,
                name: None
            }))
        ));
        assert!(matches!(
            parse_stmt("-> nas/grafana metrics push"),
            Ok(Stmt::Edge { rev: false, ref target, ref label, name: None, port: None }) if target == "nas/grafana" && label == "metrics push"
        ));
        assert!(matches!(
            parse_stmt("<- lan"),
            Ok(Stmt::Edge { rev: true, .. })
        ));
        assert!(matches!(
            parse_stmt("scope mesh"),
            Ok(Stmt::Scope(Scope::Mesh))
        ));
        assert!(parse_stmt("").is_err());
        assert!(parse_stmt("expose http").is_err());
        assert!(parse_stmt("two words").is_err());
        assert!(parse_stmt("scope everywhere").is_err());
    }

    #[test]
    fn edge_name() {
        assert!(matches!(
            parse_stmt("-> nas/vaultwarden vault :8222 name=vault@home:443"),
            Ok(Stmt::Edge { ref label, name: Some(ref n), port: Some(443), .. })
                if label == "vault :8222" && n == "vault@home"
        ));
        assert!(matches!(
            parse_stmt("-> nas/gitea name=git.example.com"),
            Ok(Stmt::Edge { name: Some(ref n), port: None, .. }) if n == "git.example.com"
        ));
        assert!(parse_stmt("-> nas/gitea name=").is_err());
        assert!(parse_stmt("-> nas/gitea name=a name=b").is_err());
        assert!(parse_stmt("-> nas/gitea name=x:http").is_err());
        assert!(parse_stmt("-> nas/gitea name=:443").is_err());
    }

    #[test]
    fn domain_expansion() {
        let map: BTreeMap<String, String> =
            [("home".to_string(), "example.com".to_string())].into();
        assert_eq!(
            expand_fqdn("vault@home", &map).unwrap(),
            "vault.example.com"
        );
        assert_eq!(expand_fqdn("@home", &map).unwrap(), "example.com");
        assert_eq!(expand_fqdn("plain.fqdn", &map).unwrap(), "plain.fqdn");
        assert_eq!(expand_fqdn("nofqdn", &map).unwrap(), "nofqdn");
        let err = expand_fqdn("vault@lan", &map).unwrap_err();
        assert!(err.contains("@lan") && err.contains("home"), "{err}");
        let err = expand_fqdn("vault@home", &BTreeMap::new()).unwrap_err();
        assert!(err.contains("nixdiag.domains"), "{err}");
    }
}
