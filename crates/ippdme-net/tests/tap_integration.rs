use ippdme_net::{IppClient, IppMockServer, IppTap, MockConfig, TapDirection, TapEvent};
use tokio::sync::broadcast::Receiver;

async fn next_line(events: &mut Receiver<TapEvent>) -> (TapDirection, String) {
    loop {
        if let TapEvent::Line {
            direction, line, ..
        } = events.recv().await.unwrap()
        {
            return (direction, line);
        }
    }
}

#[tokio::test]
async fn tap_forwards_traffic_and_reports_each_line() {
    let mock = IppMockServer::builder()
        .config(MockConfig::instant())
        .bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let mock_addr = mock.local_addr().unwrap();
    tokio::spawn(mock.serve());

    let tap = IppTap::bind(("127.0.0.1", 0), mock_addr.to_string())
        .await
        .unwrap();
    let tap_addr = tap.local_addr().unwrap();
    let mut events = tap.subscribe();
    tokio::spawn(tap.serve());

    // The client talks to the tap, unaware it isn't the machine.
    let client = IppClient::connect(tap_addr).await.unwrap();
    client.start_session().await.unwrap();
    assert_eq!(client.get_dme_version().await.unwrap(), "1.4");

    assert!(matches!(
        events.recv().await.unwrap(),
        TapEvent::Opened { conn: 1, .. }
    ));
    assert_eq!(
        next_line(&mut events).await,
        (TapDirection::ClientToServer, "00001 StartSession()".into())
    );
    assert_eq!(
        next_line(&mut events).await,
        (TapDirection::ServerToClient, "00001 # Ready()".into())
    );
    assert_eq!(
        next_line(&mut events).await,
        (TapDirection::ClientToServer, "00002 GetDMEVersion()".into())
    );
    assert_eq!(
        next_line(&mut events).await,
        (
            TapDirection::ServerToClient,
            "00002 % DMEVersion(\"1.4\")".into()
        )
    );
}

#[tokio::test]
async fn tap_reports_close_when_the_client_disconnects() {
    let mock = IppMockServer::bind(("127.0.0.1", 0)).await.unwrap();
    let mock_addr = mock.local_addr().unwrap();
    tokio::spawn(mock.serve());
    let tap = IppTap::bind(("127.0.0.1", 0), mock_addr.to_string())
        .await
        .unwrap();
    let tap_addr = tap.local_addr().unwrap();
    let mut events = tap.subscribe();
    tokio::spawn(tap.serve());

    drop(tokio::net::TcpStream::connect(tap_addr).await.unwrap());

    assert!(matches!(
        events.recv().await.unwrap(),
        TapEvent::Opened { .. }
    ));
    assert!(matches!(
        events.recv().await.unwrap(),
        TapEvent::Closed { conn: 1 }
    ));
}

#[tokio::test]
async fn tap_to_an_unreachable_target_closes_the_client() {
    // Nothing listens on the target port.
    let unused = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let dead = unused.local_addr().unwrap();
    drop(unused);

    let tap = IppTap::bind(("127.0.0.1", 0), dead.to_string())
        .await
        .unwrap();
    let tap_addr = tap.local_addr().unwrap();
    tokio::spawn(tap.serve());

    let mut client = IppClient::connect(tap_addr).await.unwrap();
    client.set_default_timeout(std::time::Duration::from_secs(2));
    assert!(client.home().await.is_err());
}
