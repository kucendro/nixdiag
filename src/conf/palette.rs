#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub name: &'static str,
    pub light: &'static str,
    pub dark: &'static str,
}

const fn c(name: &'static str, light: &'static str, dark: &'static str) -> Color {
    Color { name, light, dark }
}

pub const DIAGRAM: &[Color] = &[
    c("appFill", "#e6f0ff", "#1c2e4a"),
    c("appStroke", "#4a76c4", "#7fa7e8"),
    c("infraFill", "#ffe9cc", "#4a3413"),
    c("infraStroke", "#c47a29", "#d9995a"),
    c("baseFill", "#f0f0f0", "#2a2a2e"),
    c("baseStroke", "#999", "#666"),
    c("hostFill", "#fbfbfe", "#181825"),
    c("hostStroke", "#333", "#a6adc8"),
    c("progFill", "#eaffea", "#1e3a24"),
    c("hostCloud", "#fff3cd", "#3d3517"),
    c("public", "#c0392b", "#e74c3c"),
    c("lan", "#27893f", "#2ecc71"),
    c("mesh", "#4a76c4", "#7fa7e8"),
];

pub mod chart {
    use super::{c, Color};

    pub const SHARED: Color = c("chartShared", "#4a76c4", "#7fa7e8");
    pub const PARTIAL: Color = c("chartPartial", "#c47a29", "#d9995a");
    pub const UNIQUE: Color = c("chartUnique", "#27893f", "#2ecc71");
    pub const MARK: Color = c("chartMark", "#4a76c4", "#7fa7e8");
    pub const INK: Color = c("chartInk", "#333333", "#c9d1d9");
    pub const MUTED: Color = c("chartMuted", "#777777", "#8b949e");
    pub const TRACK: Color = c("chartTrack", "#ebebeb", "#2a2a2e");
    pub const TILE_INK: Color = c("chartTileInk", "#ffffff", "#14181f");
    pub const ALL: &[Color] = &[SHARED, PARTIAL, UNIQUE, MARK, INK, MUTED, TRACK, TILE_INK];
}
