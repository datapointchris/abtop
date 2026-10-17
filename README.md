# abtop

**Like [btop](https://github.com/aristocratos/btop), but for your AI coding agents.**

> This is a fork of [graykode/abtop](https://github.com/graykode/abtop). It adds
> [theme files](#theme-files) in btop's format, a `summary_model` setting with a
> lighter `claude --print` call for session titles, and a quota panel that drops
> the columns of agents in `hidden_agents`. Everything else follows upstream.

See every Claude Code, Codex CLI, and OpenCode session at a glance — token usage, context window %, rate limits, child processes, open ports, and more.
Claude Code, Codex CLI, and OpenCode sessions are discovered from local process/file state, so multiple active profiles are supported across macOS, Linux, and Windows.

![demo](https://raw.githubusercontent.com/graykode/abtop/main/assets/demo.gif)

## Why

- Running 3+ agents across projects? See them all in one screen.
- Hitting rate limits? Watch your quota in real-time.
- Agent spawned a server and forgot to kill it? Orphan port detection.
- Context window filling up? Per-session % bars with warnings.

All read-only. No API keys. No auth.

## Install

### macOS / Linux

> [!IMPORTANT]
> On Linux, ensure `sqlite3` is installed to enable monitoring for OpenCode sessions.

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/datapointchris/abtop/releases/latest/download/abtop-installer.sh | sh
```

### Cargo

```bash
cargo install --git https://github.com/datapointchris/abtop
```

### Windows

Native support — no WSL required. Uses `sysinfo` for process info and host CPU/MEM metrics, and `netstat -ano` for listening ports. Windows has no load average, so LOAD is reported as 0. OpenCode session discovery additionally requires the `sqlite3` CLI (`winget install SQLite.SQLite`); without it abtop prints a one-time warning to stderr.

```powershell
powershell -c "irm https://github.com/datapointchris/abtop/releases/latest/download/abtop-installer.ps1 | iex"
```

Or `cargo install --git https://github.com/datapointchris/abtop` from any terminal with Git in PATH. Claude Code config is resolved automatically from `%USERPROFILE%\.claude`.

### Other

Pre-built binaries for all platforms are available on the [GitHub Releases](https://github.com/datapointchris/abtop/releases) page.

## Usage

```bash
abtop                    # Launch TUI
abtop --once             # Print snapshot and exit
abtop --json             # Print one JSON snapshot and exit (for scripts/tools)
abtop --setup            # Install rate limit collection hook
abtop --theme dracula    # Launch with a specific theme
abtop --mouse            # Enable mouse click/scroll navigation
```

Recommended terminal size: **120x40** or larger. Minimum 80x24 — panels hide gracefully when small.
Mouse capture is off by default so terminal drag selection and copy keep working. Launch with `--mouse` if you prefer click targets and wheel navigation.

### Terminal Jump

Press `Enter` to focus the terminal running the selected agent. abtop supports cmux, tmux, and iTerm2 on macOS.

```bash
tmux new -s work
# pane 0: abtop
# pane 1: claude (project A)
# pane 2: claude (project B)
# → Enter on a session in abtop jumps to its pane
```

## Supported Agents

| Feature           | Claude Code | Codex CLI | OpenCode |
| ----------------- | :---------: | :-------: | :------: |
| Session Discovery |     ✅      |    ✅     |    ✅    |
| Token Tracking    |     ✅      |    ✅     |    ✅    |
| Context Window %  |     ✅      |    ✅     |    ❌    |
| Status Detection  |     ✅      |    ✅     |    ✅    |
| Current Task      |     ✅      |    ✅     |    ❌    |
| Rate Limit        |     ✅      |    ✅     |    ❌    |
| Git Status        |     ✅      |    ✅     |    ✅    |
| Children / Ports  |     ✅      |    ✅     |    ✅    |
| Subagents         |     ✅      |    ❌     |    ❌    |
| Memory Status     |     ✅      |    ❌     |    ❌    |

OpenCode support reads the local SQLite database at `~/.local/share/opencode/opencode.db` (also the default location on Windows; `%LOCALAPPDATA%\opencode` and `%APPDATA%\opencode` are probed as fallbacks) and requires `sqlite3` in `PATH` (on Windows: `winget install SQLite.SQLite`).

## Themes

12 built-in themes, including 4 colorblind-friendly options (`high-contrast`, `protanopia`, `deuteranopia`, `tritanopia`). Press `t` to cycle at runtime, or launch with `--theme <name>`. Set `theme` in `~/.config/abtop/config.toml` to keep one.

| btop (default) | dracula | catppuccin |
|:-:|:-:|:-:|
| ![btop](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/btop.png) | ![dracula](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/dracula.png) | ![catppuccin](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/catppuccin.png) |

| tokyo-night | gruvbox | nord |
|:-:|:-:|:-:|
| ![tokyo-night](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/tokyo-night.png) | ![gruvbox](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/gruvbox.png) | ![nord](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/nord.png) |

Colorblind-friendly themes:

| high-contrast | protanopia |
|:-:|:-:|
| ![high-contrast](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/high-contrast.png) | ![protanopia](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/protanopia.png) |

| deuteranopia | tritanopia |
|:-:|:-:|
| ![deuteranopia](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/deuteranopia.png) | ![tritanopia](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/tritanopia.png) |

Light themes (`light` — Solarized cream, `white` — GitHub-style pure white) for bright terminals:

| light | white |
|:-:|:-:|
| ![light](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/light.png) | ![white](https://raw.githubusercontent.com/graykode/abtop/main/assets/themes/white.png) |

### Theme files

`theme = "<name>"` also resolves to `~/.config/abtop/themes/<name>.theme`, the way btop's
`color_theme` resolves to a file in its themes directory. Theme files join the `t` cycle after
the built-in themes. A built-in name always wins over a file of the same name.

The file uses btop's format and btop's key names, so a btop theme loads as-is:

```text
theme[main_bg]="#181921"
theme[main_fg]="#EBDBB2"
theme[cpu_start]="#7FC678"
theme[cpu_mid]="#D79921"
theme[cpu_end]="#CC5351"
```

Colors are `#RRGGBB`, `#GG` for a grey, or `R G B` in decimal. The empty values mean what they mean
to btop:

- An empty `main_bg` leaves the terminal's own background.
- An empty `_mid` ramps straight from start to end.
- An empty `_end` paints the start color solid.

A missing key keeps the built-in `btop` theme's color. `meter_bg` and `graph_text` fall back to
`inactive_fg`, and missing `process_*` keys copy `cpu_*`.

abtop reads `main_bg`, `main_fg`, `title`, `hi_fg`, `selected_bg`, `selected_fg`,
`inactive_fg`, `graph_text`, `meter_bg`, `proc_misc`, `div_line`, the four `*_box` borders, and
the `cpu_*`, `process_*`, `used_*`, `free_*` and `cached_*` gradients. It adds keys btop does not
have:

| Key | Colors |
| --- | --- |
| `session_id` | Session ids |
| `status_fg` | Errors and rate limits |
| `warning_fg` | Warnings |
| `agent_claude`, `agent_codex`, `agent_opencode` | The agent badge in the session list |

`cpu_*` colors the context and quota meters, and `process_*` colors each session's context
percentage, so both read as severity scales.

A theme generator writes the file and sends `SIGUSR2`, which reloads the config file:

```bash
pkill -USR2 -x abtop
```

## Configuration

abtop reads `~/.config/abtop/config.toml` and never writes it. A theme picked with `t` and a panel
toggled with `1`–`7` last until abtop exits. To keep one, set it in the file.

`SIGUSR2` re-applies the file's `theme`, `show_*` and `summary_model`, dropping any change made
with a key since. `hidden_agents` and `claude_config_dirs` take a restart.

The file supports:

```toml
theme = "btop"
# Panels, toggled at runtime with 1-7. All default to true.
show_context = true
show_quota = true
show_tokens = true
show_projects = true
show_ports = true
show_sessions = true
show_mcp = true
# Hide specific agent CLIs from the TUI (case-insensitive), including
# their quota column. Useful if you only use one agent.
hidden_agents = ["codex"]
# Model for the `claude --print` call that titles each session.
# Omit to use the model your Claude Code settings select.
summary_model = "claude-sonnet-5-5"
# Additional Claude Code profile roots to scan.
# abtop also auto-discovers ~/.claude and ~/.claude-* roots that contain
# both sessions/ and projects/.
claude_config_dirs = ["~/.claude-personal", "~/.claude-work-team"]
# UI language. Omit or leave empty to auto-detect from LANG.
language = "zh"
```

### Supported Languages

| Code | Language            |
| ---- | ------------------- |
| `en` | English (default)   |
| `zh` | Simplified Chinese  |

When `language` is unset, abtop auto-detects from `LANG` — any value starting with `zh` switches to Simplified Chinese, otherwise English.

## Key Bindings

| Key                | Action                               |
| ------------------ | ------------------------------------ |
| `↑`/`↓` or `k`/`j` | Select session                       |
| `Enter`            | Jump to session terminal             |
| `x`                | Kill selected session                |
| `X`                | Kill all orphan ports                |
| `t`                | Cycle theme                          |
| `1`–`5`            | Toggle panel visibility              |
| `Esc`              | Open/close config page               |
| `q`                | Quit                                 |
| `r`                | Force refresh                        |

## Library / JSON snapshot

abtop is also a library crate, so local tools can reuse its data-collection
layer in-process — no re-scanning, no subprocesses — and serialize the same
state the TUI renders.

```bash
abtop --json    # one-shot JSON snapshot for scripts
```

For long-running consumers, build an `App`, refresh it with
`App::tick_no_summaries()` (which never spawns `claude --print`, so it doesn't
touch your Claude quota), and call `App::to_snapshot(interval_ms)` to get a
JSON-serializable [`Snapshot`]:

```rust,no_run
use abtop::app::App;
use abtop::{config, theme::Theme};

let cfg = config::load_config();
let mut app = App::new_with_config_and_claude_dirs(
    Theme::default(), &cfg.hidden_agents, cfg.panels, &cfg.claude_config_dirs,
);
app.tick_no_summaries();
let json = serde_json::to_string(&app.to_snapshot(2_000)).unwrap();
```

`App` is not `Send` (it owns the collectors), so keep it on one thread and pass
the serialized JSON elsewhere. [abtop-web-ui](https://github.com/XKHoshizora/abtop-web-ui)
is a reference consumer: a local-first web dashboard built on exactly this API.

## Privacy

abtop reads local files and local process/open-file metadata only. No API keys, no auth. In the TUI and `--once` output, tool names and file paths are shown, but file contents and prompt text are never displayed. Session summaries are generated via `claude --print`, which makes its own API call — this is the only indirect network usage.

The JSON snapshot includes richer local dashboard data, including `summary`, `chat_messages`, working directories, config roots, tool-call previews, child process commands, token counts, and port metadata. Chat text is bounded and redacted by the collectors, but it is still derived from local transcripts and may contain sensitive project context. Treat JSON snapshots as local/private data and avoid writing them to shared logs or exposing them on a network without your own access controls.

## Acknowledgements

Huge thanks to [@tbouquet](https://github.com/tbouquet) for driving much of abtop's recent shape — themes, config overlay and panel toggles, session filtering, subagent tree view, the context window gauge with compaction detection, plus a steady stream of fixes and security hardening along the way.

## License

MIT
