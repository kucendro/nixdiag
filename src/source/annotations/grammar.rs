use anyhow::{bail, Result};
use std::borrow::Cow;

macro_rules! def_grammar {
    ($n:literal) => {
        pub const GRAMMAR: u32 = $n;
        pub const VERSION: &str = concat!(
            env!("CARGO_PKG_VERSION"),
            "\nannotation grammar ",
            stringify!($n)
        );
    };
}
def_grammar!(1);

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

pub(super) struct Deprecation {
    pub(super) old: &'static str,
    pub(super) since: &'static str,
    pub(super) new: &'static str,
    pub(super) removed_in: u32,
}

pub(super) const DEPRECATIONS: &[Deprecation] = &[];

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
        let (body, warn) = canonicalize("scope mesh", 1, FAKE).unwrap();
        assert_eq!(body, "scope mesh");
        assert!(warn.is_none());

        let (body, warn) = canonicalize("tailnet public", 1, FAKE).unwrap();
        assert_eq!(body, "mesh public");
        let w = warn.unwrap();
        assert!(w.contains("0.5") && w.contains("#: mesh"), "{w}");

        let err = canonicalize("tailnet public", 2, FAKE).unwrap_err();
        assert!(
            err.contains("#: mesh") && err.contains("nixdiag migrate --to 2"),
            "{err}"
        );

        assert!(canonicalize("tailnet", GRAMMAR, DEPRECATIONS)
            .unwrap()
            .1
            .is_none());
    }
}
