pub mod d2;
pub mod wiki;

pub fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_string(), |s, (k, v)| {
        s.replace(&format!("{{{k}}}"), v)
    })
}

#[cfg(test)]
mod tests {
    use super::fill;

    #[test]
    fn slots_are_filled_by_name_and_unknown_ones_stay() {
        assert_eq!(fill("a {x} {y}", &[("x", "1")]), "a 1 {y}");
    }
}
