use std::fs;
use std::io::Write as _;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ippdme_core::{leading_tag, parse_term_str, Command, Message, Tag};
use ippdme_net::{IppClient, IppMockServer, IppTap, NetError, TapDirection, TapEvent};

use crate::tls::TlsOptions;
use serde::Serialize;
use tokio::sync::mpsc;

use crate::presets::{default_presets, flatten, PresetCategory};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Client,
    MockServer,
    /// Sit between a client and a server and show the raw lines crossing
    /// the port, without taking part in the conversation.
    Tap,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnState {
    Disconnected,
    Connected,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    Sidebar,
    Log,
    Input,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Out,
    In,
}

#[derive(Clone, Serialize)]
pub struct LogEntry {
    #[serde(skip)]
    pub direction: Direction,
    pub tag: Option<u32>,
    pub marker: Option<char>,
    pub text: String,
    pub unix_ms: u128,
    pub latency_ms: Option<u128>,
    /// A line exactly as seen on the wire (tap mode), shown without the
    /// separate tag column.
    #[serde(skip)]
    pub wire: bool,
}

/// Results reported back to the UI loop from background network tasks.
pub enum AppEvent {
    CommandResult {
        tag: Tag,
        started: Instant,
        result: Result<Message, NetError>,
    },
    Connected(Arc<IppClient>),
    ConnectFailed(String),
    /// A raw line without a tag went out; there is no reply to wait for.
    RawLineSent,
    MockServerStarted {
        port: u16,
        handle: tokio::task::JoinHandle<ippdme_net::Result<()>>,
    },
    MockServerFailed(String),
    TapStarted {
        port: u16,
        handle: tokio::task::JoinHandle<ippdme_net::Result<()>>,
    },
    TapFailed(String),
    Tap(TapEvent),
}

pub struct App {
    pub tls: Option<TlsOptions>,
    /// The port the tap accepts clients on (tap mode).
    pub tap_listen: u16,
    pub host: String,
    pub port: String,
    pub mode: Mode,
    pub conn: ConnState,
    pub client: Option<Arc<IppClient>>,
    pub mock_handle: Option<tokio::task::JoinHandle<ippdme_net::Result<()>>>,
    pub presets: Vec<PresetCategory>,
    pub selected_preset: usize,
    pub log: Vec<LogEntry>,
    pub selected_log: Option<usize>,
    pub input: String,
    /// When set, the input box sends whole lines exactly as typed (tag
    /// included, no parsing) instead of parsing a term.
    pub raw_line_mode: bool,
    pub focus: Focus,
    pub editing_addr: Option<AddrField>,
    pub status: String,
    pub should_quit: bool,
    pub connecting: bool,
    pub app_tx: mpsc::UnboundedSender<AppEvent>,
    pub app_rx: mpsc::UnboundedReceiver<AppEvent>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AddrField {
    Host,
    Port,
}

impl App {
    pub fn new(tls: Option<TlsOptions>, tap_listen: u16) -> Self {
        let (app_tx, app_rx) = mpsc::unbounded_channel();
        App {
            tls,
            tap_listen,
            host: "127.0.0.1".to_string(),
            port: "1294".to_string(),
            mode: Mode::Client,
            conn: ConnState::Disconnected,
            client: None,
            mock_handle: None,
            presets: default_presets(),
            selected_preset: 0,
            log: Vec::new(),
            selected_log: None,
            input: String::new(),
            raw_line_mode: false,
            focus: Focus::Sidebar,
            editing_addr: None,
            status: "Disconnected".to_string(),
            should_quit: false,
            connecting: false,
            app_tx,
            app_rx,
        }
    }

    fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn now_ms() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    }

    pub fn push_out(&mut self, tag: Tag, text: String) {
        self.log.push(LogEntry {
            direction: Direction::Out,
            tag: Some(tag.0),
            marker: None,
            text,
            unix_ms: Self::now_ms(),
            latency_ms: None,
            wire: false,
        });
    }

    /// Record a send attempt that was rejected locally (not connected yet, or
    /// a parse error) so it's visible in the log instead of silently
    /// vanishing behind a status-bar message the user may not see in time.
    fn push_denied(&mut self, reason: impl Into<String>) {
        let reason = reason.into();
        self.status = reason.clone();
        self.log.push(LogEntry {
            direction: Direction::Out,
            tag: None,
            marker: Some('!'),
            text: format!("<{reason}>"),
            unix_ms: Self::now_ms(),
            latency_ms: None,
            wire: false,
        });
    }

