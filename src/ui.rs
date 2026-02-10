use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::App;
use crate::types::{AppMode, Session, Window};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(frame.area());

    render_header(frame, app, chunks[0]);

    let main_chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_session_list(frame, app, main_chunks[0]);
    render_preview(frame, app, main_chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    if app.show_help {
        render_help_overlay(frame);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let session_info = if app.sessions.is_empty() {
        String::new()
    } else {
        format!(" ({} sessions)", app.sessions.len())
    };
    let header = Paragraph::new(format!("tmx{session_info} | ? help | q quit"))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(header, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(ref err) = app.error_message {
        let error_bar = Paragraph::new(err.as_str()).style(
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(error_bar, area);
        return;
    }

    let tag_indicator = app
        .tag_filter
        .as_ref()
        .map(|t| format!(" [tag:{t}]"))
        .unwrap_or_default();

    let selected_info = app
        .sessions
        .get(app.selected)
        .map(|s| {
            let status = if s.attached > 0 {
                "attached"
            } else {
                "detached"
            };
            format!(" | {} ({status})", s.name)
        })
        .unwrap_or_default();

    let footer_text = match app.mode {
        AppMode::Normal => format!(
            "NORMAL{tag_indicator}{selected_info} | {}",
            app.status_message
        ),
        AppMode::Search => format!("SEARCH /{}", app.input_buffer),
        AppMode::Input(_) => format!("INPUT  {}", app.input_buffer),
        AppMode::Confirm(_) => format!("CONFIRM | {}", app.status_message),
    };
    let footer =
        Paragraph::new(footer_text).style(Style::default().bg(Color::Blue).fg(Color::White));
    frame.render_widget(footer, area);
}

fn render_help_overlay(frame: &mut Frame) {
    let area = frame.area();
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 18u16.min(area.height.saturating_sub(4));

    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let help_lines = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("j/k  ", Style::default().fg(Color::Yellow)),
            Span::raw("Move down/up"),
        ]),
        Line::from(vec![
            Span::styled("G    ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to last"),
        ]),
        Line::from(vec![
            Span::styled("gg   ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to first"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Attach to session"),
        ]),
        Line::from(vec![
            Span::styled("n    ", Style::default().fg(Color::Yellow)),
            Span::raw("New session"),
        ]),
        Line::from(vec![
            Span::styled("r    ", Style::default().fg(Color::Yellow)),
            Span::raw("Rename session"),
        ]),
        Line::from(vec![
            Span::styled("dd   ", Style::default().fg(Color::Yellow)),
            Span::raw("Kill session"),
        ]),
        Line::from(vec![
            Span::styled("D    ", Style::default().fg(Color::Yellow)),
            Span::raw("Detach clients"),
        ]),
        Line::from(vec![
            Span::styled("/    ", Style::default().fg(Color::Yellow)),
            Span::raw("Fuzzy search"),
        ]),
        Line::from(vec![
            Span::styled("t    ", Style::default().fg(Color::Yellow)),
            Span::raw("Add tag"),
        ]),
        Line::from(vec![
            Span::styled("T    ", Style::default().fg(Color::Yellow)),
            Span::raw("Filter by tag / clear"),
        ]),
        Line::from(vec![
            Span::styled("Tab  ", Style::default().fg(Color::Yellow)),
            Span::raw("Expand/collapse windows"),
        ]),
        Line::from(vec![
            Span::styled("?    ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("q    ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::Gray),
        )),
    ];

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(Style::default().bg(Color::Black).fg(Color::White)),
        )
        .alignment(Alignment::Center);
    frame.render_widget(help, popup_area);
}

fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    let visible_count = app.visible_session_count();

    if visible_count == 0 && !app.search_active {
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

    if visible_count == 0 && app.search_active {
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

        let empty = Paragraph::new("No matches found")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(empty, centered[1]);
        return;
    }

    let available_width = area.width.saturating_sub(5) as usize;
    let mut items: Vec<ListItem> = Vec::new();
    let mut selected_item_index: Option<usize> = None;

    if app.search_active {
        for (vis_idx, match_result) in app.filtered_results.iter().enumerate() {
            if let Some(session) = app.sessions.get(match_result.session_index) {
                let is_expanded = app.expanded_sessions.contains(&session.name);
                let arrow = if is_expanded { "▼" } else { "▶" };
                let tags = app.config.get_tags(&session.name);

                let line = build_highlighted_session_line(
                    session,
                    arrow,
                    &match_result.indices,
                    &tags,
                    available_width,
                );

                if vis_idx == app.selected {
                    selected_item_index = Some(items.len());
                }
                items.push(ListItem::new(line));

                if is_expanded {
                    if let Some(windows) = app.session_windows.get(&session.name) {
                        for window in windows {
                            let window_line =
                                format_window_line(window, available_width.saturating_sub(4));
                            items.push(
                                ListItem::new(Line::from(format!("  ├─ {window_line}")))
                                    .style(Style::default().fg(Color::Cyan)),
                            );
                        }
                    }
                }
            }
        }
    } else {
        let visible_indices = app.tag_filtered_sessions();
        for (vis_idx, &session_idx) in visible_indices.iter().enumerate() {
            if let Some(session) = app.sessions.get(session_idx) {
                let is_expanded = app.expanded_sessions.contains(&session.name);
                let arrow = if is_expanded { "▼" } else { "▶" };
                let tags = app.config.get_tags(&session.name);

                let line = if tags.is_empty() {
                    let session_text =
                        format_session_line(session, available_width.saturating_sub(2));
                    Line::from(format!("{arrow} {session_text}"))
                } else {
                    build_session_line_with_tags(session, arrow, &tags, available_width)
                };

                if vis_idx == app.selected {
                    selected_item_index = Some(items.len());
                }
                items.push(ListItem::new(line));

                if is_expanded {
                    if let Some(windows) = app.session_windows.get(&session.name) {
                        for window in windows {
                            let window_line =
                                format_window_line(window, available_width.saturating_sub(4));
                            items.push(
                                ListItem::new(Line::from(format!("  ├─ {window_line}")))
                                    .style(Style::default().fg(Color::Cyan)),
                            );
                        }
                    }
                }
            }
        }
    }

    let mut state = ListState::default();
    state.select(selected_item_index);

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

fn build_highlighted_session_line<'a>(
    session: &Session,
    arrow: &str,
    match_indices: &[u32],
    tags: &[String],
    _available_width: usize,
) -> Line<'a> {
    let status = if session.attached > 0 {
        "attached"
    } else {
        "detached"
    };
    let indicator = if session.attached > 0 { "●" } else { "○" };

    let prefix = format!("{arrow} {indicator} ");

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw(prefix));

    let highlight_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
    let normal_style = Style::default();

    let indices_set: std::collections::HashSet<u32> = match_indices.iter().copied().collect();

    for (char_idx, ch) in session.name.chars().enumerate() {
        if indices_set.contains(&(char_idx as u32)) {
            spans.push(Span::styled(ch.to_string(), highlight_style));
        } else {
            spans.push(Span::styled(ch.to_string(), normal_style));
        }
    }

    for tag in tags {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("[{tag}]"),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::raw(format!(
        "  {} windows  {status}",
        session.windows
    )));
    Line::from(spans)
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Preview");

    if app.preview_content.is_empty() {
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

        let empty = Paragraph::new("No preview available")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(empty, centered[1]);
        return;
    }

    let text = app
        .preview_content
        .as_bytes()
        .into_text()
        .unwrap_or_else(|_| ratatui::text::Text::raw("Failed to parse ANSI"));

    let preview = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

    frame.render_widget(preview, area);
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

fn build_session_line_with_tags<'a>(
    session: &Session,
    arrow: &str,
    tags: &[String],
    _available_width: usize,
) -> Line<'a> {
    let status = if session.attached > 0 {
        "attached"
    } else {
        "detached"
    };
    let indicator = if session.attached > 0 { "●" } else { "○" };

    let mut spans: Vec<Span> = vec![Span::raw(format!("{arrow} {indicator} {}", session.name))];

    for tag in tags {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("[{tag}]"),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::raw(format!(
        "  {} windows  {status}",
        session.windows
    )));

    Line::from(spans)
}

