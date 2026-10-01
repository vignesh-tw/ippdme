# Examples: talk to a virtual CMM on your own machine

Everything here runs on one PC, over `127.0.0.1`. An **imposter** plays the
CMM, and you talk to it with whichever client you like. Run every command from
the **repository root**.

| Folder | What it is |
|---|---|
| `plain/imposter.yaml` | A well-behaved machine on port **1294** |
| `plain/faults.yaml` | A misbehaving machine (collision, slow reply, dropped connection, garbage) on port **1294** |
| `tls/` | Certificate script, plus TLS (**1295**) and mutual TLS (**1296**) machines |
| `python/` | Python clients (`plain_client.py`, `tls_client.py`) |
| `rust-client/` | A Rust application built on `ippdme-core` and `ippdme-net` |

## 1. Start a machine

```bash
cargo run -p ippdme-imposter --bin ippdme-imposter -- examples/plain/imposter.yaml
# ippdme-imposter listening on 127.0.0.1:1294 (examples/plain/imposter.yaml)
```

Leave it running and open a second terminal for the clients below. Stop it with
Ctrl-C. (You can run it with `faults.yaml` instead; both use port 1294, so run
one at a time.)

## 2. Talk to it

### With `nc` (no code at all)

I++ DME is lines of text over TCP, so `nc` (netcat) is enough:

```
$ nc 127.0.0.1 1294
00001 StartSession()
00001 # Ready()
00002 GetDMEVersion()
00002 % DMEVersion("1.5")
00003 PtMeas()
00003 % PtMeas(X(10.002), Y(20.001), Z(5.0), I(0.0), J(0.0), K(1.0))
00004 Bogus()
00004 ! Error(UnknownStub)
```

You type the first line of each pair; the machine answers with the second. Each
line starts with a five-digit **tag**. The reply carries the same tag, and a
marker: `#` acknowledged, `%` data, `!` error. Quit with Ctrl-C.

### With the Python client

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install maturin && maturin develop -m crates/ippdme-py/Cargo.toml
python examples/python/plain_client.py
```

### With the Rust client

```bash
cargo run -p ippdme-example-client
```

It runs a typed session (`start_session`, `home`, `go_to`, `pt_meas`, ...) and
prints each result, showing a server `Error(..)` reply separately from a
transport failure. The source is `examples/rust-client/src/main.rs`.

### With the TUI

```bash
cargo run -p ippdme-tui
```

Press `c` to connect to `127.0.0.1:1294`, pick a preset in the sidebar and press
Enter. Press `Tab` to focus the input box and type a term such as
`GoTo(X(10.0), Y(20.0), Z(5.0))`; the TUI adds the tag.

**Raw line input.** Press `r` to make the input box behave like `nc`: whatever
you type is sent exactly as typed, tag included, with no parsing. A line that
starts with a tag (`00042 StartSession()`) shows its reply in the stream; a line
without one is sent without waiting. Use it to send malformed input and see how
a server reacts. Press `r` again to go back to parsed terms.

## 3. Watch what is on the port

The client and the machine each show their own view. To see the lines **as they
cross the port**, from outside both, put a *tap* between them. The tap forwards
every byte unchanged and prints each line in each direction.

```
client  ->  tap (port 1297)  ->  machine (port 1294)
```

### With `ippdme-tap`

In a second terminal (the imposter from step 1 keeps running):

```bash
cargo run -p ippdme-net --bin ippdme-tap -- --target 127.0.0.1:1294
# ippdme-tap: 127.0.0.1:1297 -> 127.0.0.1:1294 (point the client at 127.0.0.1:1297)
```

Point any client at the tap's port (**1297**) instead of 1294:

```bash
python examples/python/plain_client.py 127.0.0.1:1297
```

The tap prints:

```
[#1] opened from 127.0.0.1:54158
[#1] client -> server  00001 StartSession()
[#1] server -> client  00001 # Ready()
[#1] client -> server  00002 GetDMEVersion()
[#1] server -> client  00002 % DMEVersion("1.5")
...
[#1] closed
```

Neither side knows the tap is there.

### With the TUI's Tap mode

Press `m` until the mode reads **Tap**. The target in the connection bar is the
machine to forward to (`h` and `p` edit it; default `127.0.0.1:1294`), and
clients connect to port 1297 (`--tap-listen PORT` changes that). Press `c` to
start the tap, then run any client against port 1297. The stream shows each line
exactly as it crossed the port: `-->` client to server, `<--` server to client,
colored by marker. Tap mode only watches; it cannot send.

### With `tcpdump`

`tcpdump` shows the same thing from the operating system, with no tap in the
path. It needs `sudo`, and works on plain ports only:

```bash
sudo tcpdump -i lo0 -A -s0 'tcp port 1294'     # macOS (use -i lo on Linux)
```

## 4. Try the failures

Stop the imposter, then start the misbehaving one:

```bash
cargo run -p ippdme-imposter --bin ippdme-imposter -- examples/plain/faults.yaml
cargo run -p ippdme-example-client
```

| Command | What the machine does | What the client sees |
|---|---|---|
| `GoTo(...)` the first time | replies `Error(CollisionDetected)` | `server said Error(CollisionDetected)`. The second `GoTo` succeeds. |
| `PtMeas()` | answers after 8 seconds | `request timed out`, because the client waits 5 seconds by default |
| `IsHomed()` | closes the connection | `connection closed` |
| `GetMachineClass()` | sends `this is not an I++ line` | the client cannot parse it and its connection ends |

Watch it through the tap (step 3) to see exactly what went over the wire, or
send the commands yourself with `nc`.

## 5. TLS

A real CMM won't speak TLS itself, but our libraries can, which is useful for
securing the link to a gateway or proxy. First make throwaway certificates:

```bash
sh examples/tls/make-certs.sh        # writes examples/tls/certs/ (git-ignored)
```

**TLS** (port 1295): the client checks the server's certificate.

```bash
cargo run -p ippdme-imposter --bin ippdme-imposter -- examples/tls/imposter-tls.yaml
python examples/python/tls_client.py
cargo run -p ippdme-example-client -- --addr 127.0.0.1:1295 \
    --ca-cert examples/tls/certs/ca.pem --server-name localhost
```

For the TUI:
`cargo run -p ippdme-tui -- --ca-cert examples/tls/certs/ca.pem --server-name localhost`,
then set the port to 1295 with `p`. (The example certificate is issued for
`localhost`, so the name must be given; the TUI would otherwise check against
`127.0.0.1`.)

**Mutual TLS** (port 1296): the machine also requires the client to present a
certificate.

```bash
cargo run -p ippdme-imposter --bin ippdme-imposter -- examples/tls/imposter-mtls.yaml
python examples/python/tls_client.py --mtls
```

What you will see when it goes wrong:

- A **plain** client on a TLS port: the machine can't read it and the client times
  out.
- A **TLS** client on the plain port (1294): the connection fails or times out
  within 5 seconds. There is no fallback to plaintext.
- Mutual TLS without a client certificate: the connection is refused.

`nc` and the tap can't read TLS: the content is encrypted, so you'd see unreadable
bytes (use the client's own log, or a TLS-terminating proxy, to see the lines).

## Using these as templates

The YAML files are the starting point for your own machine:
[`plain/imposter.yaml`](plain/imposter.yaml) is commented, and each `stubs:` entry
is "when this call arrives, reply like this". `after_ms` adds delay,
`responses` lists replay in order, and `drop` / `malformed` inject faults.
Every YAML under `examples/` is parsed by a test, so they stay valid.
