use std::fmt::{self, Display, Write};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    Root,
    Node,
    Flagged,
    Link,
    Follows,
    Host,
    File,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Root => "root",
            Class::Node => "node",
            Class::Flagged => "flagged",
            Class::Link => "link",
            Class::Follows => "follows",
            Class::Host => "host",
            Class::File => "file",
        }
    }
}

struct Shape {
    key: String,
    label: String,
    class: Class,
    rows: Vec<(String, String)>,
}

struct Edge {
    from: String,
    to: String,
    label: Option<String>,
    class: Class,
}

#[derive(Default)]
pub struct Doc {
    shapes: Vec<Shape>,
    edges: Vec<Edge>,
}

impl Doc {
    pub fn shape(&mut self, key: &str, label: &str, class: Class) {
        self.table(key, label, class, []);
    }

    pub fn table<'r>(
        &mut self,
        key: &str,
        label: &str,
        class: Class,
        rows: impl IntoIterator<Item = (&'r str, &'r str)>,
    ) {
        self.shapes.push(Shape {
            key: key.into(),
            label: label.into(),
            class,
            rows: rows
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        });
    }

    pub fn edge(&mut self, from: &str, to: &str, label: Option<&str>, class: Class) {
        self.edges.push(Edge {
            from: from.into(),
            to: to.into(),
            label: label.map(Into::into),
            class,
        });
    }
}

pub fn quote(s: &str) -> String {
    Quoted(s).to_string()
}

struct Quoted<'a>(&'a str);

impl Display for Quoted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char('"')?;
        for c in self.0.chars() {
            match c {
                '"' => f.write_str("\\\"")?,
                '\\' => f.write_str("\\\\")?,
                '\n' => f.write_str("\\n")?,
                c => f.write_char(c)?,
            }
        }
        f.write_char('"')
    }
}

impl Display for Doc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for s in &self.shapes {
            let (key, label, class) = (Quoted(&s.key), Quoted(&s.label), s.class.name());
            if s.rows.is_empty() {
                writeln!(f, "{key}: {label} {{class: {class}}}")?;
                continue;
            }
            writeln!(f, "{key}: {label} {{\n  class: {class}")?;
            for (k, v) in &s.rows {
                writeln!(f, "  {}: {}", Quoted(k), Quoted(v))?;
            }
            writeln!(f, "}}")?;
        }
        for e in &self.edges {
            write!(f, "{} -> {}", Quoted(&e.from), Quoted(&e.to))?;
            if let Some(l) = &e.label {
                write!(f, ": {}", Quoted(l))?;
            }
            writeln!(f, " {{class: {}}}", e.class.name())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Class, Doc};

    #[test]
    fn keys_and_labels_are_quoted_and_escaped() {
        let mut d = Doc::default();
        d.shape("a.b", "say \"hi\"\n\\o/", Class::Node);
        d.edge("a.b", "style", Some("x"), Class::Link);
        d.edge("style", "a.b", None, Class::Link);
        assert_eq!(
            d.to_string(),
            "\"a.b\": \"say \\\"hi\\\"\\n\\\\o/\" {class: node}\n\
             \"a.b\" -> \"style\": \"x\" {class: link}\n\
             \"style\" -> \"a.b\" {class: link}\n"
        );
    }

    #[test]
    fn rows_nest_inside_their_table() {
        let mut d = Doc::default();
        d.table("m/a.nix", "m/a.nix", Class::File, [("nginx", "service")]);
        assert_eq!(
            d.to_string(),
            "\"m/a.nix\": \"m/a.nix\" {\n  class: file\n  \"nginx\": \"service\"\n}\n"
        );
    }

    #[test]
    fn theme_defines_every_class() {
        let theme = include_str!("theme.d2");
        let all = [
            Class::Root,
            Class::Node,
            Class::Flagged,
            Class::Link,
            Class::Follows,
            Class::Host,
            Class::File,
        ];
        for c in all {
            assert!(
                theme.contains(&format!("  {}: {{", c.name())),
                "{}",
                c.name()
            );
        }
    }
}