    fn push_result(&mut self, tag: Tag, started: Instant, result: Result<Message, NetError>) {
        let latency_ms = Some(started.elapsed().as_millis());
        match result {
            Ok(msg) => {
                let marker = match &msg {
                    Message::Response { marker, .. } => Some(marker.as_char()),
                    Message::Command { .. } => None,
                };
                self.log.push(LogEntry {
                    direction: Direction::In,
                    tag: Some(msg.tag().0),
                    marker,
                    text: msg.term().to_string(),
                    unix_ms: Self::now_ms(),
                    latency_ms,
                    wire: false,
                });
            }
            Err(e) => {
                self.log.push(LogEntry {
                    direction: Direction::In,
                    tag: Some(tag.0),
                    marker: Some('!'),
                    text: format!("<{e}>"),
                    unix_ms: Self::now_ms(),
                    latency_ms,
                    wire: false,
                });
            }
        }
    }

    /// Show something the tap saw, as the exact line that crossed the port.
    fn push_tap(&mut self, event: TapEvent) {
        let (direction, marker, tag, text) = match event {
            TapEvent::Opened { conn, peer } => (
                Direction::In,
                None,
                None,
                format!("#{conn} opened from {peer}"),
            ),
            TapEvent::Closed { conn } => (Direction::In, None, None, format!("#{conn} closed")),
            TapEvent::Line {
                conn,
                direction: TapDirection::ClientToServer,
                line,
            } => (
                Direction::Out,
                None,
                leading_tag(&line).map(|t| t.0),
                format!("#{conn} {line}"),
            ),
            TapEvent::Line {
                conn,
                direction: TapDirection::ServerToClient,
                line,
            } => {
                // A reply looks like `00001 # Ack()`: the marker is the
                // second word.
                let marker = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|w| w.chars().next().filter(|c| "#!%".contains(*c)));
                (
                    Direction::In,
                    marker,
                    leading_tag(&line).map(|t| t.0),
                    format!("#{conn} {line}"),
                )
            }
        };
        self.log.push(LogEntry {
            direction,
            tag,
            marker,
            text,
            unix_ms: Self::now_ms(),
            latency_ms: None,
            wire: true,
        });
    }

    pub fn handle_app_event(&mut self, ev: AppEvent) {
        match ev {
            AppEvent::CommandResult {
                tag,
                started,
                result,
            } => self.push_result(tag, started, result),
            AppEvent::Connected(client) => {
                self.client = Some(client);
                self.conn = ConnState::Connected;
                self.connecting = false;
                let secure = if self.tls.is_some() { " (TLS)" } else { "" };
                self.status = format!("Connected to {}{secure}", self.addr());
            }
            AppEvent::RawLineSent => {
                self.status = "Sent a line without a tag: not waiting for a reply".to_string();
            }
            AppEvent::ConnectFailed(err) => {
                self.connecting = false;
                self.status = format!("Connect failed: {err}");
            }
            AppEvent::MockServerStarted { port, handle } => {
                self.port = port.to_string();
                self.mock_handle = Some(handle);
                self.status = format!("Mock server listening on 127.0.0.1:{port}");
                self.connecting = false;
            }
            AppEvent::TapStarted { port, handle } => {
                self.tap_listen = port;
                self.mock_handle = Some(handle);
                self.conn = ConnState::Connected;
                self.connecting = false;
                self.status = format!(
                    "Tap on 127.0.0.1:{port}, forwarding to {}: point a client at the tap",
                    self.addr()
                );
            }
            AppEvent::TapFailed(err) => {
                self.connecting = false;
                self.status = format!("Tap failed: {err}");
            }
            AppEvent::Tap(event) => self.push_tap(event),
            AppEvent::MockServerFailed(err) => {
                self.connecting = false;
                self.status = format!("Mock server failed: {err}");
            }
        }
    }

    pub fn toggle_mode(&mut self) {
        if self.conn == ConnState::Connected || self.connecting {
            self.status = "Disconnect before switching mode".to_string();
            return;
        }
        self.mode = match self.mode {
            Mode::Client => Mode::MockServer,
            Mode::MockServer => Mode::Tap,
            Mode::Tap => Mode::Client,
        };
    }

    pub fn connect_or_disconnect(&mut self) {
        if self.conn == ConnState::Connected {
            self.client = None;
            if let Some(h) = self.mock_handle.take() {
                h.abort();
            }
            self.conn = ConnState::Disconnected;
            self.status = "Disconnected".to_string();
            return;
        }
        if self.connecting {
            return;
        }
        self.connecting = true;
        match self.mode {
            Mode::Client => {
                let addr = self.addr();
                let tx = self.app_tx.clone();
                let tls = self.tls.as_ref().map(|opts| opts.client_config(&self.host));
                self.status = format!("Connecting to {addr}...");
                tokio::spawn(async move {
                    let connected = match tls {
                        Some(Ok(tls)) => IppClient::connect_tls(addr, &tls).await,
                        Some(Err(e)) => Err(e),
                        None => IppClient::connect(addr).await,
                    };
                    match connected {
                        Ok(client) => {
                            let _ = tx.send(AppEvent::Connected(Arc::new(client)));
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::ConnectFailed(e.to_string()));
                        }
                    }
                });
            }
            Mode::Tap => {
                let listen = self.tap_listen;
                let target = self.addr();
                let tx = self.app_tx.clone();
                self.status = format!("Starting tap on port {listen}...");
                tokio::spawn(async move {
                    match IppTap::bind(("127.0.0.1", listen), target).await {
                        Ok(tap) => {
                            let port = tap.local_addr().map_or(listen, |a| a.port());
                            // Subscribe before serve takes the tap, so no line is missed.
                            let mut events = tap.subscribe();
                            let handle = tokio::spawn(tap.serve());
                            let forward = tx.clone();
                            tokio::spawn(async move {
                                while let Ok(event) = events.recv().await {
                                    if forward.send(AppEvent::Tap(event)).is_err() {
                                        break;
                                    }
                                }
                            });
                            let _ = tx.send(AppEvent::TapStarted { port, handle });
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::TapFailed(e.to_string()));
                        }
                    }
                });
            }
            Mode::MockServer => {
                let port: u16 = self.port.parse().unwrap_or(1294);
                let tx = self.app_tx.clone();
                self.status = format!("Starting mock server on port {port}...");
                tokio::spawn(async move {
                    match IppMockServer::bind(("127.0.0.1", port)).await {
                        Ok(server) => {
                            let actual_port = match server.local_addr() {
                                Ok(a) => a.port(),
                                Err(e) => {
                                    let _ = tx.send(AppEvent::MockServerFailed(e.to_string()));
                                    return;
                                }
                            };
                            let handle = tokio::spawn(server.serve());
                            let _ = tx.send(AppEvent::MockServerStarted {
                                port: actual_port,
                                handle,
                            });
                            // Also connect a client to our own mock so presets can be sent.
                            match IppClient::connect(("127.0.0.1", actual_port)).await {
                                Ok(client) => {
                                    let _ = tx.send(AppEvent::Connected(Arc::new(client)));
                                }
                                Err(e) => {
                                    let _ = tx.send(AppEvent::ConnectFailed(e.to_string()));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::MockServerFailed(e.to_string()));
                        }
                    }
                });
            }
        }
    }

    fn not_connected_reason(&self) -> &'static str {
        if self.mode == Mode::Tap {
            "Tap mode only watches traffic; press m to switch to Client mode to send"
        } else if self.connecting {
            "Still connecting — command not sent, try again in a moment"
        } else {
            "Not connected — command not sent"
        }
    }

    fn send_command(&mut self, cmd: Command) {
        let Some(client) = self.client.clone() else {
            let reason = self.not_connected_reason();
            self.push_denied(reason);
            return;
        };
        let term: ippdme_core::Term = cmd.into();
        let tag = client.allocate_tag();
        self.push_out(tag, term.to_string());
        let tx = self.app_tx.clone();
        tokio::spawn(async move {
            let started = Instant::now();
            let result = client.send_with_tag(tag, term).await;
            let _ = tx.send(AppEvent::CommandResult {
                tag,
                started,
                result,
            });
        });
    }

    pub fn send_selected_preset(&mut self) {
        let flat = flatten(&self.presets);
        if let Some((_, _, _, preset)) = flat.get(self.selected_preset) {
            let cmd = (preset.build)();
            self.send_command(cmd);
        }
    }

    pub fn toggle_raw_line_mode(&mut self) {
        self.raw_line_mode = !self.raw_line_mode;
        self.status = if self.raw_line_mode {
            "Raw line input: lines are sent exactly as typed".to_string()
        } else {
            "Command input: terms are parsed and tagged for you".to_string()
        };
    }

    /// Send what is in the input box, in whichever input mode is active.
    pub fn submit_input(&mut self) {
        if self.raw_line_mode {
            self.send_raw_line();
        } else {
            self.send_raw_input();
        }
    }

    /// Send the input as one line, verbatim: no parsing and no tag added, so
    /// it can be malformed on purpose. Like typing into `nc`.
    fn send_raw_line(&mut self) {
        let line = self.input.trim_end().to_string();
        if line.is_empty() {
            return;
        }
        let Some(client) = self.client.clone() else {
            let reason = self.not_connected_reason();
            self.push_denied(reason);
            return;
        };
        let tag = leading_tag(&line);
        self.log.push(LogEntry {
            direction: Direction::Out,
            tag: tag.map(|t| t.0),
            marker: None,
            text: line.clone(),
            unix_ms: Self::now_ms(),
            latency_ms: None,
            wire: false,
        });
        self.input.clear();
        let tx = self.app_tx.clone();
        tokio::spawn(async move {
            let started = Instant::now();
            let event = match client.send_line(&line).await {
                Ok(Some(reply)) => AppEvent::CommandResult {
                    tag: reply.tag(),
                    started,
                    result: Ok(reply),
                },
                Ok(None) => AppEvent::RawLineSent,
                Err(e) => AppEvent::CommandResult {
                    tag: tag.unwrap_or(Tag(0)),
                    started,
                    result: Err(e),
                },
            };
            let _ = tx.send(event);
        });
    }

    pub fn send_raw_input(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        match parse_term_str(&text) {
            Ok(term) => {
                let Some(client) = self.client.clone() else {
                    let reason = self.not_connected_reason();
                    self.push_denied(reason);
                    return;
                };
                let tag = client.allocate_tag();
                self.push_out(tag, term.to_string());
                let tx = self.app_tx.clone();
                tokio::spawn(async move {
                    let started = Instant::now();
                    let result = client.send_with_tag(tag, term).await;
                    let _ = tx.send(AppEvent::CommandResult {
                        tag,
                        started,
                        result,
                    });
                });
                self.input.clear();
            }
            Err(e) => {
                self.push_denied(format!("Parse error: {e}"));
            }
        }
    }

    pub fn export_session(&mut self) {
        let stamp = Self::now_ms();
        let log_path = format!("ippdme-session-{stamp}.log");
        let json_path = format!("ippdme-session-{stamp}.json");

        let mut file = match fs::File::create(&log_path) {
            Ok(f) => f,
            Err(e) => {
                self.status = format!("Export failed: {e}");
                return;
            }
        };
        for entry in &self.log {
            let dir = match entry.direction {
                Direction::Out => "OUT",
                Direction::In => "IN ",
            };
            let lat = entry
                .latency_ms
                .map(|ms| format!(" ({ms}ms)"))
                .unwrap_or_default();
            let _ = writeln!(
                file,
                "{} {} tag={:?} marker={:?} {}{}",
                entry.unix_ms, dir, entry.tag, entry.marker, entry.text, lat
            );
        }

        match serde_json::to_string_pretty(&self.log) {
            Ok(json) => {
                if let Err(e) = fs::write(&json_path, json) {
                    self.status = format!("JSON export failed: {e}");
                    return;
                }
            }
            Err(e) => {
                self.status = format!("JSON export failed: {e}");
                return;
            }
        }

        self.status = format!("Exported session to {log_path} and {json_path}");
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Log,
            Focus::Log => Focus::Input,
            Focus::Input => Focus::Sidebar,
        };
    }

    pub fn tick(&mut self) -> Duration {
        while let Ok(ev) = self.app_rx.try_recv() {
            self.handle_app_event(ev);
        }
        Duration::from_millis(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tick until `done` holds, or fail after a couple of seconds.
    async fn tick_until(app: &mut App, done: impl Fn(&App) -> bool) {
        for _ in 0..200 {
            app.tick();
            if done(app) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("condition not reached; log: {:?}", app.status);
    }

    async fn connected_app() -> App {
        let mut app = App::new(None, 0);
        app.mode = Mode::MockServer;
        app.port = "0".to_string();
        app.connect_or_disconnect();
        tick_until(&mut app, |a| a.conn == ConnState::Connected).await;
        app
    }

    #[tokio::test]
    async fn raw_line_mode_sends_the_line_verbatim_and_logs_the_reply() {
        let mut app = connected_app().await;
        app.toggle_raw_line_mode();
        app.input = "00042 StartSession()".to_string();
        app.submit_input();
        assert!(app.input.is_empty());

        tick_until(&mut app, |a| {
            a.log.iter().any(|e| e.direction == Direction::In)
        })
        .await;
        let sent = &app.log[0];
        assert_eq!((sent.direction, sent.tag), (Direction::Out, Some(42)));
        assert_eq!(sent.text, "00042 StartSession()");
        let reply = app
            .log
            .iter()
            .find(|e| e.direction == Direction::In)
            .unwrap();
        assert_eq!((reply.tag, reply.marker), (Some(42), Some('#')));
    }

    #[tokio::test]
    async fn raw_line_without_a_tag_is_sent_but_not_awaited() {
        let mut app = connected_app().await;
        app.toggle_raw_line_mode();
        app.input = "this is not a protocol line".to_string();
        app.submit_input();

        tick_until(&mut app, |a| a.status.contains("not waiting")).await;
        assert_eq!(app.log.len(), 1);
        assert_eq!(app.log[0].text, "this is not a protocol line");
        assert_eq!(app.log[0].tag, None);
    }

    #[tokio::test]
    async fn command_mode_still_parses_terms_and_adds_the_tag() {
        let mut app = connected_app().await;
        app.input = "StartSession()".to_string();
        app.submit_input();

        tick_until(&mut app, |a| {
            a.log.iter().any(|e| e.direction == Direction::In)
        })
        .await;
        assert_eq!(app.log[0].tag, Some(1));
        assert_eq!(app.log[0].text, "StartSession()");
    }

    #[tokio::test]
    async fn tap_mode_shows_the_lines_crossing_the_port() {
        // A server for the tap to forward to, and a client going through the tap.
        let server = IppMockServer::bind(("127.0.0.1", 0)).await.unwrap();
        let server_port = server.local_addr().unwrap().port();
        tokio::spawn(server.serve());

        let mut app = App::new(None, 0);
        app.mode = Mode::Tap;
        app.host = "127.0.0.1".to_string();
        app.port = server_port.to_string();
        app.connect_or_disconnect();
        tick_until(&mut app, |a| a.conn == ConnState::Connected).await;
        assert!(app.tap_listen != 0, "tap should report its real port");

        let client = IppClient::connect(("127.0.0.1", app.tap_listen))
            .await
            .unwrap();
        client.start_session().await.unwrap();

        tick_until(&mut app, |a| a.log.iter().filter(|e| e.wire).count() >= 3).await;
        let lines: Vec<_> = app
            .log
            .iter()
            .map(|e| (e.direction, e.marker, e.text.as_str()))
            .collect();
        assert!(lines.iter().any(|l| l.2.starts_with("#1 opened from")));
        assert!(lines.contains(&(Direction::Out, None, "#1 00001 StartSession()")));
        assert!(lines.contains(&(Direction::In, Some('#'), "#1 00001 # Ready()")));
        assert!(app.log.iter().all(|e| e.wire));
    }

    #[tokio::test]
    async fn tap_mode_cannot_send_and_says_so() {
        let mut app = App::new(None, 0);
        app.mode = Mode::Tap;
        app.input = "StartSession()".to_string();
        app.submit_input();
        assert!(app.status.contains("Tap mode only watches"));
    }

    #[test]
    fn m_cycles_through_all_three_modes() {
        let mut app = App::new(None, 0);
        let mut seen = vec![app.mode];
        for _ in 0..3 {
            app.toggle_mode();
            seen.push(app.mode);
        }
        assert_eq!(
            seen,
            [Mode::Client, Mode::MockServer, Mode::Tap, Mode::Client]
        );
    }
}
