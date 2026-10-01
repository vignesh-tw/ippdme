//! Runs the examples for real and checks that what `examples/README.md`
//! promises is what actually happens, so the examples can't go stale as the
//! code changes. Every imposter YAML is started on an ephemeral port (the
//! files hard-code 1294 etc. for humans) by rewriting only its `port:` line.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use ippdme_core::IppError;
use ippdme_imposter::Imposter;
use ippdme_net::{IppClient, NetError, TlsClientConfig, TlsIdentity};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// Start the imposter described by `examples/<yaml>` on an ephemeral port.
/// `certs` redirects the YAML's `examples/tls/certs/` paths to a folder made
/// by [`make_certs`].
async fn start(yaml: &str, certs: Option<&Path>) -> SocketAddr {
    let mut text = read(&format!("examples/{yaml}"))
        .lines()
        .map(|l| if l.starts_with("port:") { "port: 0" } else { l })
        .collect::<Vec<_>>()
        .join("\n");
    if let Some(dir) = certs {
        text = text.replace("examples/tls/certs/", &format!("{}/", dir.display()));
    }
    let imposter = Imposter::from_yaml_str(&text).await.unwrap();
    let addr = imposter.local_addr().unwrap();
    tokio::spawn(imposter.serve());
    addr
}

/// Run the documented `examples/tls/make-certs.sh` in a scratch folder and
/// return the folder holding the certificates.
fn make_certs() -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "ippdme-examples-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    // The script writes next to itself, so run a copy.
    let script = dir.join("make-certs.sh");
    std::fs::copy(repo_root().join("examples/tls/make-certs.sh"), &script).unwrap();
    let status = std::process::Command::new("sh")
        .arg(&script)
        .status()
        .expect("sh and openssl are needed to run make-certs.sh");
    assert!(status.success(), "make-certs.sh failed");
    dir.join("certs")
}

fn tls_config(certs: &Path, name: &str, with_client_cert: bool) -> TlsClientConfig {
    let identity = with_client_cert.then(|| {
        TlsIdentity::from_files(certs.join("client.pem"), certs.join("client.key")).unwrap()
    });
    let ca = std::fs::read(certs.join("ca.pem")).unwrap();
    TlsClientConfig::new(name, &ca, identity.as_ref())
        .unwrap()
        .with_connect_timeout(Duration::from_secs(5))
}

async fn run_client(args: &[&str]) -> Output {
    tokio::process::Command::new(env!("CARGO_BIN_EXE_ippdme-example-client"))
        .args(args)
        .output()
        .await
        .unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

// --- plain imposter ---------------------------------------------------

/// The `nc` session in the README, line for line.
#[tokio::test]
async fn readme_nc_session_matches_the_plain_imposter() {
    let readme = read("examples/README.md");
    let session = [
        ("00001 StartSession()", "00001 # Ready()"),
        ("00002 GetDMEVersion()", "00002 % DMEVersion(\"1.5\")"),
        (
            "00003 PtMeas()",
            "00003 % PtMeas(X(10.002), Y(20.001), Z(5.0), I(0.0), J(0.0), K(1.0))",
        ),
        ("00004 Bogus()", "00004 ! Error(UnknownStub)"),
    ];

    let addr = start("plain/imposter.yaml", None).await;
    let mut stream = BufReader::new(TcpStream::connect(addr).await.unwrap());
    for (sent, expected) in session {
        assert!(readme.contains(sent), "README no longer shows {sent:?}");
        assert!(
            readme.contains(expected),
            "README no longer shows {expected:?}"
        );

        stream
            .get_mut()
            .write_all(format!("{sent}\r\n").as_bytes())
            .await
            .unwrap();
        let mut reply = String::new();
        stream.read_line(&mut reply).await.unwrap();
        assert_eq!(reply.trim_end(), expected);
    }
}

#[tokio::test]
async fn rust_client_runs_against_the_plain_example() {
    let addr = start("plain/imposter.yaml", None).await;
    let out = run_client(&["--addr", &addr.to_string()]).await;
    let text = stdout(&out);
    assert!(out.status.success(), "{text}");
    for line in [
        "connected to",
        "StartSession: ()",
        "GetDMEVersion: \"1.5\"",
        "GoTo(10, 20, 5): ()",
        "ChangeTool(RefTool): ()",
        "EndSession: ()",
    ] {
        assert!(text.contains(line), "missing {line:?} in:\n{text}");
    }
    assert!(text.contains("x: Some(10.002)"), "{text}");
}

// --- faults -----------------------------------------------------------

#[tokio::test]
async fn faults_example_behaves_as_the_readme_table_says() {
    let readme = read("examples/README.md");
    for promised in [
        "CollisionDetected",
        "this is not an I++ line",
        "connection closed",
        "8 seconds",
    ] {
        assert!(
            readme.contains(promised),
            "README no longer mentions {promised:?}"
        );
    }
    let addr = start("plain/faults.yaml", None).await;

    // GoTo: collision the first time, fine after.
    let client = IppClient::connect(addr).await.unwrap();
    match client.go_to(1.0, 2.0, 3.0).await {
        Err(NetError::Protocol(IppError::ServerError { reason })) => {
            assert_eq!(reason, "CollisionDetected")
        }
        other => panic!("expected CollisionDetected, got {other:?}"),
    }
    client.go_to(1.0, 2.0, 3.0).await.unwrap();

    // PtMeas: slower than the client is willing to wait.
    let mut slow = IppClient::connect(addr).await.unwrap();
    slow.set_default_timeout(Duration::from_millis(300));
    assert!(matches!(slow.pt_meas().await, Err(NetError::Timeout(_))));

    // IsHomed: the machine hangs up.
    let dropped = IppClient::connect(addr).await.unwrap();
    assert!(matches!(
        dropped.is_homed().await,
        Err(NetError::ConnectionClosed)
    ));

    // GetMachineClass: a line that isn't I++ at all.
    let mut raw = BufReader::new(TcpStream::connect(addr).await.unwrap());
    raw.get_mut()
        .write_all(b"00001 GetMachineClass()\r\n")
        .await
        .unwrap();
    let mut line = String::new();
    raw.read_line(&mut line).await.unwrap();
    assert_eq!(line.trim_end(), "this is not an I++ line");
}

#[tokio::test]
async fn rust_client_reports_the_documented_faults() {
    let addr = start("plain/faults.yaml", None).await;
    let out = run_client(&["--addr", &addr.to_string(), "--timeout", "0.5"]).await;
    let text = stdout(&out);
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains("server said Error(CollisionDetected)"),
        "{text}"
    );
    assert!(text.contains("request timed out"), "{text}");
}

