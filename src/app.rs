use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::tmux;
use crate::types::{AppMode, AppResult, ConfirmAction, InputPurpose, Session};

const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(500);

pub struct App {
    pub sessions: Vec<Session>,
    pub selected: usize,
    pub mode: AppMode,
    pub should_quit: bool,
    pub input_buffer: String,
    pub status_message: String,
    pub last_g_press: Option<Instant>,
    last_d_press: Option<Instant>,
}

impl App {
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected: 0,
            mode: AppMode::Normal,
            should_quit: false,
            input_buffer: String::new(),
            status_message: String::new(),
            last_g_press: None,
            last_d_press: None,
        }
    }

    pub async fn refresh_sessions(&mut self) -> AppResult<()> {
        self.sessions = tmux::list_sessions().await?;
        if self.sessions.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.sessions.len() {
            self.selected = self.sessions.len() - 1;
        }
        Ok(())
    }

    pub async fn handle_event(&mut self, event: Event) -> AppResult<()> {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            match self.mode.clone() {
                AppMode::Normal => self.handle_normal_mode(key).await?,
                AppMode::Search => self.handle_search_mode(key).await?,
                AppMode::Input(purpose) => self.handle_input_mode(key, purpose).await?,
                AppMode::Confirm(action) => self.handle_confirm_mode(key, action).await?,
            }
        }

        Ok(())
    }

    async fn handle_normal_mode(&mut self, key: KeyEvent) -> AppResult<()> {
        if matches!(
            key,
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }
        ) {
            self.should_quit = true;
            self.clear_multi_key_state();
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                self.clear_multi_key_state();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                self.clear_multi_key_state();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_previous();
                self.clear_multi_key_state();
            }
            KeyCode::Char('G') => {
                self.select_last();
                self.clear_multi_key_state();
            }
            KeyCode::Char('g') => {
                if is_double_tap(self.last_g_press) {
                    self.select_first();
                    self.last_g_press = None;
                } else {
                    self.last_g_press = Some(Instant::now());
                }
                self.last_d_press = None;
            }
            KeyCode::Char('d') => {
                if is_double_tap(self.last_d_press) {
                    if let Some(name) = self.selected_session_name() {
                        self.mode = AppMode::Confirm(ConfirmAction::KillSession(name.clone()));
                        self.status_message = format!("Kill `{name}`? (y/n)");
                    } else {
                        self.status_message = "No session selected".to_string();
                    }
                    self.last_d_press = None;
                } else {
                    self.last_d_press = Some(Instant::now());
                    self.status_message = "Kill session: press d again".to_string();
                }
                self.last_g_press = None;
            }
            KeyCode::Char('D') => {
                if let Some(name) = self.selected_session_name() {
                    match tmux::detach_client(&name).await {
                        Ok(_) => {
                            self.status_message = format!("Detached clients from `{name}`");
                            let _ = self.refresh_sessions().await;
                        }
                        Err(e) => {
                            self.status_message = format!("Failed to detach: {e}");
                        }
                    }
                } else {
                    self.status_message = "No session selected".to_string();
                }
                self.clear_multi_key_state();
            }
            KeyCode::Char('n') => {
                self.mode = AppMode::Input(InputPurpose::NewSession);
                self.input_buffer.clear();
                self.status_message = "Create new session".to_string();
                self.clear_multi_key_state();
            }
            KeyCode::Char('r') => {
                if let Some(name) = self.selected_session_name() {
                    self.mode = AppMode::Input(InputPurpose::RenameSession);
                    self.input_buffer = name;
                    self.status_message = "Rename selected session".to_string();
                } else {
                    self.status_message = "No session selected to rename".to_string();
                }
                self.clear_multi_key_state();
            }
            KeyCode::Enter => {
                if let Some(name) = self.selected_session_name() {
                    if tmux::is_inside_tmux() {
                        match tmux::switch_client(&name).await {
                            Ok(_) => {
                                self.should_quit = true;
                            }
                            Err(e) => {
                                self.status_message = format!("Failed to switch: {e}");
                            }
                        }
                    } else {
                        ratatui::restore();
                        tmux::attach_session_exec(&name);
                    }
                } else {
                    self.status_message = "No session selected".to_string();
                }
                self.clear_multi_key_state();
            }
            KeyCode::Char('/') => {
                self.mode = AppMode::Search;
                self.input_buffer.clear();
                self.status_message = "Search mode".to_string();
                self.clear_multi_key_state();
            }
            KeyCode::Char('?') => {
                self.status_message =
                    "Keys: Enter attach, j/k move, G/gg jump, n new, dd kill, D detach, r rename, / search, q quit"
                        .to_string();
                self.clear_multi_key_state();
            }
            _ => {
                self.clear_multi_key_state();
            }
        }

        Ok(())
    }

    async fn handle_search_mode(&mut self, key: KeyEvent) -> AppResult<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.input_buffer.clear();
                self.status_message = "Search cancelled".to_string();
            }
            KeyCode::Enter => {
                self.mode = AppMode::Normal;
                self.status_message = format!("Search `{}` not yet implemented", self.input_buffer);
                self.input_buffer.clear();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_input_mode(&mut self, key: KeyEvent, purpose: InputPurpose) -> AppResult<()> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.input_buffer.clear();
                self.status_message = "Input cancelled".to_string();
            }
            KeyCode::Enter => {
                let value = self.input_buffer.trim().to_string();
                self.mode = AppMode::Normal;
                self.status_message = match purpose {
                    InputPurpose::NewSession => {
                        if value.is_empty() {
                            "Session name required".to_string()
                        } else {
                            format!("Create `{value}` not yet implemented")
                        }
                    }
                    InputPurpose::RenameSession => {
                        if value.is_empty() {
                            "Session name required".to_string()
                        } else {
                            format!("Rename to `{value}` not yet implemented")
                        }
                    }
                };
                self.input_buffer.clear();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_confirm_mode(&mut self, key: KeyEvent, action: ConfirmAction) -> AppResult<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                self.mode = AppMode::Normal;
                self.status_message = match action {
                    ConfirmAction::KillSession(name) => {
                        format!("Kill `{name}` not yet implemented")
                    }
                };
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.status_message = "Cancelled".to_string();
            }
            _ => {}
        }

        Ok(())
    }

    fn clear_multi_key_state(&mut self) {
        self.last_g_press = None;
        self.last_d_press = None;
    }

    fn selected_session_name(&self) -> Option<String> {
        self.sessions.get(self.selected).map(|session| session.name.clone())
    }

    fn select_next(&mut self) {
        if self.sessions.is_empty() {
            self.selected = 0;
            return;
        }

        self.selected = (self.selected + 1).min(self.sessions.len() - 1);
    }

    fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn select_first(&mut self) {
        self.selected = 0;
    }

    fn select_last(&mut self) {
        if self.sessions.is_empty() {
            self.selected = 0;
            return;
        }

        self.selected = self.sessions.len() - 1;
    }
}

