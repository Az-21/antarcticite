use crate::core::config::{Config, DefaultFallback, Rule};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use url::Url;

pub enum RouteResult<'a> {
    Matched(&'a Rule),
    Fallback(&'a DefaultFallback),
}

static DEBOUNCE_CACHE: Lazy<Mutex<HashMap<String, Instant>>> = Lazy::new(|| Mutex::new(HashMap::new()));
const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);

/// Routes a URL string against a configuration, returning the matching rule or the default fallback.
pub fn route_url<'a>(url_str: &str, config: &'a Config) -> Result<RouteResult<'a>> {
    let url = Url::parse(url_str).context("Failed to parse URL")?;
    let domain = url.host_str().unwrap_or("");

    for rule in &config.rules {
        // Exact domain match
        if let Some(match_domain) = &rule.match_domain {
            if domain == match_domain {
                debug!("Matched exact domain: {}", match_domain);
                return Ok(RouteResult::Matched(rule));
            }
        }

        // Pattern match
        if let Some(match_pattern) = &rule.match_pattern {
            match Regex::new(match_pattern) {
                Ok(re) => {
                    if re.is_match(url_str) {
                        debug!("Matched regex pattern: {}", match_pattern);
                        return Ok(RouteResult::Matched(rule));
                    }
                }
                Err(e) => {
                    warn!("Invalid regex pattern in rule '{}': {}", match_pattern, e);
                }
            }
        }
    }

    debug!("No rule matched, using default fallback.");
    Ok(RouteResult::Fallback(&config.default))
}

fn get_browser_command(browser: &str, profile: Option<&str>, url: &str) -> Command {
    // Basic normalization
    let browser_lower = browser.to_lowercase();
    
    // We determine the actual executable and profile flag based on OS and browser type
    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        
        // Translate some common application IDs or names for macOS
        let app_name = match browser_lower.as_str() {
            "chrome" | "com.google.chrome" | "google chrome" => "Google Chrome",
            "edge" | "com.microsoft.edge" | "microsoft edge" => "Microsoft Edge",
            "firefox" | "org.mozilla.firefox" => "Firefox",
            "safari" | "com.apple.safari" => "Safari",
            "brave" | "com.brave.browser" => "Brave Browser",
            _ => browser, // Fallback to provided name
        };

        cmd.arg("-a").arg(app_name);

        if let Some(p) = profile {
            if !p.is_empty() {
                cmd.arg("--args");
                // Different browsers have different profile flags
                if app_name == "Firefox" {
                    cmd.arg("-P").arg(p);
                } else {
                    // Chromium based (Chrome, Edge, Brave)
                    cmd.arg(format!("--profile-directory={}", p));
                }
            }
        }
        cmd.arg(url);
        cmd
    }

    #[cfg(target_os = "linux")]
    {
        let executable = match browser_lower.as_str() {
            "chrome" | "com.google.chrome" | "google chrome" => "google-chrome",
            "edge" | "com.microsoft.edge" | "microsoft edge" => "microsoft-edge",
            "firefox" | "org.mozilla.firefox" => "firefox",
            "brave" | "com.brave.browser" => "brave-browser",
            _ => browser,
        };

        let mut cmd = Command::new(executable);

        if let Some(p) = profile {
            if !p.is_empty() {
                if executable == "firefox" {
                    cmd.arg("-P").arg(p);
                } else {
                    cmd.arg(format!("--profile-directory={}", p));
                }
            }
        }
        cmd.arg(url);
        cmd
    }

    #[cfg(target_os = "windows")]
    {
        let executable = match browser_lower.as_str() {
            "chrome" | "com.google.chrome" | "google chrome" => "chrome.exe",
            "edge" | "com.microsoft.edge" | "microsoft edge" => "msedge.exe",
            "firefox" | "org.mozilla.firefox" => "firefox.exe",
            "brave" | "com.brave.browser" => "brave.exe",
            _ => browser,
        };

        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("start").arg("").arg(executable);

        if let Some(p) = profile {
            if !p.is_empty() {
                if executable == "firefox.exe" {
                    cmd.arg("-P").arg(p);
                } else {
                    cmd.arg(format!("--profile-directory={}", p));
                }
            }
        }
        cmd.arg(url);
        cmd
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let mut cmd = Command::new(browser);
        cmd.arg(url);
        cmd
    }
}

