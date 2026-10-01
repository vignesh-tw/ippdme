use ippdme_imposter::{Imposter, Predicate, ResponseSpec, Stub};
use ippdme_net::{IppClient, TlsClientConfig, TlsIdentity, TlsServerConfig};
use rcgen::{BasicConstraints, CertificateParams, IsCa, Issuer, KeyPair};

struct Pki {
    ca_pem: String,
    server_cert: String,
    server_key: String,
}

fn make_pki() -> Pki {
    let mut ca_params = CertificateParams::new(vec![]).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_key = KeyPair::generate().unwrap();
    let ca_cert = ca_params.clone().self_signed(&ca_key).unwrap();
    let issuer = Issuer::new(ca_params, ca_key);

    let server_key = KeyPair::generate().unwrap();
    let server_cert = CertificateParams::new(vec!["localhost".into()])
        .unwrap()
        .signed_by(&server_key, &issuer)
        .unwrap();
    Pki {
        ca_pem: ca_cert.pem(),
        server_cert: server_cert.pem(),
        server_key: server_key.serialize_pem(),
    }
}

fn client_config(pki: &Pki) -> TlsClientConfig {
    TlsClientConfig::new("localhost", pki.ca_pem.as_bytes(), None).unwrap()
}

#[tokio::test]
async fn builder_imposter_serves_tls() {
    let pki = make_pki();
    let identity = TlsIdentity::from_pem(pki.server_cert.clone(), pki.server_key.clone());
    let imposter = Imposter::builder()
        .stub(Stub::when(Predicate::call("Home")).responds_with(ResponseSpec::ack()))
        .tls(TlsServerConfig::new(&identity, None).unwrap())
        .bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = imposter.local_addr().unwrap();
    tokio::spawn(imposter.serve());

    let client = IppClient::connect_tls(addr, &client_config(&pki))
        .await
        .unwrap();
    client.home().await.unwrap();
}

#[tokio::test]
async fn yaml_imposter_serves_tls_from_pem_files() {
    let pki = make_pki();
    let dir = std::env::temp_dir().join(format!("ippdme-imposter-tls-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let cert = dir.join("server.pem");
    let key = dir.join("server.key");
    std::fs::write(&cert, &pki.server_cert).unwrap();
    std::fs::write(&key, &pki.server_key).unwrap();

    let yaml = format!(
        "port: 0\ntls:\n  cert: {}\n  key: {}\nstubs:\n  - predicate:\n      call: Home\n    responses:\n      - ack: Ack\n",
        cert.display(),
        key.display()
    );
    let imposter = Imposter::from_yaml_str(&yaml).await.unwrap();
    let addr = imposter.local_addr().unwrap();
    tokio::spawn(imposter.serve());

    let client = IppClient::connect_tls(addr, &client_config(&pki))
        .await
        .unwrap();
    client.home().await.unwrap();
    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn plain_client_cannot_use_tls_imposter() {
    let pki = make_pki();
    let identity = TlsIdentity::from_pem(pki.server_cert.clone(), pki.server_key.clone());
    let imposter = Imposter::builder()
        .stub(Stub::when(Predicate::call("Home")).responds_with(ResponseSpec::ack()))
        .tls(TlsServerConfig::new(&identity, None).unwrap())
        .bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = imposter.local_addr().unwrap();
    tokio::spawn(imposter.serve());

    let mut client = IppClient::connect(addr).await.unwrap();
    client.set_default_timeout(std::time::Duration::from_millis(500));
    assert!(client.home().await.is_err());
}
