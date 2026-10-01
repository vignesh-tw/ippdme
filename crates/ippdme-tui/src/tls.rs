//! Command-line TLS options for the client mode.

use std::path::PathBuf;

use ippdme_net::{NetError, TlsClientConfig, TlsIdentity};

pub const USAGE: &str = "\
Usage: ippdme-tui [--ca-cert PEM [--server-name NAME] [--client-cert PEM --client-key PEM]]

With --ca-cert, client mode connects over TLS 1.3, verifying the server
against that CA. --server-name is the name the server certificate must match
(default: the host being connected to). --client-cert and --client-key
present a client certificate for mutual TLS. Mock-server mode is always
plain TCP.";

/// TLS settings for outbound connections, as given on the command line.
#[derive(Clone)]
pub struct TlsOptions {
    ca_cert: PathBuf,
    server_name: Option<String>,
    identity: Option<(PathBuf, PathBuf)>,
}

impl TlsOptions {
    /// Parse `args` (without the program name). `Ok(None)` means plain TCP.
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Result<Option<Self>, String> {
        let mut ca_cert = None;
        let mut server_name = None;
        let mut client_cert = None;
        let mut client_key = None;

        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
            match flag.as_str() {
                "--ca-cert" => ca_cert = Some(PathBuf::from(value()?)),
                "--server-name" => server_name = Some(value()?),
                "--client-cert" => client_cert = Some(PathBuf::from(value()?)),
                "--client-key" => client_key = Some(PathBuf::from(value()?)),
                other => return Err(format!("unknown argument {other:?}")),
            }
        }

        let identity = match (client_cert, client_key) {
            (Some(cert), Some(key)) => Some((cert, key)),
            (None, None) => None,
            _ => return Err("--client-cert and --client-key must be given together".into()),
        };
        match ca_cert {
            Some(ca_cert) => Ok(Some(TlsOptions {
                ca_cert,
                server_name,
                identity,
            })),
            None if server_name.is_some() || identity.is_some() => {
                Err("--server-name, --client-cert and --client-key require --ca-cert".into())
            }
            None => Ok(None),
        }
    }

    /// Build the connection config for `host`, reading the PEM files.
    pub fn client_config(&self, host: &str) -> Result<TlsClientConfig, NetError> {
        let ca_pem = std::fs::read(&self.ca_cert)?;
        let identity = match &self.identity {
            Some((cert, key)) => Some(TlsIdentity::from_files(cert, key)?),
            None => None,
        };
        let name = self.server_name.as_deref().unwrap_or(host);
        TlsClientConfig::new(name, &ca_pem, identity.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Option<TlsOptions>, String> {
        TlsOptions::from_args(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn no_flags_means_plain_tcp() {
        assert!(parse(&[]).unwrap().is_none());
    }

    #[test]
    fn ca_cert_enables_tls() {
        assert!(parse(&["--ca-cert", "ca.pem"]).unwrap().is_some());
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
}
