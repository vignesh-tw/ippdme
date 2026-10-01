use ippdme_core::CoordSystem;
use ippdme_net::{mock::spawn_ephemeral, IppClient};
use tokio::net::TcpStream;

async fn connected_client() -> IppClient {
    let (addr, _handle) = spawn_ephemeral().await.expect("mock server should bind");
    let stream = TcpStream::connect(addr)
        .await
        .expect("client should connect to mock server");
    IppClient::from_stream(stream)
}

#[tokio::test]
async fn start_session_gets_ready() {
    let client = connected_client().await;
    let resp = client.start_session().await.unwrap();
    assert!(resp.is_ack());
    assert_eq!(resp.term().name(), Some("Ready"));
}

#[tokio::test]
async fn go_to_gets_ack_after_latency() {
    let client = connected_client().await;
    let start = std::time::Instant::now();
    let resp = client.go_to(10.0, 20.0, 5.0).await.unwrap();
    assert!(resp.is_ack());
    assert!(start.elapsed() >= std::time::Duration::from_millis(450));
}

#[tokio::test]
async fn pt_meas_gets_data_response() {
    let client = connected_client().await;
    let resp = client.pt_meas().await.unwrap();
    assert!(resp.is_data());
    assert_eq!(resp.term().get_num_param("X"), Some(10.002));
}

#[tokio::test]
async fn set_coord_system_gets_ack() {
    let client = connected_client().await;
    let resp = client.set_coord_system(CoordSystem::Pcs).await.unwrap();
    assert!(resp.is_ack());
}

#[tokio::test]
async fn unknown_raw_command_gets_error() {
    let client = connected_client().await;
    let term = ippdme_core::Term::unit("TotallyMadeUpCommand");
    let resp = client.send(term).await.unwrap();
    assert!(resp.is_error());
}

#[tokio::test]
async fn multiple_concurrent_requests_correlate_by_tag() {
    let client = std::sync::Arc::new(connected_client().await);
    let a = tokio::spawn({
        let client = client.clone();
        async move { client.go_to(1.0, 2.0, 3.0).await }
    });
    let b = tokio::spawn({
        let client = client.clone();
        async move { client.get_dme_version().await }
    });
    let (a, b) = tokio::join!(a, b);
    assert!(a.unwrap().unwrap().is_ack());
    assert!(b.unwrap().unwrap().is_data());
}

#[tokio::test]
async fn subscribe_observes_events_live() {
    let client = connected_client().await;
    let mut events = client.subscribe();
    let _ = client.start_session().await.unwrap();
    let observed = events.recv().await.unwrap();
    assert!(observed.is_ack());
}

#[tokio::test]
async fn client_and_server_work_over_in_memory_duplex() {
    use ippdme_core::{response, Message, Tag, Term};
    use ippdme_net::{serve_connection, IppClient};

    let (client_side, server_side) = tokio::io::duplex(4096);
    let handler = |tag: Tag, _term: Term| async move { response::ack(tag) };
    tokio::spawn(async move { serve_connection(server_side, &handler).await });

    let client = IppClient::from_stream(client_side);
    let reply = client.home().await.unwrap();
    assert!(matches!(reply, Message::Response { .. }));
    assert!(reply.is_ack());
}

#[tokio::test]
async fn go_to_rejects_non_finite_coordinates_before_sending() {
    let (addr, _server) = ippdme_net::mock::spawn_ephemeral().await.unwrap();
    let client = IppClient::connect(addr).await.unwrap();
    let err = client.go_to(f64::NAN, 0.0, 0.0).await.unwrap_err();
    assert!(matches!(
        err,
        ippdme_net::NetError::Protocol(ippdme_core::IppError::InvalidArgument { .. })
    ));
}
