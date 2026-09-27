use itertools::Itertools;

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

pub fn store_name(path: &str) -> &str {
    let base = path.rsplit('/').next().unwrap_or(path);
    base.split_once('-').map(|(_, name)| name).unwrap_or(base)
}

pub fn package_name(name: &str) -> &str {
    let version = |(a, b): (&u8, &u8)| *a == b'-' && b.is_ascii_digit();
    let at = name.as_bytes().iter().tuple_windows().position(version);
    at.map_or(name, |i| &name[..i])
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(store_name("plain"), "plain");
        assert_eq!(store_name("/some/where/else"), "else");
    }

    #[test]
    fn package_names_drop_the_version() {
        assert_eq!(package_name("linux-6.12.9"), "linux");
        assert_eq!(package_name("glibc-2.42-67"), "glibc");
        assert_eq!(package_name("bash-5.2p37"), "bash");
        assert_eq!(package_name("gcc-13.2.0-lib"), "gcc");
        assert_eq!(
            package_name("python3.12-requests-2.32.3"),
            "python3.12-requests"
        );
        assert_eq!(package_name("playwright-chromium"), "playwright-chromium");
        assert_eq!(package_name(""), "");
        assert_eq!(package_name("-"), "-");
    }
}
