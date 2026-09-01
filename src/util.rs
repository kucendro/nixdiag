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

/// `YYYY-MM-DD` from a unix timestamp, UTC.
///
/// Hand-rolled civil-from-days (Howard Hinnant's algorithm). The only date
/// nixdiag formats is `lastModified` out of `flake.lock` — a fixed integer in
/// the file, never a clock read — which is not worth a calendar dependency.
pub fn human_date(unix: i64) -> String {
    let z = unix.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Bytes as GiB/MiB/KiB with one decimal.
pub fn human_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * KIB;
    const GIB: f64 = 1024.0 * MIB;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.1} GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.1} MiB", b / MIB)
    } else {
        format!("{:.1} KiB", b / KIB)
    }
}

/// `/nix/store/<hash>-foo-1.2` -> `foo-1.2`.
///
/// Rendered pages must never print a full store path. Nix scans build outputs
/// for store-path strings and records each as a real reference, so a listing
/// of a system closure would make the docs derivation retain that entire
/// closure — measured at 36 MiB of references from a 367-byte table. The hash
/// is also noise in a size report; the name and version are the signal.
pub fn store_name(path: &str) -> &str {
    let base = path.rsplit('/').next().unwrap_or(path);
    // The hash is base32, which has no '-', so the first one ends it.
    base.split_once('-').map(|(_, name)| name).unwrap_or(base)
}

/// `glibc-2.42-67` -> `glibc`; a name with no version is returned whole.
///
/// nixpkgs spells store names `<name>-<version>` and a version starts with a
/// digit, so the first `-` before a digit ends the name. Folding there is what
/// makes a treemap of a real closure readable: the several outputs and
/// versions of one package become one tile instead of a scatter of slivers.
pub fn package_name(name: &str) -> &str {
    let b = name.as_bytes();
    for i in 0..b.len().saturating_sub(1) {
        if b[i] == b'-' && b[i + 1].is_ascii_digit() {
            return &name[..i];
        }
    }
    name
}

/// Thousands-separated count — closure path tallies run to five figures.
pub fn human_count(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_pick_a_unit_and_one_decimal() {
        assert_eq!(human_size(0), "0.0 KiB");
        assert_eq!(human_size(1024), "1.0 KiB");
        assert_eq!(human_size(1024 * 1024), "1.0 MiB");
        assert_eq!(human_size(3_221_225_472), "3.0 GiB");
        assert_eq!(human_size(1_500_000_000), "1.4 GiB");
    }

    #[test]
    fn store_names_drop_the_hash() {
        assert_eq!(
            store_name("/nix/store/qz7wm2xhbvdn6ct9kf3rj5p8yl0aesgu-linux-6.12.9"),
            "linux-6.12.9"
        );
        assert_eq!(
            store_name("/nix/store/0d8g8n0a11v6f5m2h416ajyxmnkwc3md-glibc-2.42-67"),
            "glibc-2.42-67"
        );
        // Not a store path, or no hash to strip: left alone.
        assert_eq!(store_name("plain"), "plain");
        assert_eq!(store_name("/some/where/else"), "else");
    }

    #[test]
    fn package_names_drop_the_version() {
        assert_eq!(package_name("linux-6.12.9"), "linux");
        assert_eq!(package_name("glibc-2.42-67"), "glibc");
        assert_eq!(package_name("bash-5.2p37"), "bash");
        // Multiple outputs of one package fold together.
        assert_eq!(package_name("gcc-13.2.0-lib"), "gcc");
        // A python module keeps its interpreter prefix: the `-` before `12`
        // is not preceded by a digit-starting segment boundary we want.
        assert_eq!(
            package_name("python3.12-requests-2.32.3"),
            "python3.12-requests"
        );
        // No version at all.
        assert_eq!(package_name("playwright-chromium"), "playwright-chromium");
        assert_eq!(package_name(""), "");
        assert_eq!(package_name("-"), "-");
    }

    #[test]
    fn counts_get_thousands_separators() {
        assert_eq!(human_count(0), "0");
        assert_eq!(human_count(999), "999");
        assert_eq!(human_count(1000), "1,000");
        assert_eq!(human_count(5204), "5,204");
        assert_eq!(human_count(1234567), "1,234,567");
    }

    #[test]
    fn dates_are_civil_and_utc() {
        assert_eq!(human_date(0), "1970-01-01");
        assert_eq!(human_date(1_700_000_000), "2023-11-14");
        // a real flake.lock lastModified
        assert_eq!(human_date(1_787_498_568), "2026-08-23");
    }

    #[test]
    fn dates_before_the_epoch_do_not_panic() {
        assert_eq!(human_date(-1), "1969-12-31");
    }
}
