mod app;
mod cli;
mod presets;
mod tls;
mod ui;

use std::io;
use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{AddrField, App, Focus};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{}", cli::USAGE);
        return Ok(());
    }
    let cli =
        cli::Cli::from_args(args).map_err(|e| color_eyre::eyre::eyre!("{e}\n\n{}", cli::USAGE))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, cli).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, cli: cli::Cli) -> Result<()> {
    let mut app = App::new(cli.tls, cli.tap_listen);
    let mut events = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(100));

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        tokio::select! {
            _ = tick.tick() => {
                app.tick();
            }
            Some(Ok(ev)) = events.next() => {
                if let Event::Key(key) = ev {
                    if key.kind == KeyEventKind::Press {
                        handle_key(&mut app, key.code);
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) {
    if let Some(field) = app.editing_addr {
        match code {
            KeyCode::Enter | KeyCode::Esc => app.editing_addr = None,
            KeyCode::Backspace => match field {
                AddrField::Host => {
                    app.host.pop();
                }
                AddrField::Port => {
                    app.port.pop();
                }
            },
            KeyCode::Char(c) => match field {
                AddrField::Host => app.host.push(c),
                AddrField::Port => {
                    if c.is_ascii_digit() {
                        app.port.push(c);
                    }
                }
            },
            _ => {}
        }
        return;
    }

    if app.focus == Focus::Input {
        match code {
            KeyCode::Esc => app.focus = Focus::Sidebar,
            KeyCode::Tab => app.cycle_focus(),
            KeyCode::Enter => app.submit_input(),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Char(c) => app.input.push(c),
            _ => {}
        }
        return;
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Tab => app.cycle_focus(),
        KeyCode::Char('c') => app.connect_or_disconnect(),
        KeyCode::Char('m') => app.toggle_mode(),
        KeyCode::Char('e') => app.export_session(),
        KeyCode::Char('r') => app.toggle_raw_line_mode(),
        KeyCode::Char('h') if app.conn == app::ConnState::Disconnected && !app.connecting => {
            app.host.clear();
            app.editing_addr = Some(AddrField::Host);
        }
        KeyCode::Char('p') if app.conn == app::ConnState::Disconnected && !app.connecting => {
            app.port.clear();
            app.editing_addr = Some(AddrField::Port);
        }
        KeyCode::Up => match app.focus {
            Focus::Sidebar => {
                app.selected_preset = app.selected_preset.saturating_sub(1);
            }
            Focus::Log => {
                let len = app.log.len();
                if len > 0 {
                    app.selected_log = Some(match app.selected_log {
                        Some(i) if i > 0 => i - 1,
                        Some(i) => i,
                        None => len - 1,
                    });
                }
            }
            Focus::Input => {}
        },
        KeyCode::Down => match app.focus {
            Focus::Sidebar => {
                let max = presets::flatten(&app.presets).len().saturating_sub(1);
                app.selected_preset = (app.selected_preset + 1).min(max);
            }
            Focus::Log => {
                let len = app.log.len();
                if len > 0 {
                    app.selected_log = Some(match app.selected_log {
                        Some(i) if i + 1 < len => i + 1,
                        Some(i) => i,
                        None => 0,
                    });
                }
            }
            Focus::Input => {}
        },
        KeyCode::Enter if app.focus == Focus::Sidebar => {
            app.send_selected_preset();
        }
        _ => {}
    }
}
