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
