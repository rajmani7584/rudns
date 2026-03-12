## Rust - mDNS Server

Example
```rs
use std::net::Ipv4Addr;

use rust_ns::mdns::Server;
use tokio::signal;

#[tokio::main]
async fn main() {
    let ip = Ipv4Addr::new(192, 168, 1, 1);
    let port = 80;

    let mut server = Server::new("_http._tcp", ip, port);

    // server.add_txt_info(("path", "/"));
    // server.add_txt_info(("version", "1.0"));

    // server.set_hostname("rustns");
    // server.set_service_name("Rust-NS by @rajmani7584");

    // server.ttl(4500);

    server.run().await.unwrap();

    signal::ctrl_c().await.unwrap();

    println!("Stopping..");

    server.stop().await.unwrap();
}
```

### Include lib to your cargo project:
```toml
[dependencies]
rudns = { git = "https://github.com/rajmani7584/rust-ns", version = "0.1.0" }
```

### OR by cargo add -
```sh
cargo add --git https://github.com/rajmani7584/rust-ns
```
