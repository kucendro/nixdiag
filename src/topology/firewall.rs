use super::{Connection, Endpoint, Exposure, Network};
use crate::facts::{Facts, Firewall, NixosHost, Rules, Scope};
use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq)]
pub enum Finding {
    Unused {
        port: u32,
        udp: bool,
        on: Option<String>,
    },
    Closed {
        unit: Option<String>,
        port: u32,
        udp: bool,
        scope: Scope,
        on: Vec<String>,
    },
    Trusted(String),
    Blocked {
        from: String,
        port: u32,
        on: Vec<String>,
    },
}

impl Rules {
    fn opens(&self, port: u32, udp: bool) -> bool {
        let (single, ranges) = if udp {
            (&self.udp, &self.udp_ranges)
        } else {
            (&self.tcp, &self.tcp_ranges)
        };
        single.contains(&port) || ranges.iter().any(|(a, b)| (*a..=*b).contains(&port))
    }
}

impl Firewall {
    fn opens(&self, interface: &str, port: u32, udp: bool) -> bool {
        !self.enable
            || self.trusted.iter().any(|t| t == interface)
            || self.rules.opens(port, udp)
            || self
                .interfaces
                .get(interface)
                .is_some_and(|r| r.opens(port, udp))
    }

    fn closed(&self, on: Vec<String>, port: u32, udp: bool) -> Option<Vec<String>> {
        let open = on.iter().any(|i| self.opens(i, port, udp));
        (!on.is_empty() && !open).then_some(on)
    }
}

pub(super) fn reach(
    facts: &Facts,
    nets: &[Network],
    host: &str,
    port: u32,
    udp: bool,
) -> Vec<String> {
    let Some(n) = facts.hosts.get(host).and_then(|h| h.as_nixos()) else {
        return Vec::new();
    };
    let fw = &n.network.firewall;
    let open = |net: &&Network| {
        let on = net.members.iter().filter(|m| m.host == host);
        on.into_iter().any(|m| fw.opens(&m.interface, port, udp))
    };
    nets.iter().filter(open).map(Network::id).collect()
}

pub(super) struct Audit<'a> {
    pub facts: &'a Facts,
    pub nets: &'a [Network],
    pub exposed: &'a [Exposure],
    pub connections: &'a [Connection],
}

impl Audit<'_> {
    pub(super) fn of(&self, host: &str) -> Vec<Finding> {
        let Some(n) = self.facts.hosts.get(host).and_then(|h| h.as_nixos()) else {
            return Vec::new();
        };
        let fw = &n.network.firewall;
        if !fw.enable {
            return Vec::new();
        }
        let mut out = self.unused(host, fw);
        out.extend(self.closed(host, n));
        out.extend(
            fw.trusted
                .iter()
                .filter(|i| *i != "lo")
                .cloned()
                .map(Finding::Trusted),
        );
        out.extend(self.blocked(host, fw));
        out
    }

    fn used(&self, host: &str) -> BTreeSet<(u32, bool)> {
        let units = self.facts.hosts[host].topology().units.values();
        let ports = units.flat_map(|u| u.ports.iter().map(|p| (*p, false)));
        let exposed = self.exposed.iter().filter(|x| x.host == host);
        ports.chain(exposed.map(|x| (x.port, x.udp))).collect()
    }

    fn unused(&self, host: &str, fw: &Firewall) -> Vec<Finding> {
        let used = self.used(host);
        let rules = std::iter::once((None, &fw.rules));
        let own = fw.interfaces.iter().map(|(i, r)| (Some(i), r));
        let singles = rules.chain(own).flat_map(|(on, r)| {
            let tcp = r.tcp.iter().map(move |p| (on, *p, false));
            tcp.chain(r.udp.iter().map(move |p| (on, *p, true)))
        });
        singles
            .filter(|(_, port, udp)| !used.contains(&(*port, *udp)))
            .map(|(on, port, udp)| Finding::Unused {
                port,
                udp,
                on: on.cloned(),
            })
            .collect()
    }

    fn attached(&self, host: &str, keep: impl Fn(&Network) -> bool) -> Vec<String> {
        let members = self
            .nets
            .iter()
            .filter(|n| keep(n))
            .flat_map(|n| &n.members);
        let on: BTreeSet<&str> = members
            .filter(|m| m.host == host)
            .map(|m| m.interface.as_str())
            .collect();
        on.into_iter().map(String::from).collect()
    }

    fn closed(&self, host: &str, n: &NixosHost) -> Vec<Finding> {
        let fw = &n.network.firewall;
        let all: Vec<String> = n.network.interfaces.keys().cloned().collect();
        let exposed = self.exposed.iter().filter(|x| x.host == host);
        exposed
            .filter_map(|x| {
                let scope = x.scope?;
                let mut on = self.attached(host, |net| net.kind == Some(scope));
                if on.is_empty() {
                    on.clone_from(&all);
                }
                Some(Finding::Closed {
                    unit: x.unit.clone(),
                    port: x.port,
                    udp: x.udp,
                    scope,
                    on: fw.closed(on, x.port, x.udp)?,
                })
            })
            .collect()
    }

    fn blocked(&self, host: &str, fw: &Firewall) -> Vec<Finding> {
        let into = self.connections.iter().filter_map(|c| {
            let (Endpoint::Unit(a, u), Endpoint::Unit(b, _)) = (&c.from, &c.to) else {
                return None;
            };
            (b == host && a != host).then_some((a, u, c.port?))
        });
        into.filter_map(|(a, u, port)| {
            let shared = |net: &Network| net.members.iter().any(|m| &m.host == a);
            let on = fw.closed(self.attached(host, shared), port, false)?;
            Some(Finding::Blocked {
                from: format!("{a}/{u}"),
                port,
                on,
            })
        })
        .collect()
    }
}

#[cfg(test)]
mod tests;
