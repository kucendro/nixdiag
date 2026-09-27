#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub name: &'static str,
    pub light: &'static str,
    pub dark: &'static str,
}

const fn c(name: &'static str, light: &'static str, dark: &'static str) -> Color {
    Color { name, light, dark }
}

pub mod diagram {
    use super::{c, Color};

    pub const APP_FILL: Color = c("appFill", "#e6f0ff", "#1c2e4a");
    pub const APP_STROKE: Color = c("appStroke", "#4a76c4", "#7fa7e8");
    pub const INFRA_FILL: Color = c("infraFill", "#ffe9cc", "#4a3413");
    pub const INFRA_STROKE: Color = c("infraStroke", "#c47a29", "#d9995a");
    pub const BASE_FILL: Color = c("baseFill", "#f0f0f0", "#2a2a2e");
    pub const BASE_STROKE: Color = c("baseStroke", "#999", "#666");
    pub const HOST_FILL: Color = c("hostFill", "#fbfbfe", "#181825");
    pub const HOST_STROKE: Color = c("hostStroke", "#333", "#a6adc8");
    pub const PROG_FILL: Color = c("progFill", "#eaffea", "#1e3a24");
    pub const HOST_CLOUD: Color = c("hostCloud", "#fff3cd", "#3d3517");
    pub const PUBLIC: Color = c("public", "#c0392b", "#e74c3c");
    pub const LAN: Color = c("lan", "#27893f", "#2ecc71");
    pub const MESH: Color = c("mesh", "#4a76c4", "#7fa7e8");
    pub const INK: Color = c("ink", "#1f2328", "#cdd6f4");
    pub const LINE: Color = c("line", "#57606a", "#a6adc8");
    pub const ALL: &[Color] = &[
        APP_FILL,
        APP_STROKE,
        INFRA_FILL,
        INFRA_STROKE,
        BASE_FILL,
        BASE_STROKE,
        HOST_FILL,
        HOST_STROKE,
        PROG_FILL,
        HOST_CLOUD,
        PUBLIC,
        LAN,
        MESH,
        INK,
        LINE,
    ];
}

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
