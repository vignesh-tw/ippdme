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
    client.start_session().await.unwrap();
}

#[tokio::test]
async fn go_to_gets_ack_after_latency() {
    let client = connected_client().await;
    let start = std::time::Instant::now();
    client.go_to(10.0, 20.0, 5.0).await.unwrap();
    assert!(start.elapsed() >= std::time::Duration::from_millis(450));
}

#[tokio::test]
async fn pt_meas_gets_data_response() {
    let client = connected_client().await;
    let point = client.pt_meas().await.unwrap();
    assert_eq!(point.x(), Some(10.002));
    assert_eq!(point.k(), Some(1.0));
}

#[tokio::test]
async fn set_coord_system_gets_ack() {
    let client = connected_client().await;
    client.set_coord_system(CoordSystem::Pcs).await.unwrap();
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
    a.unwrap().unwrap();
    assert_eq!(b.unwrap().unwrap(), "1.4");
}

#[tokio::test]
async fn subscribe_observes_events_live() {
    let client = connected_client().await;
    let mut events = client.subscribe();
    client.start_session().await.unwrap();
    let observed = events.recv().await.unwrap();
    assert!(observed.is_ack());
}

#[tokio::test]
async fn client_and_server_work_over_in_memory_duplex() {
    use ippdme_core::{response, Tag, Term};
    use ippdme_net::{serve_connection, IppClient};

    let (client_side, server_side) = tokio::io::duplex(4096);
    let handler = |tag: Tag, _term: Term| async move { response::ack(tag) };
    tokio::spawn(async move { serve_connection(server_side, &handler).await });

    let client = IppClient::from_stream(client_side);
    client.home().await.unwrap();
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

#[tokio::test]
async fn typed_helpers_surface_server_errors() {
    use ippdme_core::{response, IppError, Tag, Term};
    use ippdme_net::{serve_connection, NetError};

    let (client_side, server_side) = tokio::io::duplex(4096);
    let handler = |tag: Tag, _term: Term| async move { response::error(tag, "CollisionDetected") };
    tokio::spawn(async move { serve_connection(server_side, &handler).await });

    let client = IppClient::from_stream(client_side);
    match client.go_to(1.0, 2.0, 3.0).await {
        Err(NetError::Protocol(IppError::ServerError { reason })) => {
            assert_eq!(reason, "CollisionDetected")
        }
        other => panic!("expected a server error, got {other:?}"),
    }
}

#[tokio::test]
async fn is_homed_and_user_enabled_parse_flags() {
    let client = connected_client().await;
    assert!(!client.is_homed().await.unwrap());
    assert!(!client.is_user_enabled().await.unwrap());

    client.home().await.unwrap();
    client
        .send_command(ippdme_core::Command::EnableUser)
        .await
        .unwrap();
    assert!(client.is_homed().await.unwrap());
    assert!(client.is_user_enabled().await.unwrap());
}

#[tokio::test]
async fn tool_commands_are_handled_by_the_mock() {
    use ippdme_core::{Command, ToolAlignment, ToolName, UnitVector};

    let client = connected_client().await;
    let tool = ToolName::new("RefTool").unwrap();
    for cmd in [
        Command::Tool,
        Command::FindTool(tool.clone()),
        Command::FoundTool,
        Command::ChangeTool(tool.clone()),
        Command::SetTool(tool),
        Command::GoToPar,
        Command::PtMeasPar,
    ] {
        assert!(
            client.send_command(cmd.clone()).await.unwrap().is_ack(),
            "{cmd:?}"
        );
    }

    let names = client.send_command(Command::EnumTools).await.unwrap();
    assert!(names.is_data());
    assert_eq!(names.term().args().len(), 3);

    let up = UnitVector::new(0.0, 0.0, 1.0).unwrap();
    let alignment = ToolAlignment::primary(up, 5.0).unwrap();
    let reached = client
        .send_command(Command::AlignTool(alignment))
        .await
        .unwrap();
    assert!(reached.is_data());
}

#[tokio::test]
async fn send_line_sends_text_verbatim_and_returns_the_tagged_reply() {
    let client = connected_client().await;

    let reply = client.send_line("00042 StartSession()").await.unwrap();
    let reply = reply.expect("a tagged line waits for its reply");
    assert_eq!(reply.tag(), ippdme_core::Tag(42));
    assert_eq!(reply.term().name(), Some("Ready"));

    // A well-formed but unknown command: the server answers with an error.
    let reply = client.send_line("00043 Bogus()").await.unwrap().unwrap();
    assert!(reply.is_error());
}

#[tokio::test]
async fn send_line_without_a_tag_returns_immediately() {
    let client = connected_client().await;

    // No tag, so there is no reply to wait for. The mock can't parse the
    // line and ends the connection, so later requests fail.
    let started = std::time::Instant::now();
    assert!(client
        .send_line("hello, are you there?")
        .await
        .unwrap()
        .is_none());
    assert!(started.elapsed() < std::time::Duration::from_millis(500));
    assert!(client.get_dme_version().await.is_err());
}

#[tokio::test]
async fn send_line_rejects_embedded_line_breaks() {
    let client = connected_client().await;
    assert!(client
        .send_line("00001 Home()\r\n00002 EndSession()")
        .await
        .is_err());
}

#[tokio::test]
async fn dropping_the_client_closes_the_connection() {
    use tokio::io::AsyncReadExt;

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let client = IppClient::connect(addr).await.unwrap();
    let (mut server_side, _) = listener.accept().await.unwrap();

    drop(client);

    // The server end sees the stream finish, instead of waiting forever.
    let mut buf = [0u8; 16];
    let n = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        server_side.read(&mut buf),
    )
    .await
    .expect("connection should close when the client is dropped")
    .unwrap();
    assert_eq!(n, 0);
}
