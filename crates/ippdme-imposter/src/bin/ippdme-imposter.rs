//! Standalone imposter server: load stubs from a YAML file and serve them
//! until the process is killed.
//!
//! Usage: `ippdme-imposter <path-to-imposter.yaml>`

use ippdme_imposter::Imposter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: ippdme-imposter <path-to-imposter.yaml>");
        std::process::exit(2);
    });

    let imposter = Imposter::from_yaml_file(&path).await.unwrap_or_else(|e| {
        eprintln!("failed to load {path}: {e}");
        std::process::exit(1);
    });

    let addr = imposter.local_addr().expect("bound listener has an addr");
    println!("ippdme-imposter listening on {addr} ({path})");

    if let Err(e) = imposter.serve().await {
        eprintln!("imposter server stopped: {e}");
        std::process::exit(1);
    }
}
