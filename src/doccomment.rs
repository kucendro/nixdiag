//! RFC 145 doc comments: a leading `/** ... */` block in a module file is
//! that file's documentation (Markdown body).

use std::path::Path;

/// First-token-of-file doc comment, dedented. Parsed with rnix so string
/// contents and nested expressions can never fool the scan.
pub fn leading_doc(text: &str) -> Option<String> {
    let parse = rnix::Root::parse(text);
    for el in parse.syntax().descendants_with_tokens() {
        let Some(tok) = el.into_token() else { continue };
        match tok.kind() {
            rnix::SyntaxKind::TOKEN_WHITESPACE => continue,
            rnix::SyntaxKind::TOKEN_COMMENT => {
                let s = tok.text();
                // `/**` opens a doc comment; `/***` and plain `/*` do not (RFC 145)
                if s.starts_with("/**")
                    && !s.starts_with("/***")
                    && s.ends_with("*/")
                    && s.len() >= 5
                {
                    return Some(dedent(&s[3..s.len() - 2]));
                }
                return None;
            }
            _ => return None,
        }
    }
    None
}

pub fn from_file(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let doc = leading_doc(&text)?;
    if doc.is_empty() {
        None
    } else {
        Some(doc)
    }
}

fn dedent(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let min_indent = lines
        .iter()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut out: Vec<String> = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if i == 0 {
            out.push(l.trim_start().to_string());
        } else {
            out.push(l.get(min_indent..).unwrap_or("").to_string());
        }
    }
    out.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leading_doc_extraction() {
        let doc = leading_doc("/**\n  NAS host.\n\n  Runs *storage*.\n*/\n{ }\n");
        assert_eq!(doc.as_deref(), Some("NAS host.\n\nRuns *storage*."));
        assert_eq!(leading_doc("/** hi */\n{ }").as_deref(), Some("hi"));
        assert_eq!(leading_doc("/* not a doc */\n{ }"), None);
        assert_eq!(leading_doc("# header\n{ }"), None);
        assert_eq!(leading_doc("{ a = 1; }\n/** late */"), None);
    }
}
