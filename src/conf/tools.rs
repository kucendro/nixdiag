pub struct Book {
    pub default: &'static str,
    pub dark: &'static str,
}

pub const DOT: [&str; 1] = ["-Tsvg"];
pub const D2: [&str; 6] = [
    "--layout",
    "elk",
    "--pad",
    "24",
    "--omit-version",
    "--no-xml-tag",
];
pub const D2_DARK: [&str; 2] = ["--theme", "200"];
pub const D2_LIGHT: [&str; 2] = ["--theme", "0"];
pub const DOT_FONT: &str = "sans-serif";
pub const CHART_DARK: &str = "dark";
pub const CHART_LIGHT: &str = "light";
pub const BOOK_DARK: Book = Book {
    default: "navy",
    dark: "navy",
};
pub const BOOK_LIGHT: Book = Book {
    default: "light",
    dark: "coal",
};
