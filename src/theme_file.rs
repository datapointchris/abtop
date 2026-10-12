//! User themes: `~/.config/abtop/themes/<name>.theme`, so `theme = "<name>"`
//! resolves the way btop's `color_theme` does. The file uses btop's theme
//! format and btop's key names, so a btop theme loads as-is. abtop's own keys
//! are `session_id`, `status_fg`, `warning_fg` and `agent_claude`,
//! `agent_codex`, `agent_opencode`. Missing keys keep the built-in btop theme's
//! colors, and the fallbacks follow btop's own `btop_theme.cpp`.

use std::collections::HashMap;
use std::path::PathBuf;

use ratatui::style::Color;

use crate::theme::{AgentColors, Gradient, Theme, THEME_NAMES};

pub fn themes_dir() -> Option<PathBuf> {
    crate::config::dir().map(|d| d.join("themes"))
}

fn path_of(name: &str) -> Option<PathBuf> {
    themes_dir().map(|d| d.join(format!("{name}.theme")))
}

/// User theme names, sorted, skipping any that a built-in name shadows.
pub fn names() -> Vec<String> {
    let Some(entries) = themes_dir().and_then(|d| std::fs::read_dir(d).ok()) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.extension()? != "theme" {
                return None;
            }
            Some(path.file_stem()?.to_str()?.to_owned())
        })
        .filter(|n| !THEME_NAMES.contains(&n.as_str()))
        .collect();
    names.sort();
    names
}

/// Load `<name>.theme` from the user themes directory.
pub fn load(name: &str) -> Result<Theme, String> {
    let path = path_of(name).ok_or("no config directory")?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    // Theme names are `&'static str`. A file theme loads at startup, on `t`
    // and on SIGUSR2, so leaking its name stays bounded.
    let name: &'static str = Box::leak(name.to_owned().into_boxed_str());
    Ok(build(name, &parse(&text)))
}

/// `theme[key]="value"` lines, as btop reads them. Values may be empty.
fn parse(text: &str) -> HashMap<&str, &str> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('#') {
                return None;
            }
            let (key, rest) = line.strip_prefix("theme[")?.split_once(']')?;
            let value = rest.trim_start().strip_prefix('=')?.trim();
            let value = match value.strip_prefix('"') {
                Some(quoted) => quoted.split_once('"').map_or(quoted, |(v, _)| v),
                None => value,
            };
            Some((key, value))
        })
        .collect()
}

/// `#RRGGBB`, `#GG` (grey) or `R G B` decimal, the forms btop accepts.
fn rgb(value: &str) -> Option<(u8, u8, u8)> {
    if let Some(hex) = value.strip_prefix('#') {
        let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok();
        return match hex.len() {
            2 => byte(0).map(|g| (g, g, g)),
            6 => Some((byte(0)?, byte(2)?, byte(4)?)),
            _ => None,
        };
    }
    let parts: Vec<u8> = value
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    match parts[..] {
        [r, g, b] => Some((r, g, b)),
        _ => None,
    }
}

