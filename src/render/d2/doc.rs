use std::fmt::{self, Display, Write};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    Root,
    Node,
    Flagged,
    Link,
    Follows,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Root => "root",
            Class::Node => "node",
            Class::Flagged => "flagged",
            Class::Link => "link",
            Class::Follows => "follows",
        }
    }
}

struct Shape {
    key: String,
    label: String,
    class: Class,
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
        self.shapes.push(Shape {
            key: key.into(),
            label: label.into(),
            class,
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
            let (key, label) = (Quoted(&s.key), Quoted(&s.label));
            writeln!(f, "{key}: {label} {{class: {}}}", s.class.name())?;
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
    fn theme_defines_every_class() {
        let theme = include_str!("theme.d2");
        let all = [
            Class::Root,
            Class::Node,
            Class::Flagged,
            Class::Link,
            Class::Follows,
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
