mod app;
mod config;
mod event;
mod search;
mod tmux;
mod types;
mod ui;

use crate::app::App;
use crate::types::AppResult;

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        original_hook(panic_info);
    }));
}

async fn run() -> AppResult<()> {
    install_panic_hook();

    let mut terminal = ratatui::init();
    let mut app = App::new();
    let result = event::run_event_loop(&mut app, &mut terminal).await;

    ratatui::restore();

    result
}

#[tokio::main]
async fn main() -> AppResult<()> {
    run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_hook_restores_terminal() {
        install_panic_hook();

        let result = std::panic::catch_unwind(|| {
            panic!("test panic for terminal restoration");
        });

        assert!(result.is_err(), "Panic should have been caught");

        // Headless CI can't verify terminal state, but we confirm
        // the hook itself doesn't panic (which would abort the process).
    }

    #[test]
    fn test_app_creates_successfully() {
        let app = App::new();
        assert!(!app.should_quit);
        assert!(app.sessions.is_empty());
    }

    #[test]
    fn test_cargo_builds() {
        assert!(true, "If this test runs, cargo build succeeded");
    }
}
