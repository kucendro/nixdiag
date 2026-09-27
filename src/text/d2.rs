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
