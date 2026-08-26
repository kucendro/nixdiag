//! `#:` topology annotations — comment lines in the documented repo's own
//! module files. Parsed render-side with rnix (comments are invisible to
//! eval); attachment and edge targets resolve against the evaluated facts,
//! so annotations describe real state, not strings.
//!
//! Grammar (one statement per line; a malformed line is a reported error):
//!   #: <role>                                  role (implicit verb)
//!   #: expose <port>[/udp] [scope] [name=<fqdn>]
//!   #: -> <host[/service] | fqdn | internet | lan> [label]   (and `<-`)
//!   #: name <fqdn>                             address-book entry
//!   #: scope public|mesh|lan
//!   #: unit <name>                             declare an unprojected node
//! `# nixdiag:` is the long alias; the same lines are recognized inside a
//! file-leading `/** */` doc comment. A contiguous run of annotation lines
//! forms one block; a `unit` declaration re-attaches its whole block.

use crate::facts::Facts;
use crate::modules::{build_import_graph, host_entry_modules};
use crate::repo::Repo;
use anyhow::Result;
use indexmap::IndexMap;
use rnix::{SyntaxKind, SyntaxNode, SyntaxToken};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Public,
    Mesh,
    Lan,
}

impl Scope {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "public" => Some(Scope::Public),
            "mesh" => Some(Scope::Mesh),
            "lan" => Some(Scope::Lan),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Scope::Public => "public",
            Scope::Mesh => "mesh",
            Scope::Lan => "lan",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expose {
    pub port: u32,
    pub udp: bool,
    pub scope: Option<Scope>,
    pub name: Option<String>,
}

#[derive(Debug)]
enum Stmt {
    Role(String),
    Expose(Expose),
    Edge {
        rev: bool,
        target: String,
        label: String,
    },
    Name(String),
    Scope(Scope),
    /// Declares a node the projection can't see (a container, a raw systemd
    /// unit); the contiguous `#:` block it sits in attaches to it.
    Unit(String),
}

#[derive(Debug, Clone, PartialEq)]
enum RawAttach {
    /// `services.<x>` / `programs.<x>` binding directly below the line.
    Unit(String),
    /// `#: unit <name>` declaration: placed on the hosts whose import graph
    /// reaches the file, independent of any binding.
    Declared(String),
    /// File-level: resolves to the host (entry module) or to what the file defines.
    File,
}

struct Raw {
    file: String,
    line: usize,
    attach: RawAttach,
    stmt: Stmt,
}

/// One node's annotation payload (a host box or a service inside it).
#[derive(Debug, Default)]
pub struct NodeInfo {
    pub role: Option<String>,
    pub scope: Option<Scope>,
    pub exposes: Vec<Expose>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Endpoint {
    Host(String),
    Unit(String, String),
    Internet,
    Lan,
}

#[derive(Debug)]
pub struct Edge {
    pub from: Endpoint,
    pub to: Endpoint,
    pub label: String,
}

#[derive(Debug, Default)]
pub struct Model {
    pub hosts: IndexMap<String, NodeInfo>,
    pub units: IndexMap<(String, String), NodeInfo>,
    pub edges: Vec<Edge>,
    /// Total parsed statements — zero triggers the getting-started hint.
    pub total: usize,
}

impl Model {
    /// Effective scope of an expose: its own, else the node's, else the host's.
    pub fn effective_scope(&self, host: &str, unit: Option<&str>, e: &Expose) -> Option<Scope> {
        e.scope
            .or_else(|| {
                unit.and_then(|u| {
                    self.units
                        .get(&(host.to_string(), u.to_string()))
                        .and_then(|i| i.scope)
                })
            })
            .or_else(|| self.hosts.get(host).and_then(|i| i.scope))
    }
}

pub struct Diag {
    pub file: String,
    pub line: usize,
    pub msg: String,
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.msg)
    }
}

// --- statement grammar ----------------------------------------------------

