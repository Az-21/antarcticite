use anyhow::{Context, Result};
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use tracing::{error, info, warn};

use crate::os::notifications::show_notification;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DefaultFallback {
    pub browser: String,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub match_domain: Option<String>,
    pub match_pattern: Option<String>,
    pub target_browser: String,
    pub target_profile: Option<String>,
}

impl Rule {
    pub fn is_valid(&self) -> bool {
        self.match_domain.is_some() || self.match_pattern.is_some()
    }
}

fn default_timeout() -> u32 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedirectPolicy {
    pub match_domain: String,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub default: DefaultFallback,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub redirect_policies: Vec<RedirectPolicy>,
}

// In-memory cache for the last known good configuration
static LAST_GOOD_CONFIG: Lazy<RwLock<Option<Config>>> = Lazy::new(|| RwLock::new(None));

pub fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "antarcticite", "router")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}

pub fn get_backup_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "antarcticite", "router")
        .map(|dirs| dirs.data_local_dir().join("config.backup.toml"))
}

pub fn load_config() -> Result<Config> {
    let config_path = get_config_path().context("Could not determine config directory")?;

    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Configuration file not found at {:?}",
            config_path
        ));
    }

    let config_content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            let msg = format!("Failed to read config file: {}", e);
            error!("{}", msg);
            show_notification("Config Error", &msg);
            return fallback_to_last_good();
        }
    };

    match toml::from_str::<Config>(&config_content) {
        Ok(config) => {
            // Validate rules
            let invalid_rules: Vec<_> = config.rules.iter().filter(|r| !r.is_valid()).collect();
            if !invalid_rules.is_empty() {
                warn!(
                    "Found {} invalid rules in configuration. Rules must have either match_domain or match_pattern.",
                    invalid_rules.len()
                );
            }

            // Save to memory cache
            if let Ok(mut cache) = LAST_GOOD_CONFIG.write() {
                *cache = Some(config.clone());
            }

            // Save to disk backup
            if let Some(backup_path) = get_backup_config_path() {
                if let Some(parent) = backup_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if let Ok(serialized) = toml::to_string(&config) {
                    let _ = fs::write(backup_path, serialized);
                }
            }

            Ok(config)
        }
        Err(e) => {
            let msg = format!("Failed to parse config.toml: {}", e);
            error!("{}", msg);
            show_notification(
                "Config Parsing Error",
                "Invalid config file format. Using fallback.",
            );
            fallback_to_last_good()
        }
    }
}

fn fallback_to_last_good() -> Result<Config> {
    // Try memory cache first
    if let Ok(cache) = LAST_GOOD_CONFIG.read()
        && let Some(config) = cache.as_ref()
    {
        info!("Using in-memory cached configuration.");
        return Ok(config.clone());
    }

    // Try disk backup next
    if let Some(backup_path) = get_backup_config_path()
        && backup_path.exists()
        && let Ok(content) = fs::read_to_string(&backup_path)
        && let Ok(config) = toml::from_str::<Config>(&content)
    {
        info!("Using on-disk backup configuration.");
        // Populate memory cache
        if let Ok(mut cache) = LAST_GOOD_CONFIG.write() {
            *cache = Some(config.clone());
        }
        return Ok(config);
    }

    Err(anyhow::anyhow!(
        "No valid configuration available, and no fallback found."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let toml_str = r#"
[default]
browser = "com.google.chrome"
profile = "Default"

[[rules]]
match_domain = "www.clientX.com"
target_browser = "com.google.chrome"
target_profile = "Profile 1"

[[rules]]
match_pattern = ".*\\.clientY\\.com"
target_browser = "firefox"
target_profile = "Work"

[[redirect_policies]]
match_domain = "protect-eu.mimecast.com"
timeout_seconds = 5
"#;
        let config: Config = toml::from_str(toml_str).expect("Failed to parse valid config");
        assert_eq!(config.default.browser, "com.google.chrome");
        assert_eq!(config.default.profile, Some("Default".to_string()));

        assert_eq!(config.rules.len(), 2);
        assert_eq!(
            config.rules[0].match_domain,
            Some("www.clientX.com".to_string())
        );
        assert_eq!(
            config.rules[1].match_pattern,
            Some(".*\\.clientY\\.com".to_string())
        );
        assert!(config.rules[0].is_valid());

        assert_eq!(config.redirect_policies.len(), 1);
        assert_eq!(config.redirect_policies[0].timeout_seconds, 5);
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
[default]
browser = "firefox"
"#;
        let config: Config = toml::from_str(toml_str).expect("Failed to parse minimal config");
        assert_eq!(config.default.browser, "firefox");
        assert_eq!(config.default.profile, None);
        assert_eq!(config.rules.len(), 0);
    }

    #[test]
    fn test_parse_invalid_config() {
        let toml_str = r#"
[default]
# missing browser
profile = "Default"
"#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_rule_validation() {
        let valid_rule_1 = Rule {
            match_domain: Some("example.com".to_string()),
            match_pattern: None,
            target_browser: "chrome".to_string(),
            target_profile: None,
        };
        assert!(valid_rule_1.is_valid());

        let valid_rule_2 = Rule {
            match_domain: None,
            match_pattern: Some(".*".to_string()),
            target_browser: "chrome".to_string(),
            target_profile: None,
        };
        assert!(valid_rule_2.is_valid());

        let invalid_rule = Rule {
            match_domain: None,
            match_pattern: None,
            target_browser: "chrome".to_string(),
            target_profile: None,
        };
        assert!(!invalid_rule.is_valid());
    }
}
