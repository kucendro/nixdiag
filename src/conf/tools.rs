pub struct Book {
    pub default: &'static str,
    pub dark: &'static str,
}

pub const DOT: [&str; 1] = ["-Tsvg"];
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
