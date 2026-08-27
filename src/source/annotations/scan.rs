//! The rnix pass over the documented repo's `.nix` files.
//!
//! Comments are invisible to eval, so annotations are read from the source
//! text at render time. This module only *finds* statements and records where
//! each one attaches syntactically; resolving those attachments against the
//! evaluated facts is `resolve`'s job.

use super::diag::Diag;
use super::grammar::{canonicalize, DEPRECATIONS};
use super::stmt::{parse_stmt, Stmt};
use rnix::{SyntaxKind, SyntaxNode, SyntaxToken};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RawAttach {
    /// `services.<x>` / `programs.<x>` binding directly below the line.
    Unit(String),
    /// `#: unit <name>` declaration: placed on the hosts whose import graph
    /// reaches the file, independent of any binding.
    Declared(String),
    /// File-level: resolves to the host (entry module) or to what the file defines.
    File,
}

pub(super) struct Raw {
    pub(super) file: String,
    pub(super) line: usize,
    pub(super) attach: RawAttach,
    pub(super) stmt: Stmt,
    /// Came from the file-leading `/** */` doc comment.
    pub(super) doc: bool,
}

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

pub(super) fn scan_file(
    rel: &str,
    text: &str,
    edition: u32,
    raws: &mut Vec<Raw>,
    diags: &mut Vec<Diag>,
) {
    let parse = rnix::Root::parse(text);
    let mut file_raws: Vec<Raw> = Vec::new();
    let mut push =
        |line: usize, attach: RawAttach, body: &str, doc: bool, diags: &mut Vec<Diag>| {
            // Deprecated spellings are rewritten (with a warning) before the
            // grammar proper sees them; removed ones are a hard error.
            let body = match canonicalize(body.trim(), edition, DEPRECATIONS) {
                Ok((body, warn)) => {
                    if let Some(w) = warn {
                        diags.push(Diag::deprecated(rel, line, w));
                    }
                    body
                }
                Err(msg) => {
                    diags.push(Diag::error(rel, line, msg));
                    return;
                }
            };
            match parse_stmt(&body) {
                Ok(stmt) => file_raws.push(Raw {
                    file: rel.to_string(),
                    line,
                    attach,
                    stmt,
                    doc,
                }),
                Err(msg) => diags.push(Diag::error(rel, line, msg)),
            }
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
                        push(base + i, RawAttach::File, body, true, diags);
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
            diags.push(Diag::error(rel, line, "annotation must be on its own line"));
            continue;
        }
        push(
            line,
            attach_of_path(&binding_path(&tok)),
            body,
            false,
            diags,
        );
    }

    // A `unit` declared in the file-leading doc comment is the file's default
    // attachment: file-level lines elsewhere in the file attach to it (e.g. a
    // data file, imported with a plain `import`, whose entries feed a service
    // defined somewhere else). Per-binding attachment still wins.
    let file_default = file_raws.iter().find_map(|r| match (&r.stmt, r.doc) {
        (Stmt::Unit(n), true) => Some(n.clone()),
        _ => None,
    });

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
                    Some(prev) => diags.push(Diag::error(
                        rel,
                        r.line,
                        format!("this block already declares unit `{prev}`"),
                    )),
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
    if let Some(name) = file_default {
        for r in &mut file_raws {
            if r.attach == RawAttach::File {
                r.attach = RawAttach::Declared(name.clone());
            }
        }
    }
    raws.extend(file_raws);
}

pub(super) fn nix_files(root: &Path) -> Vec<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::super::grammar::GRAMMAR;
    use super::*;

    fn scan(text: &str) -> (Vec<Raw>, Vec<Diag>) {
        let mut raws = Vec::new();
        let mut diags = Vec::new();
        scan_file("test.nix", text, GRAMMAR, &mut raws, &mut diags);
        (raws, diags)
    }

    #[test]
    fn deprecation_is_a_warning_not_an_error() {
        let mut raws = Vec::new();
        let mut diags = Vec::new();
        scan_file(
            "test.nix",
            "{\n  #: monitor\n  services.grafana.enable = true;\n}\n",
            GRAMMAR,
            &mut raws,
            &mut diags,
        );
        // No deprecations exist yet, so a clean file stays clean.
        assert!(diags.is_empty());
        assert_eq!(raws.len(), 1);
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
        assert!(matches!(parse_stmt("unit edge/nginx"), Ok(Stmt::Unit(n)) if n == "edge/nginx"));
        assert!(parse_stmt("unit two words").is_err());
        assert!(parse_stmt("unit a/b/c").is_err());
        assert!(parse_stmt("unit /x").is_err());
        assert!(parse_stmt("unit").is_err());
    }

    #[test]
    fn doc_unit_is_file_default() {
        let (raws, diags) = scan(
            "/**\n  Upstream table.\n\n  #: unit nginx\n  #: scope mesh\n*/\n{\n  #: -> luna/grafana grafana :3000\n  grafana = \"x:3000\";\n}\n",
        );
        assert!(diags.is_empty());
        assert_eq!(raws.len(), 3);
        for r in &raws {
            assert_eq!(r.attach, RawAttach::Declared("nginx".into()));
        }

        // A services./programs. binding below still wins over the file default.
        let (raws, diags) = scan(
            "/**\n  #: unit nginx\n*/\n{\n  #: monitor\n  services.grafana.enable = true;\n}\n",
        );
        assert!(diags.is_empty());
        assert_eq!(raws[1].attach, RawAttach::Unit("grafana".into()));
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
