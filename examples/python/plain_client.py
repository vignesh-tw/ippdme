"""Talk to an I++ DME machine (or an imposter) over plain TCP.

    python examples/python/plain_client.py [host:port]      # default 127.0.0.1:1294

Needs the Python package: `maturin develop -m crates/ippdme-py/Cargo.toml`.
"""

import sys

from ippdme import IppClient


def main() -> None:
    addr = sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1:1294"
    client = IppClient.connect(addr)
    print(f"connected to {addr}")

    print("StartSession ->", client.start_session())
    print("GetDMEVersion ->", client.get_dme_version())
    print("Home ->", client.home())
    print("GoTo(10, 20, 5) ->", client.go_to(x=10.0, y=20.0, z=5.0))

    measured = client.pt_meas()
    print("PtMeas ->", measured)
    if measured.is_data():
        print(f"  X={measured.get('X')} Y={measured.get('Y')} Z={measured.get('Z')}")

    # A reply can be an error; check the marker rather than assuming success.
    reply = client.send_raw("GetErrorInfo(42)")
    print("GetErrorInfo(42) ->", reply)

    print("EndSession ->", client.end_session())


if __name__ == "__main__":
    main()