fn is_double_tap(last_press: Option<Instant>) -> bool {
    last_press.is_some_and(|time| time.elapsed() <= DOUBLE_TAP_WINDOW)
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{Event, KeyEventState};

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn make_key_with_kind(code: KeyCode, modifiers: KeyModifiers, kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind,
            state: KeyEventState::NONE,
        }
    }

    fn make_session(name: &str) -> Session {
        Session {
            id: format!("${name}"),
            name: name.to_string(),
            windows: 1,
            attached: 0,
            created: 0,
            last_attached: 0,
            group: None,
            path: "/tmp".to_string(),
        }
    }

    #[test]
    fn test_app_initial_state() {
        let app = App::new();
        assert!(!app.should_quit);
        assert_eq!(app.mode, AppMode::Normal);
        assert_eq!(app.selected, 0);
        assert!(app.sessions.is_empty());
    }

    #[tokio::test]
    async fn test_ignore_key_release_events() {
        let mut app = App::new();
        let release = make_key_with_kind(KeyCode::Char('q'), KeyModifiers::NONE, KeyEventKind::Release);

        app.handle_event(Event::Key(release))
            .await
            .expect("release events should be ignored");

        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn test_app_quit_on_q() {
        let mut app = App::new();
        app.handle_event(Event::Key(make_key(KeyCode::Char('q'), KeyModifiers::NONE)))
            .await
            .expect("q should be handled");
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn test_app_quit_on_ctrl_c() {
        let mut app = App::new();
        app.handle_event(Event::Key(make_key(KeyCode::Char('c'), KeyModifiers::CONTROL)))
            .await
            .expect("ctrl-c should be handled");
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn test_vim_navigation() {
        let mut app = App::new();
        app.sessions = vec![
            make_session("alpha"),
            make_session("beta"),
            make_session("gamma"),
        ];

        app.handle_event(Event::Key(make_key(KeyCode::Char('j'), KeyModifiers::NONE)))
            .await
            .expect("j should move selection down");
        assert_eq!(app.selected, 1);

        app.handle_event(Event::Key(make_key(KeyCode::Char('k'), KeyModifiers::NONE)))
            .await
            .expect("k should move selection up");
        assert_eq!(app.selected, 0);

        app.handle_event(Event::Key(make_key(KeyCode::Char('G'), KeyModifiers::SHIFT)))
            .await
            .expect("G should jump to last");
        assert_eq!(app.selected, 2);

        app.handle_event(Event::Key(make_key(KeyCode::Char('g'), KeyModifiers::NONE)))
            .await
            .expect("first g should arm gg");
        app.handle_event(Event::Key(make_key(KeyCode::Char('g'), KeyModifiers::NONE)))
            .await
            .expect("second g should jump to first");
        assert_eq!(app.selected, 0);
    }

    #[tokio::test]
    async fn test_enter_no_session_selected() {
        let mut app = App::new();
        app.handle_event(Event::Key(make_key(KeyCode::Enter, KeyModifiers::NONE)))
            .await
            .expect("enter with no sessions should be handled");
        assert_eq!(app.status_message, "No session selected");
        assert!(!app.should_quit);
    }

    #[tokio::test]
    async fn test_enter_inside_tmux_switch_fails_gracefully() {
        let mut app = App::new();
        app.sessions = vec![make_session("target")];

        let original = std::env::var("TMUX").ok();
        unsafe { std::env::set_var("TMUX", "/tmp/tmux-fake,99999,0") };

        app.handle_event(Event::Key(make_key(KeyCode::Enter, KeyModifiers::NONE)))
            .await
            .expect("enter inside tmux should be handled");

        assert!(
            app.status_message.contains("Failed to switch")
                || app.should_quit,
            "should either fail gracefully or quit after switch: got '{}'",
            app.status_message
        );

        match original {
            Some(val) => unsafe { std::env::set_var("TMUX", val) },
            None => unsafe { std::env::remove_var("TMUX") },
        }
    }

    #[tokio::test]
    async fn test_detach_no_session() {
        let mut app = App::new();
        app.handle_event(Event::Key(make_key(KeyCode::Char('D'), KeyModifiers::SHIFT)))
            .await
            .expect("D with no sessions should be handled");
        assert_eq!(app.status_message, "No session selected");
    }

    #[tokio::test]
    async fn test_dd_enters_confirm_mode() {
        let mut app = App::new();
        app.sessions = vec![make_session("alpha")];

        app.handle_event(Event::Key(make_key(KeyCode::Char('d'), KeyModifiers::NONE)))
            .await
            .expect("first d should arm dd");
        assert_eq!(app.mode, AppMode::Normal);

        app.handle_event(Event::Key(make_key(KeyCode::Char('d'), KeyModifiers::NONE)))
            .await
            .expect("second d should enter confirm mode");

        assert_eq!(
            app.mode,
            AppMode::Confirm(ConfirmAction::KillSession("alpha".to_string()))
        );
    }
}
