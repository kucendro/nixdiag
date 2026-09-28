use std::fmt::{self, Display, Write};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    Root,
    Node,
    Flagged,
    Arrow,
    Follows,
    Host,
    Table,
    Machine,
    Infra,
    App,
    Ghost,
    NetPublic,
    NetLan,
    NetMesh,
    Public,
    Lan,
    Mesh,
    Flow,
    Local,
    Control,
    Mgmt,
    Route,
    Location,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Root => "root",
            Class::Node => "node",
            Class::Flagged => "flagged",
            Class::Arrow => "arrow",
            Class::Follows => "follows",
            Class::Host => "host",
            Class::Table => "table",
            Class::Machine => "machine",
            Class::Infra => "infra",
            Class::App => "app",
            Class::Ghost => "ghost",
            Class::NetPublic => "net-public",
            Class::NetLan => "net-lan",
            Class::NetMesh => "net-mesh",
            Class::Public => "public",
            Class::Lan => "lan",
            Class::Mesh => "mesh",
            Class::Flow => "flow",
            Class::Local => "local",
            Class::Control => "control",
            Class::Mgmt => "mgmt",
            Class::Route => "route",
            Class::Location => "location",
        }
    }
}

struct Row {
    key: String,
    kind: String,
    constraint: Option<String>,
}

pub struct Shape {
    key: String,
    label: String,
    class: Class,
    link: Option<String>,
    tooltip: Option<String>,
    rows: Vec<Row>,
    children: Vec<Shape>,
}

impl Shape {
    fn new(key: &str, label: &str, class: Class) -> Self {
        Shape {
            key: key.into(),
            label: label.into(),
            class,
            link: None,
            tooltip: None,
            rows: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn row(&mut self, key: &str, kind: &str, constraint: Option<&str>) -> &mut Self {
        self.rows.push(Row {
            key: key.into(),
            kind: kind.into(),
            constraint: constraint.map(Into::into),
        });
        self
    }

    pub fn child(&mut self, key: &str, label: &str, class: Class) -> &mut Shape {
        self.children.push(Shape::new(key, label, class));
        self.children.last_mut().expect("just pushed")
    }

    pub fn link(&mut self, url: &str) -> &mut Self {
        self.link = Some(url.into());
        self
    }

    pub fn tooltip(&mut self, text: &str) -> &mut Self {
        self.tooltip = Some(text.into());
        self
    }

    fn write(&self, f: &mut fmt::Formatter, pad: &str) -> fmt::Result {
        let (key, label, class) = (Quoted(&self.key), Quoted(&self.label), self.class.name());
        let plain = self.link.is_none()
            && self.tooltip.is_none()
            && self.rows.is_empty()
            && self.children.is_empty();
        if plain {
            return writeln!(f, "{pad}{key}: {label} {{class: {class}}}");
        }
        writeln!(f, "{pad}{key}: {label} {{\n{pad}  class: {class}")?;
        if let Some(l) = &self.link {
            writeln!(f, "{pad}  link: {}", Quoted(l))?;
        }
        if let Some(t) = &self.tooltip {
            writeln!(f, "{pad}  tooltip: {}", Quoted(t))?;
        }
        for r in &self.rows {
            write!(f, "{pad}  {}: {}", Quoted(&r.key), Quoted(&r.kind))?;
            match &r.constraint {
                Some(c) => writeln!(f, " {{constraint: {}}}", Quoted(c))?,
                None => writeln!(f)?,
            }
        }
        let inner = format!("{pad}  ");
        for c in &self.children {
            c.write(f, &inner)?;
        }
        writeln!(f, "{pad}}}")
    }
}

struct Edge {
    from: Vec<String>,
    to: Vec<String>,
    directed: bool,
    label: Option<String>,
    class: Class,
}

#[derive(Default)]
pub struct Doc {
    vertical: bool,
    shapes: Vec<Shape>,
    edges: Vec<Edge>,
}

impl Doc {
    pub fn vertical(&mut self) {
        self.vertical = true;
    }

    pub fn shape(&mut self, key: &str, label: &str, class: Class) -> &mut Shape {
        self.shapes.push(Shape::new(key, label, class));
        self.shapes.last_mut().expect("just pushed")
    }

    pub fn place(
        &mut self,
        parent: Option<&str>,
        key: &str,
        label: &str,
        class: Class,
    ) -> &mut Shape {
        match parent.and_then(|p| self.shapes.iter().position(|s| s.key == p)) {
            Some(i) => self.shapes[i].child(key, label, class),
            None => self.shape(key, label, class),
        }
    }

    pub fn edge(
        &mut self,
        from: &[impl AsRef<str>],
        to: &[impl AsRef<str>],
        label: Option<&str>,
        class: Class,
    ) {
        self.push(path(from), path(to), true, label, class);
    }

    pub fn line(
        &mut self,
        from: &[impl AsRef<str>],
        to: &[impl AsRef<str>],
        label: Option<&str>,
        class: Class,
    ) {
        self.push(path(from), path(to), false, label, class);
    }

    fn push(
        &mut self,
        from: Vec<String>,
        to: Vec<String>,
        directed: bool,
        label: Option<&str>,
        class: Class,
    ) {
        self.edges.push(Edge {
            from,
            to,
            directed,
            label: label.map(Into::into),
            class,
        });
    }
}

fn path(p: &[impl AsRef<str>]) -> Vec<String> {
    p.iter().map(|s| s.as_ref().to_string()).collect()
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

struct Path<'a>(&'a [String]);

impl Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, seg) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_char('.')?;
            }
            write!(f, "{}", Quoted(seg))?;
        }
        Ok(())
    }
}

