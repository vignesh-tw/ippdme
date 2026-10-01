"""Talk to an I++ DME imposter over TLS 1.3 (optionally mutual TLS).

    sh examples/tls/make-certs.sh
    python examples/python/tls_client.py                      # TLS, port 1295
    python examples/python/tls_client.py --mtls               # mutual TLS, port 1296

Needs the Python package: `maturin develop -m crates/ippdme-py/Cargo.toml`.
"""

import argparse

from ippdme import IppClient

CERTS = "examples/tls/certs"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--mtls", action="store_true", help="present a client certificate")
    parser.add_argument("--addr", help="host:port (default 127.0.0.1:1295, or :1296 with --mtls)")
    args = parser.parse_args()

    addr = args.addr or ("127.0.0.1:1296" if args.mtls else "127.0.0.1:1295")
    options = {"ca_cert": f"{CERTS}/ca.pem", "server_name": "localhost"}
    if args.mtls:
        options.update(client_cert=f"{CERTS}/client.pem", client_key=f"{CERTS}/client.key")

    client = IppClient.connect(addr, **options)
    print(f"connected to {addr} over TLS" + (" (mutual)" if args.mtls else ""))
    print("StartSession ->", client.start_session())
    print("GetDMEVersion ->", client.get_dme_version())
    print("PtMeas ->", client.pt_meas())
    print("EndSession ->", client.end_session())


if __name__ == "__main__":
    main()
