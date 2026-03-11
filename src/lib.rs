use std::net::Ipv4Addr;

pub mod rumdns;
mod config;
mod responder;
mod query;
mod reader;
mod builder;

pub(crate) const MDNS_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
pub(crate) const MDNS_PORT: u16 = 5353;
pub(crate) const DEFAULT_TTL: u32 = 4500;