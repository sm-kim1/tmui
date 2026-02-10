/// Core types for tmx application.

/// Represents the current mode of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Normal browsing mode
    Normal,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Result type alias using anyhow for error handling.
pub type AppResult<T> = anyhow::Result<T>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub windows: usize,
    pub attached: usize,
    pub created: i64,
    pub last_attached: i64,
    pub group: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub id: String,
    pub session_id: String,
    pub index: usize,
    pub name: String,
    pub active: bool,
    pub active_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    pub id: String,
    pub window_id: String,
    pub session_id: String,
    pub index: usize,
    pub active: bool,
    pub current_command: String,
    pub current_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_mode_default_is_normal() {
        assert_eq!(AppMode::default(), AppMode::Normal);
    }

    #[test]
    fn test_session_struct_fields() {
        let session = Session {
            id: "$0".to_string(),
            name: "work".to_string(),
            windows: 2,
            attached: 1,
            created: 1770744224,
            last_attached: 1770749593,
            group: None,
            path: "/tmp".to_string(),
        };

        assert_eq!(session.name, "work");
    }

    #[test]
    fn test_window_struct_fields() {
        let window = Window {
            id: "@0".to_string(),
            session_id: "$0".to_string(),
            index: 0,
            name: "editor".to_string(),
            active: true,
            active_command: "vim".to_string(),
        };

        assert_eq!(window.name, "editor");
    }

    #[test]
    fn test_pane_struct_fields() {
        let pane = Pane {
            id: "%0".to_string(),
            window_id: "@0".to_string(),
            session_id: "$0".to_string(),
            index: 0,
            active: true,
            current_command: "bash".to_string(),
            current_path: "/tmp".to_string(),
        };

        assert_eq!(pane.current_command, "bash");
    }
}
