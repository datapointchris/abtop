use std::path::PathBuf;

#[derive(Clone, Copy)]
pub struct PanelVisibility {
    pub context: bool,
    pub quota: bool,
    pub tokens: bool,
    pub projects: bool,
    pub ports: bool,
    pub sessions: bool,
    pub mcp: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            context: true,
            quota: true,
            tokens: true,
            projects: true,
            ports: true,
            sessions: true,
            mcp: true,
        }
    }
}

pub struct AppConfig {
    pub theme: String,
    /// Agent CLI names to exclude from the TUI (e.g. ["codex"] to hide Codex).
    /// Matched case-insensitively against each collector's agent_cli identifier.
    pub hidden_agents: Vec<String>,
    /// Additional Claude config directories to scan for sessions.
    /// Useful for multi-profile setups that use separate CLAUDE_CONFIG_DIR roots.
    pub claude_config_dirs: Vec<PathBuf>,
    pub panels: PanelVisibility,
    /// UI language override. Empty string means auto-detect from `LANG`.
    /// Recognized values: "en", "zh" (anything starting with "zh" maps to Simplified Chinese).
    pub language: String,
    /// `--model` for the `claude --print` session-title call. Empty string
    /// leaves the choice to Claude Code.
    pub summary_model: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "btop".to_string(),
            hidden_agents: Vec::new(),
            claude_config_dirs: Vec::new(),
            panels: PanelVisibility::default(),
            language: String::new(),
            summary_model: String::new(),
        }
    }
}

/// `~/.config/abtop` on macOS too, matching Linux and the README.
pub fn dir() -> Option<PathBuf> {
    #[cfg(unix)]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")));
    #[cfg(not(unix))]
    let base = dirs::config_dir();
    base.map(|d| d.join("abtop"))
}

fn config_path() -> Option<PathBuf> {
    dir().map(|d| d.join("config.toml"))
}

pub fn load_config() -> AppConfig {
    let path = match config_path() {
        Some(p) => p,
        None => return AppConfig::default(),
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return AppConfig::default(),
    };

    parse_config_body(&content)
}

fn parse_config_body(content: &str) -> AppConfig {
    let mut config = AppConfig::default();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            // Strip quotes (double or single) and inline comments
            let val = val.trim();
            let val = if let Some(comment_pos) = val.find('#') {
                val[..comment_pos].trim()
            } else {
                val
            };
            if key == "hidden_agents" {
                config.hidden_agents = parse_string_array(val);
                continue;
            }
            if key == "claude_config_dirs" {
                config.claude_config_dirs = parse_path_array(val);
                continue;
            }
            let val = val.trim_matches('"').trim_matches('\'');
            match key {
                "theme" => config.theme = val.to_string(),
                "language" => config.language = val.to_string(),
                "summary_model" => config.summary_model = val.to_string(),
                "show_context" => config.panels.context = parse_bool(val).unwrap_or(true),
                "show_quota" => config.panels.quota = parse_bool(val).unwrap_or(true),
                "show_tokens" => config.panels.tokens = parse_bool(val).unwrap_or(true),
                "show_projects" => config.panels.projects = parse_bool(val).unwrap_or(true),
                "show_ports" => config.panels.ports = parse_bool(val).unwrap_or(true),
                "show_sessions" => config.panels.sessions = parse_bool(val).unwrap_or(true),
                "show_mcp" => config.panels.mcp = parse_bool(val).unwrap_or(true),
                _ => {}
            }
        }
    }
    config
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Parse a simple one-line TOML string array like `["a", "b"]`.
/// Returns an empty Vec for malformed input to keep config loading infallible.
fn parse_string_array(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
        return Vec::new();
    };
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_path_array(raw: &str) -> Vec<PathBuf> {
    parse_string_array(raw)
        .into_iter()
        .map(|s| expand_home_path(&s))
        .collect()
}

fn expand_home_path(raw: &str) -> PathBuf {
    if raw == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_string_array_basic() {
        assert_eq!(parse_string_array(r#"["codex"]"#), vec!["codex"]);
        assert_eq!(
            parse_string_array(r#"["codex", "claude"]"#),
            vec!["codex", "claude"]
        );
    }

    #[test]
    fn parse_string_array_quote_styles_and_whitespace() {
        assert_eq!(
            parse_string_array(r#"[ 'codex' , "claude" ]"#),
            vec!["codex", "claude"]
        );
    }

    #[test]
    fn parse_string_array_empty_and_malformed() {
        assert!(parse_string_array("[]").is_empty());
        assert!(parse_string_array("not an array").is_empty());
        assert!(parse_string_array(r#"["a",,]"#)
            .iter()
            .all(|s| !s.is_empty()));
    }

    #[test]
    fn parse_path_array_expands_home_relative_entries() {
        let home = dirs::home_dir().unwrap();
        let paths = parse_path_array(r#"["~/.claude-personal", "/tmp/.claude-work"]"#);

        assert_eq!(paths[0], home.join(".claude-personal"));
        assert_eq!(paths[1], PathBuf::from("/tmp/.claude-work"));
    }

    #[test]
    fn parse_config_body_loads_claude_config_dirs() {
        let home = dirs::home_dir().unwrap();
        let cfg = parse_config_body(r#"claude_config_dirs = ["~/.claude-personal"]"#);

        assert_eq!(cfg.claude_config_dirs, vec![home.join(".claude-personal")]);
    }

    #[test]
    fn parse_bool_round_trips_visibility_keys() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("False"), Some(false));
        assert_eq!(parse_bool("nope"), None);
    }
}
