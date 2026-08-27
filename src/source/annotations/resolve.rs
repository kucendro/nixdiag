//! Raw statements + facts -> the annotation `Model`.
//!
//! Two passes, and the order matters: roles, scopes, exposes and `#: name`
//! entries first, so the address book is complete before any edge tries to
//! resolve a target against it.

use super::attach::Ctx;
use super::diag::Diag;
use super::model::{Edge, Endpoint, Expose, Model, NamedEndpoint};
use super::scan::{nix_files, scan_file};
use super::stmt::{expand_fqdn, Stmt};
use crate::facts::Facts;
use crate::source::repo::Repo;
use std::collections::BTreeMap;

pub fn collect(
    facts: &Facts,
    repo: &Repo,
    domains: &BTreeMap<String, String>,
    edition: u32,
) -> (Model, Vec<Diag>) {
    let mut raws = Vec::new();
    let mut diags = Vec::new();
    for path in nix_files(&repo.root) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if !text.contains("#:") && !text.contains("nixdiag:") {
            continue;
        }
        let rel = path
            .strip_prefix(&repo.root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        scan_file(&rel, &text, edition, &mut raws, &mut diags);
    }

    let ctx = Ctx::build(facts, repo);
    let mut model = Model {
        total: raws.len(),
        ..Model::default()
    };
    let mut book: BTreeMap<String, Vec<Endpoint>> = BTreeMap::new();

    // Pass 1: roles, exposes, names, scopes (the address book must be complete
    // before edges resolve).
    let mut attached: Vec<(usize, Vec<Endpoint>)> = Vec::new();
    for (i, raw) in raws.iter().enumerate() {
        let targets = match ctx.attach(raw) {
            Ok(t) => t,
            Err(msg) => {
                diags.push(Diag::error(&raw.file, raw.line, msg));
                continue;
            }
        };
        // `@key` fqdn positions expand once per statement; a failed expansion
        // reports one diagnostic and drops the statement.
        let expanded_name = match &raw.stmt {
            Stmt::Name(n) | Stmt::Expose(Expose { name: Some(n), .. }) => {
                match expand_fqdn(n, domains) {
                    Ok(v) => Some(v),
                    Err(msg) => {
                        diags.push(Diag::error(&raw.file, raw.line, msg));
                        continue;
                    }
                }
            }
            _ => None,
        };
        attached.push((i, targets.clone()));
        for t in &targets {
            let info = match t {
                Endpoint::Host(h) => model.hosts.entry(h.clone()).or_default(),
                Endpoint::Unit(h, u) => model.units.entry((h.clone(), u.clone())).or_default(),
                _ => unreachable!(),
            };
            let dup = |what: &str| {
                Diag::error(
                    &raw.file,
                    raw.line,
                    format!("{what} is already set for this target"),
                )
            };
            match &raw.stmt {
                Stmt::Role(r) => {
                    if matches!(t, Endpoint::Host(_)) {
                        diags.push(Diag::error(
                            &raw.file,
                            raw.line,
                            format!("role `{r}` attaches to a service, not a host"),
                        ));
                    } else if info.role.replace(r.clone()).is_some() {
                        diags.push(dup("a role"));
                    }
                }
                Stmt::Scope(s) => {
                    if info.scope.replace(*s).is_some() {
                        diags.push(dup("a scope"));
                    }
                }
                Stmt::Expose(e) => {
                    let mut e = e.clone();
                    if e.name.is_some() {
                        e.name = expanded_name.clone();
                    }
                    info.exposes.push(e);
                }
                Stmt::Name(_) => {
                    if let Some(n) = &expanded_name {
                        info.names.push(n.clone());
                        book.entry(n.clone()).or_default().push(t.clone());
                    }
                }
                // The or_default above already materialized the node.
                Stmt::Unit(_) | Stmt::Edge { .. } => {}
            }
        }
    }

    // Pass 2: edges.
    for (i, sources) in &attached {
        let raw = &raws[*i];
        let Stmt::Edge {
            rev,
            target,
            label,
            name,
            port,
        } = &raw.stmt
        else {
            continue;
        };
        let mut err = |msg: String| diags.push(Diag::error(&raw.file, raw.line, msg));
        let to = match resolve_target(target, &ctx, &model, &book, domains) {
            Ok(t) => t,
            Err(msg) => {
                err(msg);
                continue;
            }
        };
        let name = match name {
            Some(n) => match expand_fqdn(n, domains) {
                Ok(v) => Some(v),
                Err(msg) => {
                    err(msg);
                    continue;
                }
            },
            None => None,
        };
        for s in sources {
            if let Some(n) = &name {
                model.named.push(NamedEndpoint {
                    name: n.clone(),
                    port: *port,
                    node: s.clone(),
                    target: to.clone(),
                });
            }
            let (from, to) = if *rev {
                (to.clone(), s.clone())
            } else {
                (s.clone(), to.clone())
            };
            model.edges.push(Edge {
                from,
                to,
                label: label.clone(),
            });
        }
    }

    (model, diags)
}

fn resolve_target(
    target: &str,
    ctx: &Ctx,
    model: &Model,
    book: &BTreeMap<String, Vec<Endpoint>>,
    domains: &BTreeMap<String, String>,
) -> Result<Endpoint, String> {
    let target = expand_fqdn(target, domains)?;
    let target = target.as_str();
    match target {
        "internet" => return Ok(Endpoint::Internet),
        "lan" => return Ok(Endpoint::Lan),
        _ => {}
    }
    if let Some((h, u)) = target.split_once('/') {
        if !ctx.host_order.iter().any(|x| x == h) {
            return Err(format!("unknown host `{h}` in edge target `{target}`"));
        }
        let enabled = ctx
            .unit_hosts
            .get(u)
            .is_some_and(|hs| hs.iter().any(|x| x == h));
        let annotated = model.units.contains_key(&(h.to_string(), u.to_string()));
        if !enabled && !annotated {
            return Err(format!("service `{u}` is not enabled on `{h}`"));
        }
        return Ok(Endpoint::Unit(h.to_string(), u.to_string()));
    }
    if ctx.host_order.iter().any(|x| x == target) {
        return Ok(Endpoint::Host(target.to_string()));
    }
    if let Some(hosts) = ctx.unit_hosts.get(target) {
        return match hosts.as_slice() {
            [h] => Ok(Endpoint::Unit(h.clone(), target.to_string())),
            _ => Err(format!(
                "`{target}` is enabled on several hosts ({}) — use host/{target}",
                hosts.join(", ")
            )),
        };
    }
    if target.contains('.') {
        return match book.get(target).map(Vec::as_slice) {
            Some([e]) => Ok(e.clone()),
            Some(_) => Err(format!("fqdn `{target}` resolves to several nodes")),
            None => Err(format!(
                "unknown fqdn `{target}` — no `#: name {target}` declares it"
            )),
        };
    }
    Err(format!(
        "cannot resolve edge target `{target}` — not a host, a uniquely enabled \
         service, a declared fqdn, `internet` or `lan`"
    ))
}
