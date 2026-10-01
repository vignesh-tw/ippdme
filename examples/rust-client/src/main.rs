//! Example application built on `ippdme-core` (building commands) and
//! `ippdme-net` (sending them): runs a short session against any I++ DME
//! server, such as one of the imposters in `examples/`.
//!
//!     cargo run -p ippdme-example-client                      # plain, 127.0.0.1:1294
//!     cargo run -p ippdme-example-client -- --addr 127.0.0.1:1295 --ca-cert examples/tls/certs/ca.pem
//!     cargo run -p ippdme-example-client -- --addr 127.0.0.1:1296 --ca-cert examples/tls/certs/ca.pem \
//!         --client-cert examples/tls/certs/client.pem --client-key examples/tls/certs/client.key

use std::process::ExitCode;

use ippdme_core::{response, Command, IppError};
use ippdme_net::{IppClient, NetError, TlsClientConfig, TlsIdentity};

const USAGE: &str = "\
Usage: ippdme-example-client [--addr HOST:PORT] [--server-name NAME]
                             [--ca-cert PEM [--client-cert PEM --client-key PEM]]

Without --ca-cert the connection is plain TCP; with it, TLS 1.3 (and mutual TLS
when a client certificate and key are given). --server-name defaults to the host.";

struct Options {
    addr: String,
    server_name: Option<String>,
    ca_cert: Option<String>,
    client_cert: Option<String>,
    client_key: Option<String>,
}

fn parse_args() -> Result<Options, String> {
    let mut opts = Options {
        addr: "127.0.0.1:1294".into(),
        server_name: None,
        ca_cert: None,
        client_cert: None,
        client_key: None,
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--addr" => opts.addr = value()?,
            "--server-name" => opts.server_name = Some(value()?),
            "--ca-cert" => opts.ca_cert = Some(value()?),
            "--client-cert" => opts.client_cert = Some(value()?),
            "--client-key" => opts.client_key = Some(value()?),
            "-h" | "--help" => return Err(String::new()),
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    Ok(opts)
}

async fn connect(opts: &Options) -> Result<IppClient, NetError> {
    let Some(ca_cert) = &opts.ca_cert else {
        return IppClient::connect(&opts.addr).await;
    };
    let identity = match (&opts.client_cert, &opts.client_key) {
        (Some(cert), Some(key)) => Some(TlsIdentity::from_files(cert, key)?),
        _ => None,
    };
    let host = opts.addr.rsplit_once(':').map_or(&*opts.addr, |(h, _)| h);
    let name = opts.server_name.as_deref().unwrap_or(host);
    let tls = TlsClientConfig::new(name, &std::fs::read(ca_cert)?, identity.as_ref())?;
    IppClient::connect_tls(&opts.addr, &tls).await
}

/// Print a labelled result, showing a server `Error(..)` reply distinctly
/// from a transport failure.
fn report<T: std::fmt::Debug>(label: &str, result: Result<T, NetError>) -> bool {
    match result {
        Ok(value) => {
            println!("{label}: {value:?}");
            true
        }
        Err(NetError::Protocol(IppError::ServerError { reason })) => {
            println!("{label}: server said Error({reason})");
            false
        }
        Err(e) => {
            println!("{label}: failed: {e}");
            false
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let opts = match parse_args() {
        Ok(opts) => opts,
        Err(msg) => {
            if !msg.is_empty() {
                eprintln!("{msg}\n");
            }
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    let client = match connect(&opts).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("could not connect to {}: {e}", opts.addr);
            return ExitCode::FAILURE;
        }
    };
    println!("connected to {}", opts.addr);

    // Typed helpers: commands are built and validated by ippdme-core, and
    // replies come back as typed values rather than raw lines.
    let mut ok = report("StartSession", client.start_session().await);
    ok &= report("GetDMEVersion", client.get_dme_version().await);
    ok &= report("Home", client.home().await);
    ok &= report("GoTo(10, 20, 5)", client.go_to(10.0, 20.0, 5.0).await);
    ok &= report("PtMeas", client.pt_meas().await);

    // A command with no typed helper goes through send_command, which returns
    // the raw reply; response::expect_ack turns an Error(..) reply into an Err.
    let tool = ippdme_core::ToolName::new("RefTool").expect("valid tool name");
    let changed = client.send_command(Command::ChangeTool(tool)).await;
    ok &= report(
        "ChangeTool(RefTool)",
        changed.and_then(|reply| Ok(response::expect_ack(&reply)?)),
    );

    ok &= report("EndSession", client.end_session().await);

    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
