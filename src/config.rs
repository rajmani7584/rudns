use std::net::Ipv4Addr;

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) srv_name: String,
    pub(crate) srv_type: String,
    pub(crate) hostname: String,
    pub(crate) ip: Ipv4Addr,
    pub(crate) port: u16,
    pub(crate) txt: Vec<(String, String)>,
    pub(crate) interface: Ipv4Addr,
    pub(crate) ttl: u32,
}

impl Config {
    pub(crate) fn instance_fqdn(&self) -> String {
        format!("{}.{}.local.", self.srv_name, self.srv_type)
    }
    pub(crate) fn service_fqdn(&self) -> String {
        format!("{}.local.", self.srv_type)
    }
    pub(crate) fn host_fqdn(&self) -> String {
        format!("{}.local.", self.hostname)
    }

    // fn ttl(&mut self, ttl: u32) {
    //     self.ttl = ttl;
    // }
}

#[derive(PartialEq, Eq)]
pub(crate) enum RecordType {
    A = 1,
    PTR = 12,
    TXT = 16,
    AAAA = 28,
    SRV = 33,
    ANY = 255,
}

impl RecordType {
    pub(crate) fn from_u16(v: u16) -> Option<Self> {
        match v {
            1 => Some(Self::A),
            12 => Some(Self::PTR),
            16 => Some(Self::TXT),
            28 => Some(Self::AAAA),
            33 => Some(Self::SRV),
            255 => Some(Self::ANY),
            _ => None,
        }
    }
}