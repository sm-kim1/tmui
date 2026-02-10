/// UI rendering module for tmx.
/// Currently renders an empty screen — widgets come in later tasks.

use ratatui::Frame;

use crate::app::App;

/// Render the application UI to the given frame.
pub fn render(frame: &mut Frame, _app: &App) {
    // Empty render — no widgets yet.
    // Just clear the frame area (ratatui clears by default).
    let _area = frame.area();
}

#[cfg(test)]
mod tests {
    // UI rendering tests require a terminal backend.
    // We verify the render function signature compiles correctly.
    // Integration testing done via QA scenarios.

    #[test]
    fn test_render_function_exists() {
        // Compile-time check that the module and function exist
        let _ = super::render as fn(&mut ratatui::Frame, &crate::app::App);
    }
}
