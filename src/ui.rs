use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::App;
use crate::types::{AppMode, Session};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(frame.area());

    let header = Paragraph::new("tmx | ? help | q quit")
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(header, chunks[0]);

    render_session_list(frame, app, chunks[1]);

    let footer_text = match app.mode {
        AppMode::Normal => format!("NORMAL | {}", app.status_message),
        AppMode::Search => format!("SEARCH /{}", app.input_buffer),
        AppMode::Input(_) => format!("INPUT  {}", app.input_buffer),
        AppMode::Confirm(_) => format!("CONFIRM | {}", app.status_message),
    };
    let footer = Paragraph::new(footer_text).style(Style::default().bg(Color::Blue).fg(Color::White));
    frame.render_widget(footer, chunks[2]);
}

fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.sessions.is_empty() {
        let block = Block::default().borders(Borders::ALL).title("Sessions");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let centered = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(inner);

        let empty = Paragraph::new("No sessions. Press `n` to create.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(empty, centered[1]);
        return;
    }

    let available_width = area.width.saturating_sub(5) as usize;
    let items: Vec<ListItem> = app
        .sessions
        .iter()
        .map(|session| ListItem::new(Line::from(format_session_line(session, available_width))))
        .collect();

    let mut state = ListState::default();
    if !app.sessions.is_empty() {
        state.select(Some(app.selected.min(app.sessions.len() - 1)));
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Sessions"))
        .highlight_symbol(">> ")
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, area, &mut state);
}

fn format_session_line(session: &Session, max_width: usize) -> String {
    let status = if session.attached > 0 {
        "attached"
    } else {
        "detached"
    };
    let indicator = if session.attached > 0 { "●" } else { "○" };
    let full_line = format!(
        "{indicator} {}  {} windows  {status}",
        session.name, session.windows
    );

    truncate_with_ellipsis(&full_line, max_width)
}

fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }

    if max_width == 1 {
        return "…".to_string();
    }

    let mut result = String::new();
    let mut used_width = 0usize;
    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used_width + ch_width > max_width - 1 {
            break;
        }
        result.push(ch);
        used_width += ch_width;
    }
    result.push('…');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::types::Session;
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        Terminal,
    };

    fn make_session(name: &str, windows: usize, attached: usize) -> Session {
        Session {
            id: format!("${name}"),
            name: name.to_string(),
            windows,
            attached,
            created: 0,
            last_attached: 0,
            group: None,
            path: "/tmp".to_string(),
        }
    }

    fn buffer_to_text(buffer: &Buffer) -> String {
        let mut text = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }

    #[test]
    fn test_render_function_exists() {
        let _ = super::render as fn(&mut ratatui::Frame, &crate::app::App);
    }

    #[test]
    fn test_render_session_list() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![
            make_session("work", 2, 1),
            make_session("personal", 1, 0),
        ];

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(text.contains("work"));
        assert!(text.contains("personal"));
        assert!(text.contains("2 windows"));
        assert!(text.contains("attached"));
    }

    #[test]
    fn test_render_empty_list() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let app = App::new();
        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(text.contains("No sessions. Press `n` to create."));
    }

    #[test]
    fn test_render_selected_highlight() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("alpha", 1, 0), make_session("beta", 2, 1)];
        app.selected = 1;

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(text.contains(">>"), "selected row should include highlight symbol");
    }

    #[test]
    fn test_render_cjk_session_name() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("데모세션", 1, 0)];

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let rendered = format_session_line(&app.sessions[0], 70);
        assert!(rendered.contains("데모세션"));
        assert!(UnicodeWidthStr::width(rendered.as_str()) <= 70);
    }

    #[test]
    fn test_render_long_name_truncation() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session(
            "extremely-long-session-name-that-should-be-truncated",
            10,
            0,
        )];

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(text.contains('…'));
        assert!(!text.contains("extremely-long-session-name-that-should-be-truncated"));
    }

    #[test]
    fn test_render_footer_mode_label() {
        let backend = TestBackend::new(50, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let app = App::new();
        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(text.contains("NORMAL"));
    }
}
