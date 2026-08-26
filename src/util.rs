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
