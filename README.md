## Rust - mDNS Server

Example
```rs
use rudns::rumdns::RuMdns;

use std::net::Ipv4Addr;
use tokio::signal;

#[tokio::main]
async fn main() {
    let ip = Ipv4Addr::new(192, 168, 1, 1);
    let port = 3000;

    let mut mdns = RuMdns::new("_http_._tcp", ip, port);

    // mdns.add_txt_info(("path", "/"));
    // mdns.add_txt_info(("version", "1.0"));

    // mdns.set_hostname("rudns");
    // mdns.set_service_name("RsDns by Rajmani7584");

    mdns.run().await.unwrap();

    signal::ctrl_c().await.unwrap();

    println!("Stopping..");

    mdns.stop().await.unwrap();
}
```