impl Display for Doc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.vertical {
            writeln!(f, "direction: down")?;
        }
        for s in &self.shapes {
            s.write(f, "")?;
        }
        for e in &self.edges {
            let arrow = if e.directed { "->" } else { "--" };
            write!(f, "{} {arrow} {}", Path(&e.from), Path(&e.to))?;
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
        d.edge(&["a.b"], &["style"], Some("x"), Class::Arrow);
        d.edge(&["style"], &["a.b"], None, Class::Arrow);
        assert_eq!(
            d.to_string(),
            "\"a.b\": \"say \\\"hi\\\"\\n\\\\o/\" {class: node}\n\
             \"a.b\" -> \"style\": \"x\" {class: arrow}\n\
             \"style\" -> \"a.b\" {class: arrow}\n"
        );
    }

    #[test]
    fn blocks_nest_rows_children_links_and_paths() {
        let mut d = Doc::default();
        d.vertical();
        d.shape("tom", "tom", Class::Table)
            .link("./hosts.html#host-tom")
            .row("grafana", "3000/tcp", Some("monitor"))
            .row("exporter", "—", None);
        d.shape("jerry", "jerry", Class::Machine)
            .child("nginx", "nginx", Class::Infra)
            .tooltip("proxy");
        d.edge(&["jerry", "nginx"], &["tom", "grafana"], None, Class::Flow);
        d.line(&["tom"], &["lan"], Some("eth0"), Class::Lan);
        assert_eq!(
            d.to_string(),
            "direction: down\n\"tom\": \"tom\" {\n  class: table\n  link: \"./hosts.html#host-tom\"\n  \
             \"grafana\": \"3000/tcp\" {constraint: \"monitor\"}\n  \"exporter\": \"—\"\n}\n\
             \"jerry\": \"jerry\" {\n  class: machine\n  \
             \"nginx\": \"nginx\" {\n    class: infra\n    tooltip: \"proxy\"\n  }\n}\n\
             \"jerry\".\"nginx\" -> \"tom\".\"grafana\" {class: flow}\n\
             \"tom\" -- \"lan\": \"eth0\" {class: lan}\n"
        );
    }

    #[test]
    fn theme_defines_every_class() {
        let theme = include_str!("theme.d2");
        let all = [
            Class::Root,
            Class::Node,
            Class::Flagged,
            Class::Arrow,
            Class::Follows,
            Class::Host,
            Class::Table,
            Class::Machine,
            Class::Infra,
            Class::App,
            Class::Ghost,
            Class::NetPublic,
            Class::NetLan,
            Class::NetMesh,
            Class::Public,
            Class::Lan,
            Class::Mesh,
            Class::Flow,
            Class::Local,
            Class::Control,
            Class::Mgmt,
            Class::Route,
            Class::Location,
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
