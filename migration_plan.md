# Plan: rewriting `ippdme` in C#

Status: draft, no code changes made.

## Size

- About 7,100 lines of Rust, plus about 1,000 lines of examples, Python and tests.
- The core work is a port, not a redesign. `openspec/specs/` has 13 specs written from the code and tests, so there is a behavioral contract to port against.
- Estimate: 3-5 weeks for one engineer, or 1-2 weeks with an AI-assisted loop. Most of the time goes to the Python bindings and the TUI, not the protocol.

## Component mapping

| Rust | LOC | C# equivalent | Difficulty |
|---|---|---|---|
| `ippdme-core`: AST, `values.rs`, `commands.rs`, `response.rs` | ~1,950 | `Ippdme.Core` class library. Records and sealed class hierarchies for `Term`/`Message`/`Command`, with pattern matching. | Easy. C# handles sum types well and the validated constructors translate directly. |
| `winnow` parser | 229 | A hand-written recursive-descent parser. Sprache or Superpower would work, but the grammar is small, so no dependency. | Easy |
| `ippdme-net`: codec, `IppClient`, `IppServer`, `IppMockServer`, `IppTap` | ~2,300 | `System.IO.Pipelines` for CRLF framing. `Channel<T>` replaces `broadcast`, and `TaskCompletionSource` replaces the `oneshot` map. `Interlocked` replaces `AtomicU32`. | Medium |
| TLS 1.3 and mTLS (rustls) | 157 | `SslStream` with `SslProtocols.Tls13`. | Easier than Rust, since it is built in. |
| `ippdme-imposter` (YAML stubs and a Rust builder) | ~1,000 | YamlDotNet and a fluent builder. | Easy |
| `ippdme-tui` (Ratatui) | ~1,460 | Terminal.Gui or Spectre.Console. | Medium-hard |
| `ippdme-py` (PyO3) | ~410 | See decision 1. | Hardest decision |
| CI, release, cross-compile | n/a | `dotnet publish`, with `-r linux-arm64` for the Raspberry Pi. | Easy |

## Decisions that change the plan

1. **Python bindings are the main problem.** `pip install ippdme` currently ships a native extension. C# has no equivalent of PyO3. Options:
   - Drop the Python package.
   - NativeAOT shared library plus a `ctypes`/`cffi` wrapper. Works, but means maintaining a C ABI and platform wheels.
   - Python wraps the imposter or client CLI over a subprocess or HTTP. Simpler, but changes the API.
   - Pythonnet. Needs the .NET runtime on the user's machine.

   The `ippdme.testing` fixtures depend on the in-process mock server, so this decision matters most.

2. **Deployment changes.** C# is memory-safe too, but deployment goes from a static binary to either a runtime-dependent app or a NativeAOT binary of roughly 10-30 MB. NativeAOT works for the core, client and server, but YamlDotNet needs source-gen or trimming care. Raspberry Pi works via `linux-arm64`.

3. **The TUI is the least mechanical port.** Ratatui is immediate-mode and the app is built on a `select!` loop. Terminal.Gui is retained-mode, so the structure changes. Spectre.Console is closer to immediate-mode but has weaker interaction. Tap mode, raw-line mode and presets all need re-checking by hand.

4. **Concurrency idioms shift.** The Rust design avoids blocking the event loop by spawning tasks that report back via `AppEvent`. In C#, `async/await` plus `Channel<T>` carries the same pattern, so the discipline stays and the implementation changes. Rust's `Send` bounds, documented for `ippdme-py`, go away.

5. **Repo shape.** One `.sln` with `Ippdme.Core`, `Ippdme.Net`, `Ippdme.Imposter`, `Ippdme.Tui` and xUnit test projects. CLAUDE.md, CI, `Cargo.toml` and the "Build, test, lint" section all need rewriting. `dotnet format` and analyzers with warnings-as-errors replace fmt and clippy.

## Suggested phases

1. **Decide the open questions** (Python, AOT vs runtime, TUI library). Write them as an OpenSpec change, since CLAUDE.md requires propose-first for behavior changes. The specs stay the same and only the implementation language changes.
2. **Core.** Port `ast`, `values`, `commands`, `response` and the parser. Port the existing Rust unit tests alongside, and add round-trip tests against the same wire fixtures.
3. **Net.** Port the codec, client, server, mock and tap, then TLS. Run the existing integration tests, translated, as the acceptance bar.
4. **Imposter.** Port the config, stub engine and CLI. The `examples/*.yaml` files must parse unchanged; they are a free compatibility test.
5. **TUI.**
6. **Python or other bindings**, per decision 1.
7. **CI and packaging**, plus a README and CLAUDE.md rewrite.

Interim parity check: run the C# client against the Rust mock server, and the reverse. Both speak the same wire protocol, so this verifies parity before anything is deleted.

## Risks

- **Behavioral drift.** The mock's latencies, strict mode and per-connection session state are easy to get subtly wrong. The specs and the Rust integration tests are the guardrails.
- **Loss of the Rust-specific value proposition** (static binary, no runtime, a small attack surface for an industrial tool). Check that this fits how the project is used.
- **Python users** are the most likely to be broken.

## Open questions

1. What is the goal: a new implementation alongside the Rust one, or a replacement? A side-by-side port allows cross-testing and is recommended until parity.
2. What happens to the Python package: drop, rebuild via NativeAOT, or wrap?
3. Is a runtime-dependent deployment acceptable, or are NativeAOT single-file binaries required?
4. For the TUI, keep the Ratatui feel (Spectre.Console) or use a full widget toolkit (Terminal.Gui)?

Next step once these are answered: draft the OpenSpec change (`/opsx:propose`) with the design and task list.
