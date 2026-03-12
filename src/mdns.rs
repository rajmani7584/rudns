use std::{error::Error, net::Ipv4Addr, sync::Arc, time::Duration};
use tokio::task::JoinHandle;

use crate::{DEFAULT_TTL, config::Config, responder::MDnsResponder};


pub struct Server {
    responder: Option<Arc<MDnsResponder>>,
    config: Config,
    join_handle: Option<JoinHandle<()>>,
}


impl Server {
    pub fn new(srv_type: &str, ip: Ipv4Addr, port: u16) -> Self {
        let iface = match Self::get_system_ip() {
            Ok(ip) => ip,
            Err(_) => Ipv4Addr::LOCALHOST,
        };

        Self {
            config: Config {
                srv_name: "Rust-NS Service".into(),
                srv_type: srv_type.into(),
                hostname: "rustns".into(),
                ip,
                port,
                txt: vec![],
                interface: iface,
                ttl: DEFAULT_TTL,
            },
            responder: None,
            join_handle: None,
        }
    }
    pub fn get_system_ip() -> Result<Ipv4Addr, Box<dyn Error>> {
        let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
        socket.connect("8.8.8.8:80").ok();
        match socket.local_addr()?.ip() {
            std::net::IpAddr::V4(ipv4_addr) => Ok(ipv4_addr),
            _ => Ok(Ipv4Addr::LOCALHOST),
        }
    }
    pub fn set_hostname(&mut self, hostname: &str) {
        self.config.hostname = hostname.into();
    }

    pub fn set_service_name(&mut self, name: &str) {
        self.config.srv_name = name.into();
    }

    pub fn add_txt_info(&mut self, data: (&str, &str)) {
        let (key, value) = data;
        self.config.txt.push((key.into(), value.into()));
    }

    pub fn set_interface(&mut self, interface: Ipv4Addr) {
        self.config.interface = interface;
    }

    pub fn ttl(&mut self, ttl: u32) {
        self.config.ttl = ttl;
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let config = &self.config;

        let responder = Arc::new(MDnsResponder::new(config.clone())?);
        self.responder = Some(Arc::clone(&responder));

        let re = Arc::clone(&responder);

        let ttl = (self.config.ttl / 100) * 80;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(ttl as u64));
            interval.tick().await;

            loop {
                interval.tick().await;
                let _ = re.announce().await;
            }
        });

        let res = Arc::clone(&responder);
        let jh = tokio::spawn(async move {
            if let Err(err) = res.run().await {
                println!("Error: {err:?}");
            }
        });

        self.join_handle = Some(jh);

        tokio::task::yield_now().await;

        responder.announce().await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        responder.announce().await?;

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), Box<dyn Error>> {
        if let Some(res) = &self.responder {
            for _ in 0..3 { res.goodbye().await?; }
        }
        if let Some(join_handle) = &self.join_handle {
            join_handle.abort();
        }
        Ok(())
    }
}
