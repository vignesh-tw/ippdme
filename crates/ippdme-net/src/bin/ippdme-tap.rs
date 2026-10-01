//! Watch the lines flowing between an I++ DME client and server.
//!
//!     ippdme-tap --listen 127.0.0.1:1297 --target 127.0.0.1:1294
//!
//! Point the client at the listen address instead of the server. Every line
//! in either direction is printed as it passes, unchanged on its way through.
//! Plain TCP only (TLS traffic would show as unreadable bytes).

use ippdme_net::{IppTap, TapDirection, TapEvent};

const USAGE: &str = "\
Usage: ippdme-tap --target HOST:PORT [--listen ADDR:PORT]

  --target  the real server to forward to
  --listen  where clients connect to the tap (default 127.0.0.1:1297)";

#[tokio::main]
async fn main() {
    let mut listen = "127.0.0.1:1297".to_string();
    let mut target = None;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        match (flag.as_str(), args.next()) {
            ("--listen", Some(v)) => listen = v,
            ("--target", Some(v)) => target = Some(v),
            _ => {
                eprintln!("{USAGE}");
                std::process::exit(2);
            }
        }
    }
    let Some(target) = target else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };

    let tap = IppTap::bind(&listen, &target).await.unwrap_or_else(|e| {
        eprintln!("cannot listen on {listen}: {e}");
        std::process::exit(1);
    });
    println!(
        "ippdme-tap: {} -> {target} (point the client at {})",
        tap.local_addr().expect("bound listener has an addr"),
        tap.local_addr().expect("bound listener has an addr"),
    );

    let mut events = tap.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                TapEvent::Opened { conn, peer } => println!("[#{conn}] opened from {peer}"),
                TapEvent::Line {
                    conn,
                    direction,
                    line,
                } => {
                    let arrow = match direction {
                        TapDirection::ClientToServer => "client -> server",
                        TapDirection::ServerToClient => "server -> client",
                    };
                    println!("[#{conn}] {arrow}  {line}");
                }
                TapEvent::Closed { conn } => println!("[#{conn}] closed"),
            }
        }
    });

    if let Err(e) = tap.serve().await {
        eprintln!("tap stopped: {e}");
        std::process::exit(1);
    }
}
