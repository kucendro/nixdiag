use crate::conf::palette::Color;
use crate::conf::tools::{Book, BOOK_DARK, BOOK_LIGHT, CHART_DARK, CHART_LIGHT};
use clap::ValueEnum;

#[derive(Clone, Copy, Default, ValueEnum)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
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
        match self.colors.iter().rev().find(|(n, _)| n == c.name) {
            Some((_, v)) => v,
            None => match self.theme {
                Theme::Dark => c.dark,
                Theme::Light => c.light,
            },
        }
    }
}
