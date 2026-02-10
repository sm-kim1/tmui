/// Configuration management for tmx.
/// Stub module — will be implemented in a later task.

use serde::Deserialize;

/// Application configuration loaded from TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Placeholder — will be populated in config task
}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        // Just ensure it constructs without panic
        let _ = format!("{:?}", cfg);
    }
}
