use std::{error::Error, net::{IpAddr, Ipv4Addr, SocketAddr}};

use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;
use crate::{MDNS_ADDR, MDNS_PORT, builder::Builder, config::{Config, RecordType}, query::Query};


pub(crate) struct MDnsResponder {
    socket: UdpSocket,
    // socket: Arc<UdpSocket>,
    config: Config,
}

impl MDnsResponder {
    pub(crate) fn new(config: Config) -> Result<Self, Box<dyn Error>> {
        let socket = Self::create_mdns_socket(config.interface)?;

        let socket = UdpSocket::from_std(socket)?;

        Ok(Self { socket, config })
    }

    async fn send_multicast(&self, buffer: &[u8]) -> Result<(), Box<dyn Error>> {
        let dest = SocketAddr::new(IpAddr::V4(MDNS_ADDR), MDNS_PORT);
        self.socket.send_to(buffer, dest).await?;
        Ok(())
    }

    pub(crate) async fn announce(&self) -> Result<(), Box<dyn Error>> {
        let pkt = self.build_response(true, true, true, true, false);
        self.send_multicast(&pkt).await?;
        Ok(())
    }

    pub(crate) async fn goodbye(&self) -> Result<(), Box<dyn Error>> {
        let pkt = self.build_response(true, true, true, true, true);

        self.send_multicast(&pkt).await?;

        Ok(())
    }

    pub(crate) async fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut buffer = [0u8; 4096];

        loop {
            let (len, src) = self.socket.recv_from(&mut buffer).await?;

            let data = &buffer[..len];

            let Some(query) = Query::parse_query(data) else {
                continue;
            };

            if let Some(pkt) = self.should_respond(&query) {
                let dest = if src.port() == MDNS_PORT
                    && query.questions.iter().any(|q| q.qclass & 0x8000 != 0)
                {
                    src
                } else {
                    SocketAddr::new(IpAddr::V4(MDNS_ADDR), MDNS_PORT)
                };
                self.socket.send_to(&pkt, dest).await?;
            }
        }
    }

    fn should_respond(&self, query: &Query) -> Option<Vec<u8>> {
        let cfg = &self.config;
        let mut matched_ptr = false;
        let mut matched_srv = false;
        let mut matched_txt = false;
        let mut matched_a = false;

        let mut matched_meta = false;

        for q in &query.questions {
            let qt = q.qtype;
            let name = q.name.to_ascii_lowercase();

            let is_any = RecordType::from_u16(qt) == Some(RecordType::ANY);
            let is_ptr = RecordType::from_u16(qt) == Some(RecordType::PTR);
            let is_srv = RecordType::from_u16(qt) == Some(RecordType::SRV);
            let is_txt = RecordType::from_u16(qt) == Some(RecordType::TXT);
            let is_a = RecordType::from_u16(qt) == Some(RecordType::A);

            if (is_ptr || is_any) && name == "_services._dns-sd._udp.local." {
                matched_meta = true;
            }

            if (is_ptr || is_any) && name == cfg.service_fqdn().to_ascii_lowercase() {
                matched_ptr = true;
                matched_srv = true;
                matched_txt = true;
                matched_a = true;
            }
            if (is_srv || is_any) && name == cfg.instance_fqdn().to_ascii_lowercase() {
                matched_srv = true;
            }
            if (is_txt || is_any) && name == cfg.instance_fqdn().to_ascii_lowercase() {
                matched_txt = true;
            }
            if (is_a || is_any) && name == cfg.host_fqdn().to_ascii_lowercase() {
                matched_a = true;
            }
        }
        if matched_meta {
            return Some(self.build_meta_response());
        }

        if !matched_ptr && !matched_srv && !matched_txt && !matched_a {
            return None;
        }

        let pkt = self.build_response(matched_ptr, matched_srv, matched_txt, matched_a, false);
        Some(pkt)
    }

    fn build_meta_response(&self) -> Vec<u8> {
        let cfg = &self.config;
        let mut b = Builder::response(0);
        b.ptr(
            "_services._dns-sd._udp.local.",
            &cfg.service_fqdn(),
            cfg.ttl,
        );
        b.build()
    }
    fn build_response(&self, ptr: bool, srv: bool, txt: bool, a: bool, goodbye: bool) -> Vec<u8> {
        let config = &self.config;

        let ttl = if goodbye { 0 } else { config.ttl };
        let txt_data: Vec<(&str, &str)> = config
            .txt
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let mut builder = Builder::response(0);

        builder.ptr("_services._dns-sd._udp.local.", &config.service_fqdn(), ttl);
        if ptr {
            builder.ptr(&config.service_fqdn(), &config.instance_fqdn(), ttl);
        }
        if srv {
            builder.srv(
                &config.instance_fqdn(),
                &config.host_fqdn(),
                0,
                0,
                config.port,
                ttl,
            );
        }
        if txt {
            builder.txt(&config.instance_fqdn(), &txt_data, ttl);
        }
        if a {
            if ptr || srv {
                builder.additional();
            }
            builder.a(&config.host_fqdn(), config.ip, ttl);
        }

        builder.build()
    }

    fn create_mdns_socket(iface: Ipv4Addr) -> Result<std::net::UdpSocket, Box<dyn Error>> {
        let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        sock.set_reuse_address(true)?;
        sock.set_reuse_port(true)?;
        sock.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), MDNS_PORT).into())?;

        sock.join_multicast_v4(&MDNS_ADDR, &iface)?;
        // sock.set_multicast_ttl_v4(255)?;
        sock.set_multicast_loop_v4(false)?;
        sock.set_multicast_if_v4(&iface)?;
        sock.set_nonblocking(true)?;
        Ok(sock.into())
    }
}