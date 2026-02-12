use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

use crate::app::App;
use crate::types::AppResult;

pub const TICK_RATE: Duration = Duration::from_millis(250);

pub async fn run_event_loop(app: &mut App, terminal: &mut DefaultTerminal) -> AppResult<()> {
    let mut interval = tokio::time::interval(TICK_RATE);
    let mut events = spawn_event_channel();

    let _ = app.refresh_sessions().await;
    let _ = app.refresh_preview().await;
    terminal.draw(|frame| crate::ui::render(frame, app))?;

    while !app.should_quit {
        tokio::select! {
            _ = interval.tick() => {
                app.tick_clear_errors();
                if let Err(e) = app.refresh_sessions().await {
                    app.set_error(format!("Refresh failed: {e}"));
                }
                let _ = app.refresh_preview().await;
                terminal.draw(|frame| crate::ui::render(frame, app))?;
            }
            maybe_event = events.recv() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        let is_resize = matches!(event, Event::Resize(_, _));
                        let previous_selected = app.selected;
                        if let Err(e) = app.handle_event(event).await {
                            app.set_error(format!("{e}"));
                        }
                        if app.selected != previous_selected || is_resize {
                            let _ = app.refresh_preview().await;
                        }
                        terminal.draw(|frame| crate::ui::render(frame, app))?;
                    }
                    Some(Err(error)) => {
                        return Err(anyhow!(error));
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn spawn_event_channel() -> mpsc::UnboundedReceiver<std::io::Result<Event>> {
    let (sender, receiver) = mpsc::unbounded_channel();

    thread::spawn(move || loop {
        let event = event::read();
        let should_stop = event.is_err();
        if sender.send(event).is_err() {
            break;
        }
        if should_stop {
            break;
        }
    });

    receiver
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_function_exists() {
        let _ = run_event_loop;
    }

    #[test]
    fn test_tick_rate_is_250ms() {
        assert_eq!(TICK_RATE, Duration::from_millis(250));
    }
}
