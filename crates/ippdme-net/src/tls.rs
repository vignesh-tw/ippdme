//! TLS 1.3 (optionally mutual) configuration for [`crate::IppClient`] and
//! [`crate::IppServer`]. Enabled by the `tls` feature.
//!
//! Certificates and keys are passed as PEM bytes so callers decide where
//! they come from; the `*_file` helpers read them from disk.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::server::WebPkiClientVerifier;
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use tokio_rustls::{TlsAcceptor, TlsConnector};

use crate::error::{NetError, Result};

fn tls_err(e: impl std::fmt::Display) -> NetError {
    NetError::Tls(e.to_string())
}

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

fn certs(pem: &[u8]) -> Result<Vec<CertificateDer<'static>>> {
    CertificateDer::pem_slice_iter(pem)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(tls_err)
}

fn roots(pem: &[u8]) -> Result<RootCertStore> {
    let mut store = RootCertStore::empty();
    for cert in certs(pem)? {
        store.add(cert).map_err(tls_err)?;
    }
    Ok(store)
}

/// How long a TLS handshake may take before the connection is dropped, on
/// both the client and the server side by default.
pub const DEFAULT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

/// A certificate chain and its private key, both PEM-encoded.
#[derive(Clone)]
pub struct TlsIdentity {
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
}

impl TlsIdentity {
    pub fn from_pem(cert_pem: impl Into<Vec<u8>>, key_pem: impl Into<Vec<u8>>) -> Self {
        TlsIdentity {
            cert_pem: cert_pem.into(),
            key_pem: key_pem.into(),
        }
    }

    pub fn from_files(cert: impl AsRef<Path>, key: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::from_pem(std::fs::read(cert)?, std::fs::read(key)?))
    }

    fn parts(&self) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
        let key = PrivateKeyDer::from_pem_slice(&self.key_pem).map_err(tls_err)?;
        Ok((certs(&self.cert_pem)?, key))
    }
}

/// Client-side TLS settings: which CA to trust, the name to verify the
/// server certificate against, and an optional client identity for mutual
/// TLS. Only TLS 1.3 is offered.
#[derive(Clone)]
pub struct TlsClientConfig {
    connector: TlsConnector,
    server_name: ServerName<'static>,
    connect_timeout: Duration,
}

impl TlsClientConfig {
    pub fn new(
        server_name: &str,
        root_ca_pem: &[u8],
        identity: Option<&TlsIdentity>,
    ) -> Result<Self> {
        let builder = ClientConfig::builder_with_provider(provider())
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(tls_err)?
            .with_root_certificates(roots(root_ca_pem)?);
        let config = match identity {
            Some(id) => {
                let (chain, key) = id.parts()?;
                builder.with_client_auth_cert(chain, key).map_err(tls_err)?
            }
            None => builder.with_no_client_auth(),
        };
        let server_name = ServerName::try_from(server_name.to_owned()).map_err(tls_err)?;
        Ok(TlsClientConfig {
            connector: TlsConnector::from(Arc::new(config)),
            server_name,
            connect_timeout: DEFAULT_HANDSHAKE_TIMEOUT,
        })
    }

    /// Limit for the TCP connect plus the TLS handshake (default 5s).
    pub fn with_connect_timeout(mut self, limit: Duration) -> Self {
        self.connect_timeout = limit;
        self
    }

    pub(crate) fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }

    pub(crate) fn connector(&self) -> &TlsConnector {
        &self.connector
    }

    pub(crate) fn server_name(&self) -> ServerName<'static> {
        self.server_name.clone()
    }
}

/// Server-side TLS settings: the server's identity and, to require mutual
/// TLS, the CA that client certificates must chain to. Only TLS 1.3 is
/// offered.
#[derive(Clone)]
pub struct TlsServerConfig {
    acceptor: TlsAcceptor,
}

impl TlsServerConfig {
    pub fn new(identity: &TlsIdentity, client_ca_pem: Option<&[u8]>) -> Result<Self> {
        let builder = ServerConfig::builder_with_provider(provider())
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(tls_err)?;
        let builder = match client_ca_pem {
            Some(ca) => {
                let verifier =
                    WebPkiClientVerifier::builder_with_provider(Arc::new(roots(ca)?), provider())
                        .build()
                        .map_err(tls_err)?;
                builder.with_client_cert_verifier(verifier)
            }
            None => builder.with_no_client_auth(),
        };
        let (chain, key) = identity.parts()?;
        let config = builder.with_single_cert(chain, key).map_err(tls_err)?;
        Ok(TlsServerConfig {
            acceptor: TlsAcceptor::from(Arc::new(config)),
        })
    }

    pub(crate) fn acceptor(&self) -> &TlsAcceptor {
        &self.acceptor
    }
}
