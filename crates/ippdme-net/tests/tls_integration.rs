#![cfg(feature = "tls")]

use ippdme_net::{IppClient, IppMockServer, TlsClientConfig, TlsIdentity, TlsServerConfig};
use rcgen::{BasicConstraints, CertificateParams, IsCa, Issuer, KeyPair};

struct Ca {
    cert: rcgen::Certificate,
    issuer: Issuer<'static, KeyPair>,
}

impl Ca {
    fn new() -> Self {
        let mut params = CertificateParams::new(vec![]).unwrap();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let key = KeyPair::generate().unwrap();
        let cert = params.clone().self_signed(&key).unwrap();
        Ca {
            cert,
            issuer: Issuer::new(params, key),
        }
    }

    fn pem(&self) -> Vec<u8> {
        self.cert.pem().into_bytes()
    }

    fn issue(&self, name: &str) -> TlsIdentity {
        let params = CertificateParams::new(vec![name.to_string()]).unwrap();
        let key = KeyPair::generate().unwrap();
        let cert = params.signed_by(&key, &self.issuer).unwrap();
        TlsIdentity::from_pem(cert.pem(), key.serialize_pem())
    }
}

async fn spawn_tls_mock(server_cfg: TlsServerConfig) -> std::net::SocketAddr {
    let server = IppMockServer::bind_tls(("127.0.0.1", 0), server_cfg)
        .await
        .unwrap();
    let addr = server.local_addr().unwrap();
    tokio::spawn(server.serve());
    addr
}

#[tokio::test]
async fn client_talks_to_server_over_tls() {
    let ca = Ca::new();
    let server_cfg = TlsServerConfig::new(&ca.issue("localhost"), None).unwrap();
    let addr = spawn_tls_mock(server_cfg).await;

    let client_cfg = TlsClientConfig::new("localhost", &ca.pem(), None).unwrap();
    let client = IppClient::connect_tls(addr, &client_cfg).await.unwrap();
    assert_eq!(client.get_dme_version().await.unwrap(), "1.4");
}

#[tokio::test]
async fn client_rejects_server_with_untrusted_certificate() {
    let server_ca = Ca::new();
    let other_ca = Ca::new();
    let server_cfg = TlsServerConfig::new(&server_ca.issue("localhost"), None).unwrap();
    let addr = spawn_tls_mock(server_cfg).await;

    let client_cfg = TlsClientConfig::new("localhost", &other_ca.pem(), None).unwrap();
    assert!(IppClient::connect_tls(addr, &client_cfg).await.is_err());
}

#[tokio::test]
async fn client_rejects_server_with_wrong_name() {
    let ca = Ca::new();
    let server_cfg = TlsServerConfig::new(&ca.issue("localhost"), None).unwrap();
    let addr = spawn_tls_mock(server_cfg).await;

    let client_cfg = TlsClientConfig::new("not-localhost", &ca.pem(), None).unwrap();
    assert!(IppClient::connect_tls(addr, &client_cfg).await.is_err());
}

#[tokio::test]
async fn mutual_tls_accepts_known_client_and_rejects_anonymous_one() {
    let ca = Ca::new();
    let server_cfg = TlsServerConfig::new(&ca.issue("localhost"), Some(&ca.pem())).unwrap();
    let addr = spawn_tls_mock(server_cfg).await;

    let with_cert =
        TlsClientConfig::new("localhost", &ca.pem(), Some(&ca.issue("client"))).unwrap();
    let client = IppClient::connect_tls(addr, &with_cert).await.unwrap();
    assert_eq!(client.get_dme_version().await.unwrap(), "1.4");

    // In TLS 1.3 the client can finish its handshake before the server
    // rejects it, so the failure may surface on connect or on first request.
    let anonymous = TlsClientConfig::new("localhost", &ca.pem(), None).unwrap();
    let rejected = match IppClient::connect_tls(addr, &anonymous).await {
        Err(_) => true,
        Ok(mut client) => {
            client.set_default_timeout(std::time::Duration::from_millis(500));
            client.get_dme_version().await.is_err()
        }
    };
    assert!(rejected);
}

#[tokio::test]
async fn tls_connect_to_a_silent_peer_times_out_instead_of_hanging() {
    // Accepts TCP but never answers the ClientHello, like a machine that
    // is waiting for a CRLF-terminated I++ line.
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _held = listener.accept().await;
        std::future::pending::<()>().await;
    });

    let ca = Ca::new();
    let limit = std::time::Duration::from_millis(300);
    let cfg = TlsClientConfig::new("localhost", &ca.pem(), None)
        .unwrap()
        .with_connect_timeout(limit);
    let started = std::time::Instant::now();
    match IppClient::connect_tls(addr, &cfg).await {
        Err(ippdme_net::NetError::ConnectTimeout(d)) => assert_eq!(d, limit),
        other => panic!("expected ConnectTimeout, got {:?}", other.err()),
    }
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
}

#[tokio::test]
async fn tls_connect_to_a_plaintext_server_fails() {
    let plain = IppMockServer::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = plain.local_addr().unwrap();
    tokio::spawn(plain.serve());

    let ca = Ca::new();
    let cfg = TlsClientConfig::new("localhost", &ca.pem(), None)
        .unwrap()
        .with_connect_timeout(std::time::Duration::from_secs(2));
    assert!(IppClient::connect_tls(addr, &cfg).await.is_err());
}
