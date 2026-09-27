pub struct Book {
    pub default: &'static str,
    pub dark: &'static str,
}

pub const D2_LAYOUT: [&str; 2] = ["--layout", "elk"];
pub const D2_DARK: [&str; 2] = ["--theme", "200"];
pub const DOT: [&str; 1] = ["-Tsvg"];
pub const DOT_FONT: &str = "sans-serif";
pub const BOOK_DARK: Book = Book {
    default: "navy",
    dark: "navy",
};
pub const BOOK_LIGHT: Book = Book {
    default: "light",
    dark: "coal",
};
