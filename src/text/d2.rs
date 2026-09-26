pub const HEADER: &str = "# Auto-generated from the Nix config by nixdiag. Do not edit.";
pub const DIRECTION: &str = "direction: right";

pub mod topology {
    pub const CLASSES: &str = "\
classes: {
  app: { style: { fill: ${appFill}; stroke: ${appStroke} } }
  infra: { style: { fill: ${infraFill}; stroke: ${infraStroke} } }
  base: { style: { fill: ${baseFill}; stroke: ${baseStroke}; font-size: 13 } }
}";
    pub const INTERNET: &str =
        "{id}: \"🌐 Internet\" { shape: cloud; style: { fill: ${hostFill}; stroke: ${public} } }";
    pub const LAN: &str =
        "{id}: \"🏠 LAN\" { shape: cloud; style: { fill: ${hostFill}; stroke: ${lan} } }";
    pub const NIXOS_ICON: &str = "🖥️";
    pub const DARWIN_ICON: &str = "🍏";
    pub const HOST_OPEN: &str =
        "{id}: \"{icon} {host}\" {\n  style: { fill: ${hostFill}; stroke: ${hostStroke}; bold: true }";
    pub const HOST_CLOSE: &str = "}";
    pub const UNIT: &str = "  {id}: \"{label}\" { class: {class} }";
    pub const UNIT_WITH_ROLE: &str = "{unit}\\n({role})";
    pub const PORTS: &str = "  ports: \"{ports}\" { class: base }";
    pub const TCP: &str = "tcp {ports}";
    pub const UDP: &str = "udp {ports}";
    pub const TCP_AND_UDP: &str = "{tcp} · {udp}";
    pub const BASE: &str = "  base: \"+ {count} system services\" { class: base }";
    pub const CONNECTIONS: &str = "# connections";
    pub const CONNECTION: &str = "{from} -> {to}: \"{label}\" { style.stroke: {color} }";
    pub const EXPOSE: &str = ":{port}{proto}";
    pub const EXPOSE_NAMED: &str = "{name} :{port}{proto}";
    pub const UDP_SUFFIX: &str = "/udp";
}

pub mod modules {
    pub const HOST: &str =
        "{id}: \"{host}\" { shape: cloud; style.fill: ${hostCloud}; style.bold: true }";
    pub const DIR_OPEN: &str = "{pad}{id}: \"{name}\" {";
    pub const CLOSE: &str = "{pad}}";
    pub const FILE: &str = "{pad}{id}: \"{name}\" { shape: page }";
    pub const FILE_OPEN: &str = "{pad}{id}: \"{name}\" { shape: page";
    pub const SERVICE: &str = "{pad}  svc_{id}: \"{name}\" { shape: oval; style.fill: ${appFill} }";
    pub const PROGRAM: &str =
        "{pad}  prog_{id}: \"{name}\" { shape: hexagon; style.fill: ${progFill} }";
    pub const HOST_EDGES: &str = "# host -> entry module";
    pub const IMPORT_EDGES: &str = "# module imports";
    pub const EDGE: &str = "{from} -> {to}";
}

pub mod inputs {
    pub const ROOT: &str = "{id}: \"this flake\" { style.fill: ${hostCloud}; style.bold: true }";
    pub const INPUT: &str = "{id}: \"{label}\" { style.fill: ${baseFill}; {stroke} }";
    pub const FLAGGED_LABEL: &str = "{name} {rev}";
    pub const FLAGGED_STROKE: &str = "style.stroke: ${public}; style.stroke-width: 2";
    pub const STROKE: &str = "style.stroke: ${baseStroke}";
    pub const EDGE: &str = "{from} -> {to}{label}";
    pub const EDGE_LABEL: &str = ": \"{input}\"";
    pub const FOLLOWS: &str =
        "{from} -> {to}{label} { style.stroke: ${mesh}; style.stroke-dash: 3 }";
    pub const DIRECT_EDGES: &str = "# direct inputs";
    pub const FOLLOWS_EDGES: &str = "# follows (deduplication)";
}
