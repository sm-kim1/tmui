/// Event handling for tmx.
/// Provides crossterm event polling with async support.

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

/// Poll for a crossterm event with a timeout.
/// Returns `None` if no event is available within the timeout.
pub fn poll_event(timeout: Duration) -> Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

/// Check if a key event represents a quit action.
pub fn is_quit(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } | KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_quit_on_q() {
        let q_key = make_key(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(is_quit(&q_key), "'q' key should trigger quit");
    }

    #[test]
    fn test_quit_on_ctrl_c() {
        let ctrl_c = make_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(is_quit(&ctrl_c), "Ctrl+C should trigger quit");
    }

    #[test]
    fn test_no_quit_on_other_keys() {
        let a_key = make_key(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!is_quit(&a_key), "'a' key should not trigger quit");

        let enter_key = make_key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(!is_quit(&enter_key), "Enter should not trigger quit");
    }

    #[test]
    fn test_no_quit_on_shift_q() {
        // Shift+Q (uppercase Q) should not quit
        let shift_q = make_key(KeyCode::Char('q'), KeyModifiers::SHIFT);
        assert!(!is_quit(&shift_q), "Shift+Q should not trigger quit");
    }
}
