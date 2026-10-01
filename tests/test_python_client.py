import pytest

from ippdme import IppClient, IppMockServer


def test_start_session_is_ack(ipp_client):
    # ipp_client fixture already started the session; do it again to check
    # the response shape directly.
    response = ipp_client.start_session()
    assert response.is_ack()
    assert response.tag > 0


def test_go_to_acks(ipp_client):
    response = ipp_client.go_to(x=10.0, y=20.0, z=5.0)
    assert response.is_ack()


def test_pt_meas_returns_data(ipp_client):
    response = ipp_client.pt_meas()
    assert response.is_data()
    assert response.name == "PtMeas"
    assert response.get("X") == pytest.approx(10.002)
    assert response.get("K") == pytest.approx(1.0)


def test_set_coord_system_accepts_mcs_and_pcs(ipp_client):
    assert ipp_client.set_coord_system("MCS").is_ack()
    assert ipp_client.set_coord_system("pcs").is_ack()


def test_set_coord_system_rejects_unknown(ipp_client):
    with pytest.raises(ValueError):
        ipp_client.set_coord_system("BOGUS")


def test_unknown_command_gets_error_with_reason(ipp_client):
    response = ipp_client.send_raw("TotallyMadeUpCommand()")
    assert response.is_error()
    assert response.reason == "UnknownCommand"


def test_mock_server_reports_bound_port():
    server = IppMockServer(port=0)
    server.start_in_background()
    try:
        assert server.port != 0
        client = IppClient.connect(f"127.0.0.1:{server.port}")
        assert client.start_session().is_ack()
    finally:
        server.stop()


def test_go_to_takes_roughly_the_simulated_latency(ipp_client):
    import time

    start = time.monotonic()
    ipp_client.go_to(x=1.0, y=1.0, z=1.0)
    assert time.monotonic() - start >= 0.45


def test_go_to_rejects_non_finite_coordinates(ipp_client):
    with pytest.raises(ValueError):
        ipp_client.go_to(x=float("nan"), y=0.0, z=0.0)


def _server(**kwargs):
    server = IppMockServer(port=0, latency_ms=0, **kwargs)
    server.start_in_background()
    return server


def test_mock_is_stateful_per_connection():
    server = _server()
    try:
        client = IppClient.connect(f"127.0.0.1:{server.port}")
        assert "IsHomed(0.0)" in str(client.send_raw("IsHomed()"))
        client.home()
        assert "IsHomed(1.0)" in str(client.send_raw("IsHomed()"))
        client.go_to(x=1.0, y=2.0, z=3.0)
        assert client.pt_meas().get("X") == pytest.approx(1.0)
    finally:
        server.stop()


def test_strict_mock_rejects_out_of_order_commands():
    server = _server(strict=True)
    try:
        client = IppClient.connect(f"127.0.0.1:{server.port}")
        assert client.home().is_error()  # no session yet
        client.start_session()
        assert client.home().is_error()  # user not enabled
        client.send_raw("EnableUser()")
        assert client.go_to(x=1.0, y=1.0, z=1.0).is_error()  # not homed
        assert client.home().is_ack()
        assert client.go_to(x=1.0, y=1.0, z=1.0).is_ack()
    finally:
        server.stop()