fn build(name: &'static str, values: &HashMap<&str, &str>) -> Theme {
    let base = Theme::btop();
    let color = |key: &str, default: Color| {
        values
            .get(key)
            .and_then(|v| rgb(v))
            .map_or(default, |(r, g, b)| Color::Rgb(r, g, b))
    };
    let inactive_fg = color("inactive_fg", base.inactive_fg);
    let cpu_grad = gradient(values, "cpu", &base.cpu_grad);
    let has_process = values.get("process_start").and_then(|v| rgb(v)).is_some();
    Theme {
        name,
        main_bg: match values.get("main_bg") {
            Some(&"") => Color::Reset,
            _ => color("main_bg", base.main_bg),
        },
        main_fg: color("main_fg", base.main_fg),
        title: color("title", base.title),
        hi_fg: color("hi_fg", base.hi_fg),
        selected_bg: color("selected_bg", base.selected_bg),
        selected_fg: color("selected_fg", base.selected_fg),
        inactive_fg,
        graph_text: color("graph_text", inactive_fg),
        meter_bg: color("meter_bg", inactive_fg),
        proc_misc: color("proc_misc", base.proc_misc),
        div_line: color("div_line", base.div_line),
        session_id: color("session_id", base.session_id),
        status_fg: color("status_fg", base.status_fg),
        warning_fg: color("warning_fg", base.warning_fg),
        cpu_box: color("cpu_box", base.cpu_box),
        mem_box: color("mem_box", base.mem_box),
        net_box: color("net_box", base.net_box),
        proc_box: color("proc_box", base.proc_box),
        agent_colors: AgentColors {
            claude: color("agent_claude", base.agent_colors.claude),
            codex: color("agent_codex", base.agent_colors.codex),
            opencode: color("agent_opencode", base.agent_colors.opencode),
        },
        proc_grad: if has_process {
            gradient(values, "process", &cpu_grad)
        } else {
            gradient(values, "cpu", &base.cpu_grad)
        },
        cpu_grad,
        used_grad: gradient(values, "used", &base.used_grad),
        free_grad: gradient(values, "free", &base.free_grad),
        cached_grad: gradient(values, "cached", &base.cached_grad),
    }
}

/// btop's gradient rules: a missing stop keeps the default, an empty `_mid`
/// ramps straight from start to end, and an empty `_end` paints start solid.
fn gradient(values: &HashMap<&str, &str>, prefix: &str, default: &Gradient) -> Gradient {
    let stop = |suffix: &str, fallback: (u8, u8, u8)| match values
        .get(format!("{prefix}_{suffix}").as_str())
    {
        Some(&"") => None,
        Some(v) => Some(rgb(v).unwrap_or(fallback)),
        None => Some(fallback),
    };
    let start = stop("start", default.start).unwrap_or(default.start);
    let Some(end) = stop("end", default.end) else {
        return Gradient { start, mid: start, end: start };
    };
    // abtop always interpolates through a mid; the midpoint gives the same ramp.
    let mid = stop("mid", default.mid).unwrap_or((
        ((start.0 as u16 + end.0 as u16) / 2) as u8,
        ((start.1 as u16 + end.1 as u16) / 2) as u8,
        ((start.2 as u16 + end.2 as u16) / 2) as u8,
    ));
    Gradient { start, mid, end }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btop_format_rules_and_abtop_keys() {
        let text = r##"
# comment
theme[main_bg]=""
theme[main_fg]="#E6D5C2"
theme[title]="#40"
theme[hi_fg]="10 20 30"
theme[inactive_fg]="#928374"
theme[cpu_start]="#000000"
theme[cpu_mid]=""
theme[cpu_end]="#64C832"
theme[used_start]="#FF0000"
theme[used_end]=""
theme[status_fg]="#CC5351"
theme[agent_claude]="#D79921"
"##;
        let t = build("sample", &parse(text));
        assert_eq!(t.main_bg, Color::Reset);
        assert_eq!(t.main_fg, Color::Rgb(0xE6, 0xD5, 0xC2));
        assert_eq!(t.title, Color::Rgb(0x40, 0x40, 0x40));
        assert_eq!(t.hi_fg, Color::Rgb(10, 20, 30));
        assert_eq!(t.meter_bg, Color::Rgb(0x92, 0x83, 0x74));
        assert_eq!(t.cpu_grad.mid, (50, 100, 25));
        assert_eq!(t.proc_grad.end, t.cpu_grad.end);
        assert_eq!((t.used_grad.mid, t.used_grad.end), ((255, 0, 0), (255, 0, 0)));
        assert_eq!(t.status_fg, Color::Rgb(0xCC, 0x53, 0x51));
        assert_eq!(t.agent_colors.claude, Color::Rgb(0xD7, 0x99, 0x21));
        assert_eq!(t.warning_fg, Theme::btop().warning_fg);
    }
}
