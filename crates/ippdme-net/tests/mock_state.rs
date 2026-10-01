use ippdme_core::{response, Command, CoordSystem, CoordSystemName, IppError, ToolName};
use ippdme_net::mock::spawn_ephemeral_with;
use ippdme_net::{IppClient, MockConfig, NetError};

async fn client(config: MockConfig) -> IppClient {
    let (addr, _server) = spawn_ephemeral_with(config).await.unwrap();
    IppClient::connect(addr).await.unwrap()
}

fn reason(err: NetError) -> String {
    match err {
        NetError::Protocol(IppError::ServerError { reason }) => reason,
        other => panic!("expected a server error, got {other:?}"),
    }
}

async fn error_reason(c: &IppClient, cmd: Command) -> String {
    let reply = c.send_command(cmd).await.unwrap();
    reason(response::expect_ack(&reply).unwrap_err().into())
}

#[tokio::test]
async fn instant_config_skips_movement_latency() {
    let c = client(MockConfig::instant()).await;
    let start = std::time::Instant::now();
    c.home().await.unwrap();
    c.go_to(1.0, 2.0, 3.0).await.unwrap();
    assert!(start.elapsed() < std::time::Duration::from_millis(200));
}

#[tokio::test]
async fn go_to_moves_the_machine_and_pt_meas_reports_it() {
    let c = client(MockConfig::instant()).await;
    c.go_to(1.0, 2.0, 3.0).await.unwrap();
    let p = c.pt_meas().await.unwrap();
    assert_eq!((p.x(), p.y(), p.z()), (Some(1.0), Some(2.0), Some(3.0)));
}

#[tokio::test]
async fn coordinate_system_and_tool_state_is_remembered() {
    let c = client(MockConfig::instant()).await;
    assert_eq!(
        c.send_command(Command::GetCoordSystem)
            .await
            .unwrap()
            .term()
            .args()[0]
            .to_string(),
        "PartCsy"
    );
    c.set_coord_system(CoordSystem::Mcs).await.unwrap();
    assert_eq!(
        c.send_command(Command::GetCoordSystem)
            .await
            .unwrap()
            .term()
            .args()[0]
            .to_string(),
        "MachineCsy"
    );

    let known = ToolName::new("RefTool").unwrap();
    let unknown = ToolName::new("Nope").unwrap();
    assert!(response::expect_ack(&c.send_command(Command::SetTool(known)).await.unwrap()).is_ok());
    assert_eq!(
        error_reason(&c, Command::SetTool(unknown.clone())).await,
        "ToolNotFound"
    );
    assert_eq!(
        error_reason(&c, Command::FindTool(unknown)).await,
        "ToolNotFound"
    );
    assert_eq!(error_reason(&c, Command::FoundTool).await, "ToolNotDefined");
}

#[tokio::test]
async fn saved_coordinate_systems_can_be_listed_loaded_and_deleted() {
    let c = client(MockConfig::instant()).await;
    let name = CoordSystemName::new("Fixture1").unwrap();

    assert_eq!(
        error_reason(&c, Command::LoadCoordSystem(name.clone())).await,
        "CoordSystemNotFound"
    );

    c.send_command(Command::SaveActiveCoordSystem(name.clone()))
        .await
        .unwrap();
    let listed = c.send_command(Command::EnumCoordSystems).await.unwrap();
    assert_eq!(listed.term().args().len(), 1);
    assert!(response::expect_ack(
        &c.send_command(Command::LoadCoordSystem(name.clone()))
            .await
            .unwrap()
    )
    .is_ok());

    c.send_command(Command::DeleteCoordSystem(name.clone()))
        .await
        .unwrap();
    assert_eq!(
        error_reason(&c, Command::DeleteCoordSystem(name)).await,
        "CoordSystemNotFound"
    );
}

#[tokio::test]
async fn each_connection_is_its_own_session() {
    let (addr, _server) = spawn_ephemeral_with(MockConfig::instant()).await.unwrap();
    let a = IppClient::connect(addr).await.unwrap();
    let b = IppClient::connect(addr).await.unwrap();
    a.home().await.unwrap();
    assert!(a.is_homed().await.unwrap());
    assert!(!b.is_homed().await.unwrap());
}

#[tokio::test]
async fn strict_mode_enforces_call_order() {
    let c = client(MockConfig::instant().strict()).await;

    assert_eq!(error_reason(&c, Command::Home).await, "NoSession");
    assert!(c.get_dme_version().await.is_ok());

    c.start_session().await.unwrap();
    assert_eq!(error_reason(&c, Command::Home).await, "UserNotEnabled");

    c.send_command(Command::EnableUser).await.unwrap();
    assert_eq!(
        error_reason(&c, Command::go_to(1.0, 2.0, 3.0).unwrap()).await,
        "NotHomed"
    );

    c.home().await.unwrap();
    c.go_to(1.0, 2.0, 3.0).await.unwrap();
    c.pt_meas().await.unwrap();

    c.end_session().await.unwrap();
    assert_eq!(error_reason(&c, Command::pt_meas()).await, "NoSession");
}

#[tokio::test]
async fn lenient_mode_allows_any_order() {
    let c = client(MockConfig::instant()).await;
    c.go_to(1.0, 2.0, 3.0).await.unwrap();
}
