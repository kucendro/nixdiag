use super::{codes, table, Page, Wiki};
use crate::conf::files::page;
use crate::facts::Firewall;
use crate::text::wiki::{code, firewall as t, NONE};
use crate::topology::Finding;
use anyhow::Result;
use itertools::Itertools;
use std::iter::once;

pub(super) struct FirewallPage;

impl Page for FirewallPage {
    fn file(&self) -> &'static str {
        page::FIREWALL
    }

    fn title(&self) -> &'static str {
        t::TITLE
    }

    fn body(&self, w: &Wiki) -> Result<Option<Vec<String>>> {
        let none = Vec::new();
        let mut hosts: Vec<_> = w
            .facts
            .hosts
            .iter()
            .filter_map(|(host, h)| {
                let found = w.model.findings.get(host).unwrap_or(&none);
                Some((host, &h.as_nixos()?.network.firewall, found))
            })
            .collect();
        hosts.sort_by_key(|(_, _, found)| std::cmp::Reverse(found.len()));
        let mut o = Vec::new();
        for (host, fw, found) in hosts {
            o.push(t::host(host, &page::wall(host)));
            o.push(if fw.enable {
                table(t::HEAD, rows(fw))
            } else {
                t::OFF.into()
            });
            o.push(if found.is_empty() {
                t::CLEAN.into()
            } else {
                format!(
                    "{}\n\n{}",
                    t::FINDINGS,
                    found.iter().map(finding).join("\n")
                )
            });
        }
        Ok((!o.is_empty()).then_some(o))
    }
}

fn finding(f: &Finding) -> String {
    let on = |on: &[String]| codes(on, ", ");
    match f {
        Finding::Unused { port, udp, on } => t::unused(
            &t::port(*port, *udp),
            &on.as_deref().map_or(t::ALL.into(), code),
        ),
        Finding::Closed {
            unit,
            port,
            udp,
            scope,
            on: at,
        } => t::closed(
            &unit.as_deref().map_or(t::HOST.into(), code),
            &t::port(*port, *udp),
            scope.label(),
            &on(at),
        ),
        Finding::Trusted(i) => t::open_all(i),
        Finding::Blocked { from, port, on: at } => {
            t::blocked(from, &t::port(*port, false), &on(at))
        }
    }
}

fn rows(fw: &Firewall) -> Vec<[String; 3]> {
    let all = once((t::ALL.to_string(), &fw.rules));
    let own = fw.interfaces.iter().map(|(i, r)| (code(i), r));
    let open = all.chain(own).map(|(i, r)| {
        [
            i,
            ports(&r.tcp, &r.tcp_ranges),
            ports(&r.udp, &r.udp_ranges),
        ]
    });
    let trusted = fw
        .trusted
        .iter()
        .map(|i| [t::trusted(i), t::EVERY.into(), t::EVERY.into()]);
    open.chain(trusted).collect()
}

fn ports(single: &[u32], ranges: &[(u32, u32)]) -> String {
    let ranges = ranges.iter().map(|(a, b)| t::range(*a, *b));
    let all = single
        .iter()
        .map(u32::to_string)
        .chain(ranges)
        .collect_vec();
    if all.is_empty() {
        NONE.into()
    } else {
        codes(all, ", ")
    }
}
