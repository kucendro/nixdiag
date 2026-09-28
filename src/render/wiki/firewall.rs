use super::{codes, table, Page, Wiki};
use crate::conf::files::page;
use crate::facts::Firewall;
use crate::text::wiki::{code, firewall as t, NONE};
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
        let mut o = Vec::new();
        for (host, h) in &w.facts.hosts {
            let Some(n) = h.as_nixos() else { continue };
            let fw = &n.network.firewall;
            o.push(t::host(host, &page::wall(host)));
            o.push(if fw.enable {
                table(t::HEAD, rows(fw))
            } else {
                t::OFF.into()
            });
        }
        Ok((!o.is_empty()).then_some(o))
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
