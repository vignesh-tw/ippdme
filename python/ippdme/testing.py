"""Pytest fixtures for Hardware-in-the-Loop (HIL) style CI pipelines.

    # conftest.py
    pytest_plugins = ["ippdme.testing"]

    def test_go_to(ipp_client):
        response = ipp_client.go_to(x=10.0, y=20.0, z=5.0)
        assert response.is_ack()
"""

import pytest

from ._ippdme import IppClient, IppMockServer


@pytest.fixture
def mock_server():
    """A running embedded mock CMM on an ephemeral port, stopped at teardown."""
    server = IppMockServer(port=0)
    server.start_in_background()
    yield server
    server.stop()


@pytest.fixture
def ipp_client(mock_server):
    """An `IppClient` connected to `mock_server`, with a session already
    started; the session is ended at teardown."""
    client = IppClient.connect(f"127.0.0.1:{mock_server.port}")
    client.start_session()
    yield client
    client.end_session()