fn parse_stmt(body: &str) -> Result<Stmt, String> {
    let toks: Vec<&str> = body.split_whitespace().collect();
    match toks.as_slice() {
        [] => Err("empty annotation".into()),
        [arrow @ ("->" | "<-"), target, label @ ..] => Ok(Stmt::Edge {
            rev: *arrow == "<-",
            target: (*target).to_string(),
            label: label.join(" "),
        }),
        ["->" | "<-"] => Err("edge needs a target: `-> <host[/service] | fqdn> [label]`".into()),
        ["expose", port, rest @ ..] => parse_expose(port, rest),
        ["expose"] => Err("expose needs a port: `expose <port>[/udp] [scope] [name=<fqdn>]`".into()),
        ["name", fqdn] => Ok(Stmt::Name((*fqdn).to_string())),
        ["name", ..] => Err("name takes exactly one fqdn: `name <fqdn>`".into()),
        ["unit", name]
            if name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') =>
        {
            Ok(Stmt::Unit((*name).to_string()))
        }
        ["unit", ..] => Err("unit takes exactly one name: `unit <name>`".into()),
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

// --- comment scan (rnix) --------------------------------------------------

/// Body of an annotation line comment: `#: …` or `# nixdiag: …`.
fn line_comment_body(s: &str) -> Option<&str> {
    let after = s.strip_prefix('#')?;
    if let Some(b) = after.strip_prefix(':') {
        return Some(b);
    }
    after.trim_start().strip_prefix("nixdiag:")
}

/// Body of an annotation line inside a doc comment.
fn doc_line_body(l: &str) -> Option<&str> {
    let t = l.trim_start();
    t.strip_prefix("#:").or_else(|| t.strip_prefix("nixdiag:"))
}

fn line_of(text: &str, offset: usize) -> usize {
    text[..offset].bytes().filter(|b| *b == b'\n').count() + 1
}

fn own_line(text: &str, offset: usize) -> bool {
    let start = text[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    text[start..offset].chars().all(char::is_whitespace)
}

/// Ident segments of a binding's attrpath (None for string/dynamic segments).
fn attrpath_segments(binding: &SyntaxNode) -> Vec<Option<String>> {
    let Some(ap) = binding
        .children()
        .find(|c| c.kind() == SyntaxKind::NODE_ATTRPATH)
    else {
        return Vec::new();
    };
    ap.children()
        .map(|seg| {
            if seg.kind() == SyntaxKind::NODE_IDENT {
                Some(seg.text().to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Attachment of a comment token: the full attrpath of the binding it sits
/// above (or inside), outermost segment first.
fn binding_path(tok: &SyntaxToken) -> Vec<Option<String>> {
    let mut next = tok.next_token();
    while let Some(t) = &next {
        match t.kind() {
            SyntaxKind::TOKEN_WHITESPACE | SyntaxKind::TOKEN_COMMENT => next = t.next_token(),
            _ => break,
        }
    }
    let Some(t) = next else { return Vec::new() };
    let Some(node) = t.parent() else {
        return Vec::new();
    };
    let mut path: Vec<Option<String>> = Vec::new();
    for anc in node
        .ancestors()
        .filter(|n| n.kind() == SyntaxKind::NODE_ATTRPATH_VALUE)
    {
        let mut segs = attrpath_segments(&anc);
        segs.extend(path);
        path = segs;
    }
    path
}

fn attach_of_path(mut path: &[Option<String>]) -> RawAttach {
    if path.first().is_some_and(|s| s.as_deref() == Some("config")) {
        path = &path[1..];
    }
    if let [Some(first), Some(second), ..] = path {
        if first == "services" || first == "programs" {
            return RawAttach::Unit(second.clone());
        }
    }
    RawAttach::File
}

fn scan_file(rel: &str, text: &str, raws: &mut Vec<Raw>, diags: &mut Vec<Diag>) {
    let parse = rnix::Root::parse(text);
    let mut file_raws: Vec<Raw> = Vec::new();
    let mut push =
        |line: usize, attach: RawAttach, body: &str, diags: &mut Vec<Diag>| match parse_stmt(
            body.trim(),
        ) {
            Ok(stmt) => file_raws.push(Raw {
                file: rel.to_string(),
                line,
                attach,
                stmt,
            }),
            Err(msg) => diags.push(Diag {
                file: rel.to_string(),
                line,
                msg,
            }),
        };
    let mut leading = true;
    for el in parse.syntax().descendants_with_tokens() {
        let Some(tok) = el.into_token() else { continue };
        match tok.kind() {
            SyntaxKind::TOKEN_WHITESPACE => continue,
            SyntaxKind::TOKEN_COMMENT => {}
            _ => {
                leading = false;
                continue;
            }
        }
        let s = tok.text();
        let offset = usize::from(tok.text_range().start());
        // RFC 145 doc comment leading the file: directive lines are file-level.
        if s.starts_with("/**") && !s.starts_with("/***") && s.ends_with("*/") && s.len() >= 5 {
            if leading {
                let base = line_of(text, offset);
                for (i, l) in s[3..s.len() - 2].lines().enumerate() {
                    if let Some(body) = doc_line_body(l) {
                        push(base + i, RawAttach::File, body, diags);
                    }
                }
            }
            leading = false;
            continue;
        }
        let Some(body) = line_comment_body(s) else {
            continue;
        };
        let line = line_of(text, offset);
        if !own_line(text, offset) {
            diags.push(Diag {
                file: rel.to_string(),
                line,
                msg: "annotation must be on its own line".into(),
            });
            continue;
        }
        push(line, attach_of_path(&binding_path(&tok)), body, diags);
    }

    // Contiguous annotation lines form one block; a `unit <name>` declaration
    // re-attaches the whole block to that declared unit.
    let mut i = 0;
    while i < file_raws.len() {
        let mut j = i + 1;
        while j < file_raws.len() && file_raws[j].line == file_raws[j - 1].line + 1 {
            j += 1;
        }
        let mut declared: Option<String> = None;
        for r in &file_raws[i..j] {
            if let Stmt::Unit(n) = &r.stmt {
                match &declared {
                    Some(prev) => diags.push(Diag {
                        file: rel.to_string(),
                        line: r.line,
                        msg: format!("this block already declares unit `{prev}`"),
                    }),
                    None => declared = Some(n.clone()),
                }
            }
        }
        if let Some(name) = declared {
            for r in &mut file_raws[i..j] {
                r.attach = RawAttach::Declared(name.clone());
            }
        }
        i = j;
    }
    raws.extend(file_raws);
}

fn nix_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                stack.push(e.path());
            } else if name.ends_with(".nix") {
                out.push(e.path());
            }
        }
    }
    out.sort();
    out
}

// --- resolution against facts ---------------------------------------------

struct Ctx {
    /// facts host order, for deterministic attachment.
    host_order: Vec<String>,
    /// unit name -> hosts that enable it.
    unit_hosts: BTreeMap<String, Vec<String>>,
    /// repo-relative file -> (host, unit) pairs it enables.
    file_units: BTreeMap<String, Vec<(String, String)>>,
    /// host -> repo-relative files reachable from its entry modules.
    reach: HashMap<String, HashSet<String>>,
    /// repo-relative entry module -> host.
    entry_of: HashMap<String, String>,
}

impl Ctx {
    fn build(facts: &Facts, repo: &Repo) -> Self {
        let host_order: Vec<String> = facts.hosts.keys().cloned().collect();
        let mut unit_hosts: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut file_units: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        for (host, f) in &facts.hosts {
            for unit in f.units() {
                unit_hosts
                    .entry(unit.name.clone())
                    .or_default()
                    .push(host.clone());
                for rel in repo.repo_files(&unit.files) {
                    let e = file_units.entry(rel).or_default();
                    let pair = (host.clone(), unit.name.clone());
                    if !e.contains(&pair) {
                        e.push(pair);
                    }
                }
            }
        }
        let flake_text = std::fs::read_to_string(repo.root.join("flake.nix")).unwrap_or_default();
        let mut reach = HashMap::new();
        let mut entry_of = HashMap::new();
        let rel = |p: &Path| -> String {
            p.strip_prefix(&repo.root)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/")
        };
        for host in facts.hosts.keys() {
            let entries = host_entry_modules(host, &flake_text, repo);
            for e in &entries {
                entry_of.insert(rel(e), host.clone());
            }
            let (nodes, _) = build_import_graph(&entries, repo);
            reach.insert(host.clone(), nodes);
        }
        Ctx {
            host_order,
            unit_hosts,
            file_units,
            reach,
            entry_of,
        }
    }

    fn hosts_reaching(&self, file: &str) -> Vec<String> {
        self.host_order
            .iter()
            .filter(|h| self.reach.get(*h).is_some_and(|r| r.contains(file)))
            .cloned()
            .collect()
    }

    /// Where a raw annotation lands: the host box, or (host, unit) nodes.
    fn attach(&self, raw: &Raw) -> Result<Vec<Endpoint>, String> {
        match &raw.attach {
            RawAttach::Unit(u) => {
                let via = self.hosts_reaching(&raw.file);
                let hosts = match self.unit_hosts.get(u) {
                    Some(hosts) => {
                        // Narrow to hosts that import this file, when the graph knows it.
                        let narrowed: Vec<String> =
                            hosts.iter().filter(|h| via.contains(h)).cloned().collect();
                        if narrowed.is_empty() {
                            hosts.clone()
                        } else {
                            narrowed
                        }
                    }
                    // Nested enables (services.x.sub.enable) are invisible to the
                    // generic projection; the binding in this file is still real
                    // state, so fall back to the hosts that import the file.
                    None if !via.is_empty() => via,
                    None => {
                        return Err(format!(
                            "`{u}` is not enabled on any host (and no host's import graph reaches this file)"
                        ))
                    }
                };
                let mut hosts = hosts;
                hosts.sort_by_key(|h| self.host_order.iter().position(|x| x == h));
                Ok(hosts
                    .into_iter()
                    .map(|h| Endpoint::Unit(h, u.clone()))
                    .collect())
            }
            RawAttach::Declared(name) => {
                let via = self.hosts_reaching(&raw.file);
                if via.is_empty() {
                    return Err(format!(
                        "cannot place declared unit `{name}`: no host's import graph reaches this file"
                    ));
                }
                Ok(via
                    .into_iter()
                    .map(|h| Endpoint::Unit(h, name.clone()))
                    .collect())
            }
            RawAttach::File => {
                if let Some(host) = self.entry_of.get(&raw.file) {
                    return Ok(vec![Endpoint::Host(host.clone())]);
                }
                match self.file_units.get(&raw.file) {
                    Some(pairs) => Ok(pairs
                        .iter()
                        .map(|(h, u)| Endpoint::Unit(h.clone(), u.clone()))
                        .collect()),
                    None => Err(
                        "annotation attaches to nothing: not above a services./programs. \
                         binding, not a host entry module, and this file defines no \
                         service or program"
                            .into(),
                    ),
                }
            }
        }
    }
}

pub fn collect(facts: &Facts, repo: &Repo) -> (Model, Vec<Diag>) {
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
        scan_file(&rel, &text, &mut raws, &mut diags);
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
                diags.push(Diag {
                    file: raw.file.clone(),
                    line: raw.line,
                    msg,
                });
                continue;
            }
        };
        attached.push((i, targets.clone()));
        for t in &targets {
            let info = match t {
                Endpoint::Host(h) => model.hosts.entry(h.clone()).or_default(),
                Endpoint::Unit(h, u) => model.units.entry((h.clone(), u.clone())).or_default(),
                _ => unreachable!(),
            };
            let dup = |what: &str| Diag {
                file: raw.file.clone(),
                line: raw.line,
                msg: format!("{what} is already set for this target"),
            };
            match &raw.stmt {
                Stmt::Role(r) => {
                    if matches!(t, Endpoint::Host(_)) {
                        diags.push(Diag {
                            file: raw.file.clone(),
                            line: raw.line,
                            msg: format!("role `{r}` attaches to a service, not a host"),
                        });
                    } else if info.role.replace(r.clone()).is_some() {
                        diags.push(dup("a role"));
                    }
                }
                Stmt::Scope(s) => {
                    if info.scope.replace(*s).is_some() {
                        diags.push(dup("a scope"));
                    }
                }
                Stmt::Expose(e) => info.exposes.push(e.clone()),
                Stmt::Name(n) => {
                    info.names.push(n.clone());
                    book.entry(n.clone()).or_default().push(t.clone());
                }
                // The or_default above already materialized the node.
                Stmt::Unit(_) | Stmt::Edge { .. } => {}
            }
        }
    }

    // Pass 2: edges.
    for (i, sources) in &attached {
        let raw = &raws[*i];
        let Stmt::Edge { rev, target, label } = &raw.stmt else {
            continue;
        };
        let to = match resolve_target(target, &ctx, &model, &book) {
            Ok(t) => t,
            Err(msg) => {
                diags.push(Diag {
                    file: raw.file.clone(),
                    line: raw.line,
                    msg,
                });
                continue;
            }
        };
        for s in sources {
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
) -> Result<Endpoint, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(text: &str) -> (Vec<Raw>, Vec<Diag>) {
        let mut raws = Vec::new();
        let mut diags = Vec::new();
        scan_file("test.nix", text, &mut raws, &mut diags);
        (raws, diags)
    }

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
            Ok(Stmt::Edge { rev: false, ref target, ref label }) if target == "nas/grafana" && label == "metrics push"
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
    fn attaches_to_service_binding() {
        let (raws, diags) =
            scan("{\n  #: mesh-control\n  services.headscale = {\n    enable = true;\n  };\n}\n");
        assert!(
            diags.is_empty(),
            "{:?}",
            diags.iter().map(|d| d.to_string()).collect::<Vec<_>>()
        );
        assert_eq!(raws.len(), 1);
        assert_eq!(raws[0].attach, RawAttach::Unit("headscale".into()));
        assert_eq!(raws[0].line, 2);
    }

    #[test]
    fn unit_declaration_reattaches_its_block() {
        // A raw systemd unit is invisible to the parser; `unit` declares it
        // and pulls the whole contiguous block onto the declared node.
        let (raws, diags) = scan(
            "{\n  #: unit kubicek\n  #: scope mesh\n  systemd.services.kubicek = {\n    wantedBy = [ ];\n  };\n}\n",
        );
        assert!(diags.is_empty());
        assert_eq!(raws.len(), 2);
        assert_eq!(raws[0].attach, RawAttach::Declared("kubicek".into()));
        assert_eq!(raws[1].attach, RawAttach::Declared("kubicek".into()));

        // A `unit` in a block above a services binding overrides it (e.g. to
        // split a sub-service from its parent unit).
        let (raws, diags) = scan(
            "{\n  #: unit beszel-agent\n  #: agent\n  services.beszel.agent = {\n    enable = true;\n  };\n}\n",
        );
        assert!(diags.is_empty());
        assert_eq!(raws[1].attach, RawAttach::Declared("beszel-agent".into()));

        // Non-contiguous lines are separate blocks: the role keeps its own
        // binding attachment.
        let (raws, diags) =
            scan("{\n  #: unit qore\n\n  #: monitor\n  services.grafana.enable = true;\n}\n");
        assert!(diags.is_empty());
        assert_eq!(raws[0].attach, RawAttach::Declared("qore".into()));
        assert_eq!(raws[1].attach, RawAttach::Unit("grafana".into()));

        let (_, diags) = scan("{\n  #: unit a\n  #: unit b\n  x = 1;\n}\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].msg.contains("already declares"));

        assert!(matches!(parse_stmt("unit kubicek"), Ok(Stmt::Unit(n)) if n == "kubicek"));
        assert!(parse_stmt("unit two words").is_err());
        assert!(parse_stmt("unit").is_err());
    }

    #[test]
    fn attaches_inside_nested_binding() {
        let (raws, diags) = scan(
            "{\n  services.nginx = {\n    enable = true;\n    #: -> nas/grafana\n    virtualHosts.\"g.example\" = { };\n  };\n}\n",
        );
        assert!(diags.is_empty());
        assert_eq!(raws[0].attach, RawAttach::Unit("nginx".into()));
    }

    #[test]
    fn attaches_file_level_from_doc_comment() {
        let (raws, diags) = scan(
            "/**\n  The edge node.\n\n  #: name edge.example.com\n*/\n{ services.nginx.enable = true; }\n",
        );
        assert!(diags.is_empty());
        assert_eq!(raws.len(), 1);
        assert_eq!(raws[0].attach, RawAttach::File);
        assert_eq!(raws[0].line, 4);
        assert!(matches!(&raws[0].stmt, Stmt::Name(n) if n == "edge.example.com"));
    }

    #[test]
    fn long_alias_and_own_line() {
        let (raws, diags) = scan("{\n  # nixdiag: storage\n  services.zfs.enable = true;\n}\n");
        assert!(diags.is_empty());
        assert_eq!(raws[0].attach, RawAttach::Unit("zfs".into()));

        let (_, diags) = scan("{ services.zfs.enable = true; #: storage\n}\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].msg.contains("own line"));
    }

    #[test]
    fn malformed_is_reported() {
        let (raws, diags) = scan("{\n  #: expose eighty\n  services.nginx.enable = true;\n}\n");
        assert!(raws.is_empty());
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].line, 2);
    }

    #[test]
    fn plain_comments_ignored() {
        let (raws, diags) = scan("{\n  # just a note\n  services.nginx.enable = true;\n}\n");
        assert!(raws.is_empty());
        assert!(diags.is_empty());
    }
}
