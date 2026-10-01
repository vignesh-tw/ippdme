//! Command-line TLS options for the client mode.

use std::path::PathBuf;

use ippdme_net::{NetError, TlsClientConfig, TlsIdentity};

/// TLS settings for outbound connections, as given on the command line.
#[derive(Clone)]
pub struct TlsOptions {
    ca_cert: PathBuf,
    server_name: Option<String>,
    identity: Option<(PathBuf, PathBuf)>,
}

impl TlsOptions {
    /// `ca_cert` turns TLS on; `server_name` and the client identity (cert
    /// and key together) refine it.
    pub fn new(
        ca_cert: PathBuf,
        server_name: Option<String>,
        identity: Option<(PathBuf, PathBuf)>,
    ) -> Self {
        TlsOptions {
            ca_cert,
            server_name,
            identity,
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
