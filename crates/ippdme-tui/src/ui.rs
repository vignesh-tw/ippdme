use ratatui::layout::{Constraint, Direction as LayoutDirection, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, ConnState, Direction, Focus, Mode};
use crate::presets::flatten;

pub fn draw(f: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(LayoutDirection::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_connection_bar(f, app, root[0]);

    let body = Layout::default()
        .direction(LayoutDirection::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(20)])
        .split(root[1]);

    draw_sidebar(f, app, body[0]);
    draw_log(f, app, body[1]);
    draw_input(f, app, root[2]);
    draw_help(f, root[3]);
}

fn draw_connection_bar(f: &mut Frame, app: &App, area: Rect) {
    let (status_color, status_label) = match app.conn {
        ConnState::Connected => (Color::Green, "CONNECTED"),
        ConnState::Disconnected if app.connecting => (Color::Yellow, "CONNECTING"),
        ConnState::Disconnected => (Color::Red, "DISCONNECTED"),
    };
    let mode_label = match app.mode {
        Mode::Client => "Client",
        Mode::MockServer => "Mock Server",
        Mode::Tap => "Tap",
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" {status_label} "),
            Style::default()
                .fg(Color::Black)
                .bg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Mode: ", Style::default().fg(Color::Gray)),
        Span::styled(mode_label, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("Target: ", Style::default().fg(Color::Gray)),
        Span::styled(
            match app.mode {
                Mode::Tap => format!("127.0.0.1:{} -> {}:{}", app.tap_listen, app.host, app.port),
                _ => format!(
                    "{}:{}{}",
                    app.host,
                    app.port,
                    if app.tls.is_some() && app.mode == Mode::Client {
                        " [TLS]"
                    } else {
                        ""
                    }
                ),
            },
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(&app.status, Style::default().fg(Color::Cyan)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ippdme-tui — Postman for I++ DME ");
    f.render_widget(Paragraph::new(line).block(block), area);
}

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let flat = flatten(&app.presets);
    let mut items = Vec::new();
    let mut last_cat: Option<&str> = None;
    for (i, (_, _, cat_name, preset)) in flat.iter().enumerate() {
        if last_cat != Some(cat_name) {
            items.push(ListItem::new(Line::from(Span::styled(
                cat_name.to_string(),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ))));
            last_cat = Some(cat_name);
        }
        let selected = i == app.selected_preset;
        let style = if selected {
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", preset.label),
            style,
        ))));
    }

    let border_style = if app.focus == Focus::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" Presets ");
    f.render_widget(List::new(items).block(block), area);
}

fn draw_log(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .log
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let (arrow, color) = match (entry.direction, entry.marker) {
                (Direction::Out, Some('!')) => ("!!!", Color::Red),
                (Direction::Out, _) => ("-->", Color::Blue),
                (Direction::In, Some('#')) => ("<--", Color::Green),
                (Direction::In, Some('!')) => ("<--", Color::Red),
                (Direction::In, Some('%')) => ("<--", Color::Yellow),
                (Direction::In, _) => ("<--", Color::White),
            };
            let tag_str = entry
                .tag
                .map(|t| format!("{t:05}"))
                .unwrap_or_else(|| "-----".to_string());
            let latency = entry
                .latency_ms
                .map(|ms| format!(" [{ms}ms]"))
                .unwrap_or_default();
            let mut style = Style::default().fg(color);
            if app.selected_log == Some(i) {
                style = style.add_modifier(Modifier::REVERSED);
            }
            let text = if entry.wire {
                format!("{arrow} {}", entry.text)
            } else {
                format!("{arrow} {tag_str} {}{latency}", entry.text)
            };
            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let border_style = if app.focus == Focus::Log {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(if app.mode == Mode::Tap {
            " Wire view: lines crossing the port (--> client to server, <-- server to client) "
        } else {
            " Live Protocol Stream (blue=out, green=ack, red=error, yellow=data) "
        });
    f.render_widget(List::new(items).block(block), area);
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focus == Focus::Input {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(if app.raw_line_mode {
            " Raw line, sent exactly as typed (e.g. 00001 StartSession()) "
        } else {
            " Raw command (e.g. GoTo(X(10.0), Y(20.0), Z(5.0))) "
        });
    let text = if app.input.is_empty() && app.focus != Focus::Input {
        Span::styled(
            if app.raw_line_mode {
                "Press Tab to focus, then type a full line, tag included, and Enter to send it verbatim"
            } else {
                "Press Tab to focus, then type a raw I++ term and Enter to send"
            },
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw(app.input.as_str())
    };
    f.render_widget(
        Paragraph::new(Line::from(text))
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help = "Tab: switch focus | ↑/↓: navigate | Enter: send | c: connect/disconnect | \
                m: toggle mode | r: raw line input | h/p: edit host/port | e: export | q/Esc: quit";
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            help,
            Style::default().fg(Color::DarkGray),
        ))),
        area,
    );
}