pub fn open_url(url_str: &str, config: &Config) -> Result<()> {
    {
        let mut cache = DEBOUNCE_CACHE.lock().unwrap();
        let now = Instant::now();
        
        // Clean up old entries
        cache.retain(|_, time| now.duration_since(*time) < DEBOUNCE_DURATION);

        if let Some(&last_time) = cache.get(url_str) {
            if now.duration_since(last_time) < DEBOUNCE_DURATION {
                info!("Debounced duplicate request for URL: {}", url_str);
                return Ok(()); // Silently ignore duplicate clicks within 500ms
            }
        }
        
        cache.insert(url_str.to_string(), now);
    }

    info!("Routing request for URL: {}", url_str);
    let route_res = route_url(url_str, config)?;

    let (browser, profile) = match route_res {
        RouteResult::Matched(rule) => {
            info!("Matched rule mapping to browser '{}', profile '{:?}'", rule.target_browser, rule.target_profile);
            (rule.target_browser.as_str(), rule.target_profile.as_deref())
        }
        RouteResult::Fallback(fallback) => {
            info!("Fell back to default browser '{}', profile '{:?}'", fallback.browser, fallback.profile);
            (fallback.browser.as_str(), fallback.profile.as_deref())
        }
    };

    let mut cmd = get_browser_command(browser, profile, url_str);
    
    debug!("Executing browser command: {:?}", cmd);
    
    // Spawn disowns the process, which is exactly what we want for launching a browser
    match cmd.spawn() {
        Ok(_) => {
            info!("Successfully launched browser.");
            Ok(())
        }
        Err(e) => {
            error!("Failed to launch browser: {}", e);
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_config() -> Config {
        Config {
            default: DefaultFallback {
                browser: "firefox".to_string(),
                profile: None,
            },
            rules: vec![
                Rule {
                    match_domain: Some("www.clientx.com".to_string()),
                    match_pattern: None,
                    target_browser: "chrome".to_string(),
                    target_profile: Some("Profile 1".to_string()),
                },
                Rule {
                    match_domain: None,
                    match_pattern: Some(r".*\.clienty\.com".to_string()),
                    target_browser: "edge".to_string(),
                    target_profile: Some("Work".to_string()),
                },
            ],
            redirect_policies: vec![],
        }
    }

    #[test]
    fn test_route_url_exact_domain() {
        let config = get_test_config();
        let res = route_url("https://www.clientx.com/dashboard", &config).unwrap();
        match res {
            RouteResult::Matched(rule) => {
                assert_eq!(rule.target_browser, "chrome");
            }
            _ => panic!("Expected matched rule"),
        }
    }

    #[test]
    fn test_route_url_pattern() {
        let config = get_test_config();
        let res = route_url("https://app.clienty.com/login", &config).unwrap();
        match res {
            RouteResult::Matched(rule) => {
                assert_eq!(rule.target_browser, "edge");
            }
            _ => panic!("Expected matched rule"),
        }
    }

    #[test]
    fn test_route_url_fallback() {
        let config = get_test_config();
        let res = route_url("https://www.google.com", &config).unwrap();
        match res {
            RouteResult::Fallback(fb) => {
                assert_eq!(fb.browser, "firefox");
            }
            _ => panic!("Expected fallback"),
        }
    }

    #[test]
    fn test_route_url_invalid() {
        let config = get_test_config();
        let res = route_url("not_a_valid_url", &config);
        assert!(res.is_err());
    }

    #[test]
    fn test_browser_command_formatting_macos() {
        #[cfg(target_os = "macos")]
        {
            let cmd = get_browser_command("chrome", Some("Default"), "https://example.com");
            let cmd_str = format!("{:?}", cmd);
            assert!(cmd_str.contains("Google Chrome"));
            assert!(cmd_str.contains("--profile-directory=Default"));
            assert!(cmd_str.contains("https://example.com"));

            let cmd_ff = get_browser_command("firefox", Some("dev"), "https://example.com");
            let cmd_ff_str = format!("{:?}", cmd_ff);
            assert!(cmd_ff_str.contains("Firefox"));
            assert!(cmd_ff_str.contains("-P"));
            assert!(cmd_ff_str.contains("dev"));
        }
    }

    #[test]
    fn test_browser_command_formatting_linux() {
        #[cfg(target_os = "linux")]
        {
            let cmd = get_browser_command("chrome", Some("Default"), "https://example.com");
            let cmd_str = format!("{:?}", cmd);
            assert!(cmd_str.contains("google-chrome"));
            assert!(cmd_str.contains("--profile-directory=Default"));

            let cmd_ff = get_browser_command("firefox", Some("dev"), "https://example.com");
            let cmd_ff_str = format!("{:?}", cmd_ff);
            assert!(cmd_ff_str.contains("firefox"));
            assert!(cmd_ff_str.contains("-P"));
            assert!(cmd_ff_str.contains("dev"));
        }
    }

    #[test]
    fn test_browser_command_formatting_windows() {
        #[cfg(target_os = "windows")]
        {
            let cmd = get_browser_command("chrome", Some("Default"), "https://example.com");
            let cmd_str = format!("{:?}", cmd);
            assert!(cmd_str.contains("chrome.exe"));
            assert!(cmd_str.contains("--profile-directory=Default"));

            let cmd_ff = get_browser_command("firefox", Some("dev"), "https://example.com");
            let cmd_ff_str = format!("{:?}", cmd_ff);
            assert!(cmd_ff_str.contains("firefox.exe"));
            assert!(cmd_ff_str.contains("-P"));
            assert!(cmd_ff_str.contains("dev"));
        }
    }
}
