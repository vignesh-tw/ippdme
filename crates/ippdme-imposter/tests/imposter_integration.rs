use ippdme_core::Term;
use ippdme_imposter::{Imposter, Predicate, ResponseSpec, Stub};
use ippdme_net::IppClient;
use tokio::net::TcpStream;

async fn client_for(imposter: Imposter) -> (IppClient, std::net::SocketAddr) {
    let addr = imposter.local_addr().unwrap();
    tokio::spawn(imposter.serve());
    let stream = TcpStream::connect(addr).await.unwrap();
    (IppClient::from_stream(stream), addr)
}

#[tokio::test]
async fn yaml_stub_replies_with_configured_data() {
    let yaml = r#"
port: 0
stubs:
  - predicate:
      call: PtMeas
    responses:
      - data:
          call: PtMeas
          args:
            - { name: X, value: 10.002 }
            - { name: Y, value: 20.001 }
"#;
    let imposter = Imposter::from_yaml_str(yaml).await.unwrap();
    let (client, _addr) = client_for(imposter).await;

    let resp = client.pt_meas().await.unwrap();
    assert!(resp.is_data());
    assert_eq!(resp.term().get_num_param("X"), Some(10.002));
    assert_eq!(resp.term().get_num_param("Y"), Some(20.001));
}

#[tokio::test]
async fn yaml_start_session_acks_with_ready_term() {
    let yaml = r#"
port: 0
stubs:
  - predicate:
      call: StartSession
    responses:
      - ack: Ready
"#;
    let imposter = Imposter::from_yaml_str(yaml).await.unwrap();
    let (client, _addr) = client_for(imposter).await;

    let resp = client.start_session().await.unwrap();
    assert!(resp.is_ack());
    assert_eq!(resp.term().name(), Some("Ready"));
}

#[tokio::test]
async fn yaml_predicate_matches_exact_args_only() {
    let yaml = r#"
port: 0
stubs:
  - predicate:
      call: GetErrorInfo
      args: [42]
    responses:
      - data:
          call: GetErrorInfo
          args:
            - { str: "Collision detected" }
"#;
    let imposter = Imposter::from_yaml_str(yaml).await.unwrap();
    let (client, _addr) = client_for(imposter).await;

    // Matching args: should get the configured stub response.
    let resp = client
        .send(Term::call("GetErrorInfo", vec![Term::Number(42.0)]))
        .await
        .unwrap();
    assert!(resp.is_data());

    // Non-matching args: no stub applies, so it falls back to an error.
    let resp = client
        .send(Term::call("GetErrorInfo", vec![Term::Number(1.0)]))
        .await
        .unwrap();
    assert!(resp.is_error());
}

#[tokio::test]
async fn stub_sequence_sticks_on_last_response() {
    let imposter = Imposter::builder()
        .stub(
            Stub::when(Predicate::call("GoTo"))
                .responds_with(ResponseSpec::Error("CollisionDetected".into()))
                .responds_with(ResponseSpec::ack()),
        )
        .bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let (client, _addr) = client_for(imposter).await;

    let first = client.go_to(1.0, 2.0, 3.0).await.unwrap();
    assert!(first.is_error());

    let second = client.go_to(1.0, 2.0, 3.0).await.unwrap();
    assert!(second.is_ack());

    // Sequence exhausted: keeps acking.
    let third = client.go_to(1.0, 2.0, 3.0).await.unwrap();
    assert!(third.is_ack());
}

#[tokio::test]
async fn received_calls_records_what_the_client_sent() {
    let imposter = Imposter::builder()
        .stub(Stub::when(Predicate::call("Home")).responds_with(ResponseSpec::ack()))
        .bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let addr = imposter.local_addr().unwrap();
    let log = imposter.handle();
    tokio::spawn(imposter.serve());

    let stream = TcpStream::connect(addr).await.unwrap();
    let client = IppClient::from_stream(stream);
    client.home().await.unwrap();

    let received = log.received_calls();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].name(), Some("Home"));
}
