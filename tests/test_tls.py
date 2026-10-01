import shutil
import subprocess

import pytest

from ippdme import IppClient, IppMockServer

pytestmark = pytest.mark.skipif(
    shutil.which("openssl") is None, reason="openssl CLI is required to make test certs"
)


def _run(*args, cwd):
    subprocess.run(args, cwd=cwd, check=True, capture_output=True)


def _make_ca(d, name):
    _run(
        "openssl", "req", "-x509", "-newkey", "rsa:2048",
        "-nodes", "-subj", f"/CN={name}", "-days", "2",
        "-keyout", f"{name}.key", "-out", f"{name}.pem",
        cwd=d,
    )  # fmt: skip


def _issue(d, ca, name):
    (d / f"{name}.ext").write_text(f"subjectAltName=DNS:{name}\n")
    _run(
        "openssl", "req", "-newkey", "rsa:2048",
        "-nodes", "-subj", f"/CN={name}", "-keyout", f"{name}.key", "-out", f"{name}.csr",
        cwd=d,
    )  # fmt: skip
    _run(
        "openssl", "x509", "-req", "-in", f"{name}.csr", "-CA", f"{ca}.pem",
        "-CAkey", f"{ca}.key", "-CAcreateserial", "-days", "2",
        "-extfile", f"{name}.ext", "-out", f"{name}.pem",
        cwd=d,
    )  # fmt: skip


@pytest.fixture(scope="module")
def pki(tmp_path_factory):
    d = tmp_path_factory.mktemp("pki")
    _make_ca(d, "ca")
    _make_ca(d, "other-ca")
    _issue(d, "ca", "localhost")
    _issue(d, "ca", "client")
    return {name: str(d / name) for name in ("ca", "other-ca", "localhost", "client")}


@pytest.fixture
def tls_server(pki):
    server = IppMockServer(
        port=0, cert=pki["localhost"] + ".pem", key=pki["localhost"] + ".key"
    )
    server.start_in_background()
    yield server
    server.stop()


def test_client_talks_to_tls_server(tls_server, pki):
    client = IppClient.connect(
        f"127.0.0.1:{tls_server.port}",
        ca_cert=pki["ca"] + ".pem",
        server_name="localhost",
    )
    assert client.get_dme_version().is_data()


def test_untrusted_server_is_rejected(tls_server, pki):
    with pytest.raises(ConnectionError):
        IppClient.connect(
            f"127.0.0.1:{tls_server.port}",
            ca_cert=pki["other-ca"] + ".pem",
            server_name="localhost",
        )


def test_plain_client_cannot_talk_to_tls_server(tls_server):
    with pytest.raises((ConnectionError, TimeoutError)):
        IppClient.connect(f"127.0.0.1:{tls_server.port}").get_dme_version()


def test_mutual_tls(pki):
    server = IppMockServer(
        port=0,
        cert=pki["localhost"] + ".pem",
        key=pki["localhost"] + ".key",
        client_ca=pki["ca"] + ".pem",
    )
    server.start_in_background()
    try:
        client = IppClient.connect(
            f"127.0.0.1:{server.port}",
            ca_cert=pki["ca"] + ".pem",
            server_name="localhost",
            client_cert=pki["client"] + ".pem",
            client_key=pki["client"] + ".key",
        )
        assert client.get_dme_version().is_data()
    finally:
        server.stop()


def test_tls_options_are_validated(pki):
    with pytest.raises(ValueError):
        IppClient.connect("127.0.0.1:1", server_name="localhost")
    with pytest.raises(ValueError):
        IppClient.connect(
            "127.0.0.1:1", ca_cert=pki["ca"] + ".pem", client_cert=pki["client"] + ".pem"
        )
    with pytest.raises(ValueError):
        IppMockServer(port=0, cert=pki["localhost"] + ".pem")
