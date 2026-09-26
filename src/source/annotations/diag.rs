#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sev {
    Error,
    Deprecated,
}

pub struct Diag {
    pub file: String,
    pub line: usize,
    pub msg: String,
    pub sev: Sev,
}

impl Diag {
    pub fn error(file: &str, line: usize, msg: impl Into<String>) -> Self {
        Diag {
            file: file.to_string(),
            line,
            msg: msg.into(),
            sev: Sev::Error,
        }
    }

    pub fn deprecated(file: &str, line: usize, msg: impl Into<String>) -> Self {
        Diag {
            file: file.to_string(),
            line,
            msg: msg.into(),
            sev: Sev::Deprecated,
        }
    }
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.msg)
    }
}
