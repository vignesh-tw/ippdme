"""ippdme: a modern, memory-safe client and mock server for the I++ DME
protocol used to command Coordinate Measuring Machines (CMMs).

    from ippdme import IppClient, IppMockServer

    server = IppMockServer(port=1294)
    server.start_in_background()

    client = IppClient.connect("127.0.0.1:1294")
    client.start_session()
    response = client.go_to(x=10.0, y=20.0, z=5.0)
    print(response.is_ack())
"""

from ._ippdme import IppClient, IppMockServer, Response

__all__ = ["IppClient", "IppMockServer", "Response"]