fn format_window_line(window: &Window, max_width: usize) -> String {
    let active_mark = if window.active { "*" } else { " " };
    let full_line = format!(
        "{}: {}{} ({})",
        window.index, window.name, active_mark, window.active_command
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
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

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
        app.sessions = vec![make_session("work", 2, 1), make_session("personal", 1, 0)];

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
        assert!(
            text.contains(">>"),
            "selected row should include highlight symbol"
        );
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

    #[test]
    fn test_ansi_to_text_basic() {
        use ansi_to_tui::IntoText;
        let ansi = b"\x1b[31mhello\x1b[0m world";
        let text = ansi.into_text().expect("basic ANSI should parse");
        let plain: String = text
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(plain.contains("hello"));
        assert!(plain.contains("world"));
    }

    #[test]
    fn test_ansi_24bit_color() {
        use ansi_to_tui::IntoText;
        use ratatui::style::Color;
        let ansi = b"\x1b[38;2;255;0;0mred text\x1b[0m";
        let text = ansi.into_text().expect("24-bit ANSI should parse");
        let span = &text.lines[0].spans[0];
        assert_eq!(span.style.fg, Some(Color::Rgb(255, 0, 0)));
        assert!(span.content.contains("red text"));
    }

    #[test]
    fn test_preview_cjk_width() {
        use unicode_width::UnicodeWidthStr;
        let korean = "안녕하세요";
        assert_eq!(UnicodeWidthStr::width(korean), 10);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("test", 1, 0)];
        app.preview_content = format!("{korean}\n");

        terminal
            .draw(|f| render(f, &app))
            .expect("render with CJK preview should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        for ch in korean.chars() {
            assert!(
                text.contains(ch),
                "CJK char '{ch}' should appear in preview buffer"
            );
        }
    }

    #[test]
    fn test_preview_empty_pane() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("test", 1, 0)];

        terminal
            .draw(|f| render(f, &app))
            .expect("render with empty preview should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(
            text.contains("No preview available"),
            "empty preview should show fallback text"
        );
    }

    #[test]
    fn test_preview_nonexistent_session() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let app = App::new();

        terminal
            .draw(|f| render(f, &app))
            .expect("render with no sessions should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(
            text.contains("Preview") || text.contains("No preview"),
            "preview area should render gracefully with no sessions"
        );
    }

    #[test]
    fn test_render_expanded_session_shows_windows() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("work", 2, 1)];
        app.expanded_sessions.insert("work".to_string());
        app.session_windows.insert(
            "work".to_string(),
            vec![
                crate::types::Window {
                    id: "@0".to_string(),
                    session_id: "$0".to_string(),
                    index: 0,
                    name: "editor".to_string(),
                    active: true,
                    active_command: "vim".to_string(),
                },
                crate::types::Window {
                    id: "@1".to_string(),
                    session_id: "$0".to_string(),
                    index: 1,
                    name: "shell".to_string(),
                    active: false,
                    active_command: "bash".to_string(),
                },
            ],
        );

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(
            text.contains("editor"),
            "expanded session should show window name 'editor'"
        );
        assert!(
            text.contains("shell"),
            "expanded session should show window name 'shell'"
        );
    }

    #[test]
    fn test_render_collapsed_session_hides_windows() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("work", 2, 1)];
        app.session_windows.insert(
            "work".to_string(),
            vec![crate::types::Window {
                id: "@0".to_string(),
                session_id: "$0".to_string(),
                index: 0,
                name: "editor".to_string(),
                active: true,
                active_command: "vim".to_string(),
            }],
        );

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(text.contains("work"), "session name should show");
        assert!(
            !text.contains("editor"),
            "collapsed session should NOT show window names"
        );
    }

    #[test]
    fn test_render_window_active_indicator() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("dev", 1, 0)];
        app.expanded_sessions.insert("dev".to_string());
        app.session_windows.insert(
            "dev".to_string(),
            vec![crate::types::Window {
                id: "@0".to_string(),
                session_id: "$0".to_string(),
                index: 0,
                name: "main".to_string(),
                active: true,
                active_command: "vim".to_string(),
            }],
        );

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(text.contains("*"), "active window should have * indicator");
        assert!(text.contains("main"), "window name should display");
    }

    #[test]
    fn test_render_expand_collapse_arrow() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("alpha", 2, 0), make_session("beta", 1, 0)];
        app.expanded_sessions.insert("alpha".to_string());

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(
            text.contains("▼") || text.contains("▾"),
            "expanded session should show down arrow"
        );
        assert!(
            text.contains("▶") || text.contains("▸"),
            "collapsed session should show right arrow"
        );
    }

    #[test]
    fn test_preview_layout_split() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("alpha", 1, 0)];
        app.preview_content = "preview text here".to_string();

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(text.contains("Sessions"), "left pane should show Sessions");
        assert!(text.contains("Preview"), "right pane should show Preview");
        assert!(
            text.contains("preview text here"),
            "preview content should be visible"
        );
    }

    #[test]
    fn test_render_help_overlay() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("test", 1, 0)];
        app.show_help = true;

        terminal
            .draw(|f| render(f, &app))
            .expect("render with help overlay should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(
            text.contains("Keybindings"),
            "help overlay should show keybindings title"
        );
        assert!(
            text.contains("Fuzzy search"),
            "help overlay should list search keybinding"
        );
        assert!(
            text.contains("Quit"),
            "help overlay should list quit keybinding"
        );
    }

    #[test]
    fn test_render_error_in_status_bar() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.error_message = Some("tmux command failed".to_string());
        app.error_time = Some(std::time::Instant::now());

        terminal
            .draw(|f| render(f, &app))
            .expect("render with error should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(
            text.contains("tmux command failed"),
            "error should display in status bar"
        );
    }

    #[test]
    fn test_render_header_session_count() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("a", 1, 0), make_session("b", 1, 0)];

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(
            text.contains("2 sessions"),
            "header should show session count"
        );
    }

    #[test]
    fn test_render_status_bar_selected_info() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        let mut app = App::new();
        app.sessions = vec![make_session("mywork", 2, 1)];
        app.selected = 0;

        terminal
            .draw(|f| render(f, &app))
            .expect("render should succeed");

        let text = buffer_to_text(terminal.backend().buffer());
        assert!(
            text.contains("mywork"),
            "status bar should show selected session name"
        );
        assert!(
            text.contains("attached"),
            "status bar should show attach status"
        );
    }
}
