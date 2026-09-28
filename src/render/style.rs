use crate::conf::palette::Color;
use crate::conf::tools::{Book, BOOK_DARK, BOOK_LIGHT, CHART_DARK, CHART_LIGHT, D2_DARK, D2_LIGHT};
use clap::ValueEnum;

#[derive(Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub const ALL: [Theme; 2] = [Theme::Dark, Theme::Light];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }

    pub fn d2(self) -> [&'static str; 2] {
        match self {
            Theme::Dark => D2_DARK,
            Theme::Light => D2_LIGHT,
        }
    }

    pub fn chart(self) -> &'static str {
        match self {
            Theme::Dark => CHART_DARK,
            Theme::Light => CHART_LIGHT,
        }
    }

    pub fn book(self) -> Book {
        match self {
            Theme::Dark => BOOK_DARK,
            Theme::Light => BOOK_LIGHT,
        }
    }
}

#[derive(Default)]
pub struct Style {
    pub theme: Theme,
    pub background: Option<String>,
    pub colors: Vec<(String, String)>,
}

impl Style {
    pub fn color(&self, c: &Color) -> &str {
        self.paint(self.theme, c)
    }

    pub fn paint(&self, theme: Theme, c: &Color) -> &str {
        match self.colors.iter().rev().find(|(n, _)| n == c.name) {
            Some((_, v)) => v,
            None => match theme {
                Theme::Dark => c.dark,
                Theme::Light => c.light,
            },
        }
    }
}
