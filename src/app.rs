/// Application state and main loop for tmx.
use std::time::Duration;

use anyhow::Result;
use crossterm::event::Event;
use ratatui::DefaultTerminal;

use crate::event;
use crate::types::AppMode;
use crate::ui;

/// Main application state.
pub struct App {
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Current application mode.
    pub mode: AppMode,
}

impl App {
    /// Create a new App instance.
    pub fn new() -> Self {
        Self {
            should_quit: false,
            mode: AppMode::default(),
        }
    }

    /// Run the main application loop.
    /// Takes ownership of the terminal and runs until quit is requested.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| ui::render(frame, self))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Handle all pending events.
    fn handle_events(&mut self) -> Result<()> {
        // Poll with 50ms timeout for responsive UI
        if let Some(evt) = event::poll_event(Duration::from_millis(50))? {
            match evt {
                Event::Key(key) => {
                    // Only handle key press events (not release/repeat)
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        self.handle_key(key);
                    }
                }
                _ => {} // Ignore mouse, resize, etc. for now
            }
        }
        Ok(())
    }

    /// Handle a single key event.
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if event::is_quit(&key) {
            self.should_quit = true;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_app_initial_state() {
        let app = App::new();
        assert!(!app.should_quit);
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_app_quit_on_q() {
        let mut app = App::new();
        let q_key = make_key(KeyCode::Char('q'), KeyModifiers::NONE);
        app.handle_key(q_key);
        assert!(app.should_quit, "App should quit after 'q' key");
    }

    #[test]
    fn test_app_quit_on_ctrl_c() {
        let mut app = App::new();
        let ctrl_c = make_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
        app.handle_key(ctrl_c);
        assert!(app.should_quit, "App should quit after Ctrl+C");
    }

    #[test]
    fn test_app_no_quit_on_other_key() {
        let mut app = App::new();
        let a_key = make_key(KeyCode::Char('a'), KeyModifiers::NONE);
        app.handle_key(a_key);
        assert!(!app.should_quit, "App should not quit after 'a' key");
    }

    #[test]
    fn test_app_default() {
        let app = App::default();
        assert!(!app.should_quit);
        assert_eq!(app.mode, AppMode::Normal);
    }
}