// --- TLS --------------------------------------------------------------

#[tokio::test]
async fn tls_examples_work_with_the_documented_settings() {
    let readme = read("examples/README.md");
    let certs = make_certs();
    for name in [
        "ca.pem",
        "server.pem",
        "server.key",
        "client.pem",
        "client.key",
    ] {
        assert!(
            certs.join(name).exists(),
            "make-certs.sh did not write {name}"
        );
    }

    // TLS: the documented client settings connect; the README's flags exist.
    let tls = start("tls/imposter-tls.yaml", Some(&certs)).await;
    let ok = tls_config(&certs, "localhost", false);
    let client = IppClient::connect_tls(tls, &ok).await.unwrap();
    assert_eq!(client.get_dme_version().await.unwrap(), "1.5");

    // The pitfall the README warns about: the certificate is for `localhost`,
    // not the IP address.
    assert!(readme.contains("--server-name localhost"));
    let by_ip = tls_config(&certs, "127.0.0.1", false);
    assert!(IppClient::connect_tls(tls, &by_ip).await.is_err());

    // The Rust client with the README's TLS flags.
    let ca = certs.join("ca.pem");
    let out = run_client(&[
        "--addr",
        &tls.to_string(),
        "--ca-cert",
        ca.to_str().unwrap(),
        "--server-name",
        "localhost",
    ])
    .await;
    assert!(out.status.success(), "{}", stdout(&out));

    // Mutual TLS: a client certificate is required.
    let mtls = start("tls/imposter-mtls.yaml", Some(&certs)).await;
    let with_cert = tls_config(&certs, "localhost", true);
    let client = IppClient::connect_tls(mtls, &with_cert).await.unwrap();
    assert_eq!(client.get_dme_version().await.unwrap(), "1.5");

    let mut anonymous = IppClient::connect_tls(mtls, &tls_config(&certs, "localhost", false))
        .await
        .unwrap();
    anonymous.set_default_timeout(Duration::from_millis(500));
    assert!(anonymous.get_dme_version().await.is_err());

    let (client_pem, client_key) = (certs.join("client.pem"), certs.join("client.key"));
    let out = run_client(&[
        "--addr",
        &mtls.to_string(),
        "--ca-cert",
        ca.to_str().unwrap(),
        "--server-name",
        "localhost",
        "--client-cert",
        client_pem.to_str().unwrap(),
        "--client-key",
        client_key.to_str().unwrap(),
    ])
    .await;
    assert!(out.status.success(), "{}", stdout(&out));

    let _ = std::fs::remove_dir_all(certs.parent().unwrap());
}
