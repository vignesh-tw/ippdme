//! Runs the `ippdme-tap` binary and checks that its output looks like the
//! transcript in `examples/README.md`.

use std::process::Stdio;
use std::time::Duration;

use ippdme_net::{IppClient, IppMockServer, MockConfig};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

type Lines = tokio::io::Lines<BufReader<tokio::process::ChildStdout>>;

async fn next_line(out: &mut Lines) -> String {
    timeout(Duration::from_secs(5), out.next_line())
        .await
        .expect("tap printed nothing in time")
        .unwrap()
        .expect("tap closed its output")
}

#[tokio::test]
async fn tap_binary_prints_the_documented_transcript() {
    let readme = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/README.md"),
    )
    .unwrap();

    let server = IppMockServer::builder()
        .config(MockConfig::instant())
        .bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let target = server.local_addr().unwrap();
    tokio::spawn(server.serve());

    let mut tap = Command::new(env!("CARGO_BIN_EXE_ippdme-tap"))
        .args(["--listen", "127.0.0.1:0", "--target", &target.to_string()])
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut out = BufReader::new(tap.stdout.take().unwrap()).lines();

    // The header names the real listen address: "ippdme-tap: <addr> -> <target> ..."
    let header = next_line(&mut out).await;
    let listen = header
        .strip_prefix("ippdme-tap: ")
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or_else(|| panic!("unexpected header {header:?}"))
        .to_string();

    let client = IppClient::connect(&listen).await.unwrap();
    client.start_session().await.unwrap();
    drop(client);

    let opened = next_line(&mut out).await;
    assert!(opened.starts_with("[#1] opened from "), "{opened}");
    let request = next_line(&mut out).await;
    let reply = next_line(&mut out).await;
    let closed = next_line(&mut out).await;

    // The README shows exactly these lines.
    assert_eq!(request, "[#1] client -> server  00001 StartSession()");
    assert_eq!(reply, "[#1] server -> client  00001 # Ready()");
    assert_eq!(closed, "[#1] closed");
    for line in [&request, &reply, &closed] {
        assert!(
            readme.contains(line.as_str()),
            "README no longer shows {line:?}"
        );
    }
    assert!(readme.contains("[#1] opened from "));
}
