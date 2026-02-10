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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_mode_default_is_normal() {
        assert_eq!(AppMode::default(), AppMode::Normal);
    }
}
