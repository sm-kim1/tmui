/// Configuration management for tmx.
/// Handles session tags and groups with XDG TOML persistence.
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Application configuration loaded from/saved to TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub tags: HashMap<String, Vec<String>>, // session_name -> [tag1, tag2]
    #[serde(default)]
    pub groups: HashMap<String, Vec<String>>, // group_name -> [session_name1, ...]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tags: HashMap::new(),
            groups: HashMap::new(),
        }
    }
}

impl Config {
    /// Returns the XDG config file path: ~/.config/tmx/config.toml
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("tmx");
        config_dir.join("config.toml")
    }

    /// Load config from XDG path. Falls back to defaults on parse error.
    /// If the config file is corrupted, renames it to .bak and returns defaults.
    pub fn load() -> Result<Self> {
        Self::load_from(Self::config_path())
    }

    /// Load config from a specific path (for testing).
    pub fn load_from(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            let config = Config::default();
            // Try to create the file with defaults
            let _ = config.save_to(&path);
            return Ok(config);
        }

        let content = std::fs::read_to_string(&path)?;
        match toml::from_str::<Config>(&content) {
            Ok(config) => Ok(config),
            Err(_e) => {
                // Corrupted config: rename to .bak, don't overwrite
                let bak_path = path.with_extension("toml.bak");
                let _ = std::fs::rename(&path, &bak_path);
                Ok(Config::default())
            }
        }
    }

    /// Save config to XDG path.
    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::config_path())
    }

    /// Save config to a specific path (for testing).
    pub fn save_to(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add a tag to a session.
    pub fn add_tag(&mut self, session: &str, tag: &str) {
        let tags = self.tags.entry(session.to_string()).or_default();
        if !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
        }
    }

    /// Remove a tag from a session.
    pub fn remove_tag(&mut self, session: &str, tag: &str) {
        if let Some(tags) = self.tags.get_mut(session) {
            tags.retain(|t| t != tag);
            if tags.is_empty() {
                self.tags.remove(session);
            }
        }
    }

    /// Get tags for a session.
    pub fn get_tags(&self, session: &str) -> Vec<String> {
        self.tags
            .get(session)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all session names that have a given tag.
    pub fn sessions_with_tag(&self, tag: &str) -> Vec<String> {
        self.tags
            .iter()
            .filter(|(_, tags)| tags.contains(&tag.to_string()))
            .map(|(session, _)| session.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_config_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("tmx-test").join(name);
        let _ = fs::create_dir_all(&dir);
        dir.join("config.toml")
    }

    fn cleanup(path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        assert!(cfg.tags.is_empty());
        assert!(cfg.groups.is_empty());
    }

    #[test]
    fn test_config_roundtrip() {
        let path = temp_config_path("roundtrip");
        let _guard = scopeguard(path.clone());

        let mut config = Config::default();
        config.add_tag("work", "important");
        config.add_tag("work", "dev");
        config.add_tag("personal", "home");

        config.save_to(&path).expect("save should succeed");

        let loaded = Config::load_from(path.clone()).expect("load should succeed");
        assert_eq!(loaded.get_tags("work"), vec!["important", "dev"]);
        assert_eq!(loaded.get_tags("personal"), vec!["home"]);
    }

    #[test]
    fn test_xdg_config_path() {
        let path = Config::config_path();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("tmx") && path_str.ends_with("config.toml"),
            "config path should be in tmx dir and named config.toml, got: {path_str}"
        );
    }

    #[test]
    fn test_corrupted_config_fallback() {
        let path = temp_config_path("corrupted");
        let _guard = scopeguard(path.clone());

        // Write invalid TOML
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&path, "{{{{invalid toml content!!!!").expect("write should succeed");

        let config = Config::load_from(path.clone()).expect("load should not crash on corruption");
        assert!(config.tags.is_empty(), "corrupted config should fall back to defaults");

        // Original file should be renamed to .bak
        let bak_path = path.with_extension("toml.bak");
        assert!(bak_path.exists(), "corrupted file should be renamed to .bak");
    }

    #[test]
    fn test_missing_config_creates_default() {
        let path = temp_config_path("missing");
        let _guard = scopeguard(path.clone());

        // Ensure file doesn't exist
        let _ = fs::remove_file(&path);

        let config = Config::load_from(path.clone()).expect("load should succeed for missing config");
        assert!(config.tags.is_empty());

        // File should have been created
        assert!(path.exists(), "missing config should create default file");
    }

    #[test]
    fn test_add_tag_to_session() {
        let mut config = Config::default();
        config.add_tag("work", "important");
        assert_eq!(config.get_tags("work"), vec!["important"]);

        // Adding duplicate should not create duplicate
        config.add_tag("work", "important");
        assert_eq!(config.get_tags("work"), vec!["important"]);

        // Adding different tag
        config.add_tag("work", "dev");
        assert_eq!(config.get_tags("work"), vec!["important", "dev"]);
    }

    #[test]
    fn test_remove_tag() {
        let mut config = Config::default();
        config.add_tag("work", "important");
        config.add_tag("work", "dev");

        config.remove_tag("work", "important");
        assert_eq!(config.get_tags("work"), vec!["dev"]);

        // Remove last tag should remove the session entry
        config.remove_tag("work", "dev");
        assert!(config.get_tags("work").is_empty());
        assert!(!config.tags.contains_key("work"));

        // Removing from nonexistent session should not panic
        config.remove_tag("nonexistent", "tag");
    }

    #[test]
    fn test_filter_by_tag() {
        let mut config = Config::default();
        config.add_tag("work", "important");
        config.add_tag("personal", "important");
        config.add_tag("dev", "coding");

        let important = config.sessions_with_tag("important");
        assert_eq!(important.len(), 2);
        assert!(important.contains(&"work".to_string()));
        assert!(important.contains(&"personal".to_string()));

        let coding = config.sessions_with_tag("coding");
        assert_eq!(coding.len(), 1);
        assert!(coding.contains(&"dev".to_string()));

        let none = config.sessions_with_tag("nonexistent");
        assert!(none.is_empty());
    }

    #[test]
    fn test_config_dir_unwritable() {
        // Use a path that should be unwritable
        let path = PathBuf::from("/proc/tmx-test/config.toml");

        let config = Config::default();
        let result = config.save_to(&path);
        assert!(result.is_err(), "saving to unwritable dir should fail gracefully");
    }

    /// Cleanup helper that removes a temp dir when dropped.
    fn scopeguard(path: PathBuf) -> impl Drop {
        struct Guard(PathBuf);
        impl Drop for Guard {
            fn drop(&mut self) {
                cleanup(&self.0);
            }
        }
        Guard(path)
    }
}
