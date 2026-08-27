//! Grammar editions and the deprecation lifecycle.
//!
//! The annotation grammar is the one nixdiag surface that outlives nixdiag
//! versions — it lives in other people's module files — so it is versioned
//! as an edition, declared by the consumer and enforced here.

use anyhow::{bail, Result};
use std::borrow::Cow;

/// The annotation grammar is the one surface that outlives nixdiag versions:
/// it lives in other people's module files. Editions are declared by the
/// consumer (`nixdiag.grammar` / `mkDocs.grammar` / `--grammar`); removals
/// happen at an edition bump and nowhere else.
macro_rules! def_grammar {
    ($n:literal) => {
        /// Annotation grammar edition this binary implements.
        pub const GRAMMAR: u32 = $n;
        /// `nixdiag --version` — bug reports must carry the edition.
        pub const VERSION: &str = concat!(
            env!("CARGO_PKG_VERSION"),
            "\nannotation grammar ",
            stringify!($n)
        );
    };
}
def_grammar!(1);

/// The edition in force. Unset means "whatever this binary implements", so
/// zero-config stays zero-config. A declaration newer than we implement is
/// fatal — better than guessing at an unknown statement.
pub fn resolve_edition(declared: Option<u32>) -> Result<u32> {
    match declared {
        None => Ok(GRAMMAR),
        Some(d) if d > GRAMMAR => bail!(
            "the flake declares annotation grammar {d}, but nixdiag {} implements \
             grammar {GRAMMAR}; upgrade nixdiag, or lower the declared grammar to {GRAMMAR}",
            env!("CARGO_PKG_VERSION")
        ),
        Some(d) => Ok(d),
    }
}

/// A statement verb retired in favour of another spelling. The old spelling
/// keeps working — with a warning — until the edition named by `removed_in`.
pub(super) struct Deprecation {
    pub(super) old: &'static str,
    pub(super) since: &'static str,
    pub(super) new: &'static str,
    pub(super) removed_in: u32,
}

/// Nothing is deprecated in grammar 1. Entries land here when a replacement
/// spelling ships, and are only ever dropped at an edition bump.
pub(super) const DEPRECATIONS: &[Deprecation] = &[];

/// Rewrite a deprecated verb into its current spelling before parsing.
/// `Err` when the edition in force has already removed it.
pub(super) fn canonicalize<'a>(
    body: &'a str,
    edition: u32,
    table: &[Deprecation],
) -> Result<(Cow<'a, str>, Option<String>), String> {
    let verb = body.split_whitespace().next().unwrap_or("");
    let Some(d) = table.iter().find(|d| d.old == verb) else {
        return Ok((Cow::Borrowed(body), None));
    };
    if edition >= d.removed_in {
        return Err(format!(
            "`#: {}` was removed in grammar {}; use `#: {}` (rewrite with \
             `nixdiag migrate --to {}`)",
            d.old, d.removed_in, d.new, d.removed_in
        ));
    }
    let rest = body.split_once(verb).map(|(_, r)| r).unwrap_or("");
    Ok((
        Cow::Owned(format!("{}{rest}", d.new)),
        Some(format!(
            "`#: {}` deprecated since {}, use `#: {}`",
            d.old, d.since, d.new
        )),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real table is empty in grammar 1, so the mechanism is driven with
    /// a synthetic entry: retired in 0.5, gone at the next edition.
    const FAKE: &[Deprecation] = &[Deprecation {
        old: "tailnet",
        since: "0.5",
        new: "mesh",
        removed_in: 2,
    }];

    #[test]
    fn grammar_edition_skew() {
        assert_eq!(resolve_edition(None).unwrap(), GRAMMAR);
        assert_eq!(resolve_edition(Some(GRAMMAR)).unwrap(), GRAMMAR);
        // Older declaration: compat mode, the edition in force is theirs.
        assert_eq!(resolve_edition(Some(GRAMMAR - 1)).unwrap(), GRAMMAR - 1);
        let err = resolve_edition(Some(GRAMMAR + 1)).unwrap_err().to_string();
        assert!(
            err.contains(&(GRAMMAR + 1).to_string()) && err.contains(&GRAMMAR.to_string()),
            "{err}"
        );
        assert!(VERSION.contains("annotation grammar 1"), "{VERSION}");
    }

    #[test]
    fn deprecated_verb_is_rewritten_then_removed() {
        // Untouched verbs pass through borrowed, with no warning.
        let (body, warn) = canonicalize("scope mesh", 1, FAKE).unwrap();
        assert_eq!(body, "scope mesh");
        assert!(warn.is_none());

        // Inside its window: rewritten to the current spelling, plus a warning
        // naming the version it was retired in and the replacement.
        let (body, warn) = canonicalize("tailnet public", 1, FAKE).unwrap();
        assert_eq!(body, "mesh public");
        let w = warn.unwrap();
        assert!(w.contains("0.5") && w.contains("#: mesh"), "{w}");

        // At the edition that removes it: a hard error naming the migration.
        let err = canonicalize("tailnet public", 2, FAKE).unwrap_err();
        assert!(
            err.contains("#: mesh") && err.contains("nixdiag migrate --to 2"),
            "{err}"
        );

        // Grammar 1 deprecates nothing.
        assert!(canonicalize("tailnet", GRAMMAR, DEPRECATIONS)
            .unwrap()
            .1
            .is_none());
    }
}
