//! Command-line options.

use std::path::PathBuf;

use crate::tls::TlsOptions;

pub const USAGE: &str = "\
Usage: ippdme-tui [--ca-cert PEM [--server-name NAME] [--client-cert PEM --client-key PEM]]
                  [--tap-listen PORT]

TLS (client mode): with --ca-cert the client connects over TLS 1.3, verifying
the server against that CA. --server-name is the name the server certificate
must match (default: the host being connected to). --client-cert and
--client-key present a client certificate for mutual TLS. Mock-server mode is
always plain TCP.

Tap mode: --tap-listen is the port the tap accepts clients on (default 1297);
the host and port in the connection bar are where it forwards to. Plain TCP only.";

pub const DEFAULT_TAP_LISTEN: u16 = 1297;

pub struct Cli {
    pub tls: Option<TlsOptions>,
    pub tap_listen: u16,
}

impl Cli {
    /// Parse `args` (without the program name).
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Result<Cli, String> {
        let mut ca_cert = None;
        let mut server_name = None;
        let mut client_cert = None;
        let mut client_key = None;
        let mut tap_listen = DEFAULT_TAP_LISTEN;

        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
            match flag.as_str() {
                "--ca-cert" => ca_cert = Some(PathBuf::from(value()?)),
                "--server-name" => server_name = Some(value()?),
                "--client-cert" => client_cert = Some(PathBuf::from(value()?)),
                "--client-key" => client_key = Some(PathBuf::from(value()?)),
                "--tap-listen" => {
                    let v = value()?;
                    tap_listen = v
                        .parse()
                        .map_err(|_| format!("--tap-listen needs a port number, got {v:?}"))?;
                }
                other => return Err(format!("unknown argument {other:?}")),
            }
        }

        let identity = match (client_cert, client_key) {
            (Some(cert), Some(key)) => Some((cert, key)),
            (None, None) => None,
            _ => return Err("--client-cert and --client-key must be given together".into()),
        };
        let tls = match ca_cert {
            Some(ca_cert) => Some(TlsOptions::new(ca_cert, server_name, identity)),
            None if server_name.is_some() || identity.is_some() => {
                return Err(
                    "--server-name, --client-cert and --client-key require --ca-cert".into(),
                )
            }
            None => None,
        };
        Ok(Cli { tls, tap_listen })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Cli, String> {
        Cli::from_args(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn no_flags_means_plain_tcp_and_default_tap_port() {
        let cli = parse(&[]).unwrap();
        assert!(cli.tls.is_none());
        assert_eq!(cli.tap_listen, DEFAULT_TAP_LISTEN);
    }

    #[test]
    fn ca_cert_enables_tls() {
        assert!(parse(&["--ca-cert", "ca.pem"]).unwrap().tls.is_some());
    }

    #[test]
    fn client_identity_needs_both_files() {
        assert!(parse(&["--ca-cert", "ca.pem", "--client-cert", "c.pem"]).is_err());
    }

    #[test]
    fn tls_flags_without_ca_are_rejected() {
        assert!(parse(&["--server-name", "cmm"]).is_err());
        assert!(parse(&["--bogus"]).is_err());
    }

    #[test]
    fn tap_listen_takes_a_port() {
        assert_eq!(parse(&["--tap-listen", "2000"]).unwrap().tap_listen, 2000);
        assert!(parse(&["--tap-listen", "abc"]).is_err());
        assert!(parse(&["--tap-listen"]).is_err());
    }
}
