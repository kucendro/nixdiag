use crate::facts::Scope;
use crate::render::d2::{Class, Doc};
use crate::text::d2::{networks as n, topology as t};
use crate::topology::Network;
use ipnet::IpNet;
use itertools::Itertools;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Net {
    Internet,
    Lan,
    Mesh,
}

impl Net {
    pub fn of(scope: Option<Scope>) -> Option<Net> {
        Some(match scope? {
            Scope::Public => Net::Internet,
            Scope::Lan => Net::Lan,
            Scope::Mesh => Net::Mesh,
        })
    }

    fn key(self) -> &'static str {
        match self {
            Net::Internet => ":internet",
            Net::Lan => ":lan",
            Net::Mesh => ":mesh",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Net::Internet => t::INTERNET,
            Net::Lan => t::LAN,
            Net::Mesh => t::MESH,
        }
    }

    fn edge(self) -> Class {
        match self {
            Net::Internet => Class::Public,
            Net::Lan => Class::Lan,
            Net::Mesh => Class::Mesh,
        }
    }

    fn cloud(self) -> Class {
        match self {
            Net::Internet => Class::NetPublic,
            Net::Lan => Class::NetLan,
            Net::Mesh => Class::NetMesh,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Cloud<'a> {
    Net(Net),
    Network(&'a Network),
}

impl<'a> Cloud<'a> {
    pub fn around(nets: &'a [Network], host: &str, scope: Option<Scope>) -> Vec<Cloud<'a>> {
        let Some(net) = Net::of(scope) else {
            return Vec::new();
        };
        let own: Vec<Cloud> = nets
            .iter()
            .filter(|n| net != Net::Internet && Net::of(n.kind) == Some(net))
            .filter(|n| n.members.iter().any(|m| m.host == host))
            .map(Cloud::Network)
            .collect();
        if own.is_empty() {
            vec![Cloud::Net(net)]
        } else {
            own
        }
    }

    pub fn of(nets: &'a [Network], ids: &[String]) -> Vec<Cloud<'a>> {
        let reached = nets.iter().filter(|n| ids.contains(&n.id()));
        let clouds = reached.map(|n| match n.kind {
            Some(Scope::Public) => Cloud::Net(Net::Internet),
            _ => Cloud::Network(n),
        });
        clouds.unique_by(Cloud::key).collect()
    }

    pub fn route(nets: &'a [Network], prefix: &IpNet) -> Cloud<'a> {
        let within = nets
            .iter()
            .find(|n| prefix.prefix_len() > 0 && n.contains(&prefix.addr()));
        within.map_or(Cloud::Net(Net::Internet), Cloud::Network)
    }

    fn net(&self) -> Net {
        match self {
            Cloud::Net(net) => *net,
            Cloud::Network(n) => Net::of(n.kind).unwrap_or(Net::Lan),
        }
    }

    pub fn key(&self) -> String {
        match self {
            Cloud::Net(net) => net.key().into(),
            Cloud::Network(n) => format!("net {}", n.id()),
        }
    }

    pub fn edge(&self) -> Class {
        self.net().edge()
    }

    pub fn draw(&self, doc: &mut Doc, parent: Option<&str>) {
        let label = match self {
            Cloud::Net(net) => net.label().into(),
            Cloud::Network(nw) => n::network(nw.name.as_deref(), &nw.cidrs.iter().join("\n")),
        };
        doc.place(parent, &self.key(), &label, self.net().cloud());
    }
}
