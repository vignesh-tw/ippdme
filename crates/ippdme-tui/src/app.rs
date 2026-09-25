use std::fs;
use std::io::Write as _;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ippdme_core::{parse_term_str, Command, Message, Tag};
use ippdme_net::{IppClient, IppMockServer, NetError};
use serde::Serialize;
use tokio::sync::mpsc;

use crate::presets::{default_presets, flatten, PresetCategory};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Client,
    MockServer,
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
    MockServerStarted {
        port: u16,
        handle: tokio::task::JoinHandle<ippdme_net::Result<()>>,
    },
    MockServerFailed(String),
}

pub struct App {
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
    pub focus: Focus,
    pub editing_addr: Option<AddrField>,
    pub status: String,
    pub should_quit: bool,
    pub connecting: bool,
    pub app_tx: mpsc::UnboundedSender<AppEvent>,
    pub app_rx: mpsc::UnboundedReceiver<AppEvent>,
    pending_started: std::collections::HashMap<u32, Instant>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AddrField {
    Host,
    Port,
}

impl App {
    pub fn new() -> Self {
        let (app_tx, app_rx) = mpsc::unbounded_channel();
        App {
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
            focus: Focus::Sidebar,
            editing_addr: None,
            status: "Disconnected".to_string(),
            should_quit: false,
            connecting: false,
            app_tx,
            app_rx,
            pending_started: std::collections::HashMap::new(),
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
        self.pending_started.insert(tag.0, Instant::now());
        self.log.push(LogEntry {
            direction: Direction::Out,
            tag: Some(tag.0),
            marker: None,
            text,
            unix_ms: Self::now_ms(),
            latency_ms: None,
        });
    }

    fn push_result(&mut self, tag: Tag, started: Instant, result: Result<Message, NetError>) {
        let latency_ms = Some(started.elapsed().as_millis());
        self.pending_started.remove(&tag.0);
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
                });
            }
        }
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
                self.status = format!("Connected to {}", self.addr());
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
            Mode::MockServer => Mode::Client,
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
                self.status = format!("Connecting to {addr}...");
                tokio::spawn(async move {
                    match IppClient::connect(addr).await {
                        Ok(client) => {
                            let _ = tx.send(AppEvent::Connected(Arc::new(client)));
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::ConnectFailed(e.to_string()));
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

    fn send_command(&mut self, cmd: Command) {
        let Some(client) = self.client.clone() else {
            self.status = "Not connected".to_string();
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

    pub fn send_raw_input(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }
        match parse_term_str(&text) {
            Ok(term) => {
                let Some(client) = self.client.clone() else {
                    self.status = "Not connected".to_string();
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
                self.status = format!("Parse error: {e}");
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

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
