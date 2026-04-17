# rssr

A terminal RSS reader built with [Ratatui](https://ratatui.rs). Catppuccin Mocha theme, keyboard-driven navigation, and full RSS/Atom support.

## Features

- RSS and Atom feed support
- Organise feeds into categories
- Starred/favorites list with per-source grouping
- OPML import and export
- Readability fetch — automatically pulls full article content when the feed only provides a summary
- Catppuccin Mocha colour theme throughout

## Installation

```bash
cargo install rssr-rs
```

Requires Rust 1.85+ (edition 2024).

## Usage

Launch with:

```bash
rssr
```

### Keybindings

| Key | Context | Action |
|-----|---------|--------|
| `j` / `↓` | Lists | Next item |
| `k` / `↑` | Lists | Previous item |
| `Enter` | Feed list | Open feed / toggle category |
| `Enter` | Article list | Open article |
| `Esc` | Any | Go back |
| `Tab` / `Shift+Tab` | Any | Switch tab |
| `q` | Most views | Quit |
| `r` | Feed / article list | Refresh current feed |
| `e` | Feed list | Open feed editor |
| `m` | Article list / detail | Toggle read/unread |
| `s` | Article list / detail | Toggle starred |
| `o` | Article detail | Open link in browser |
| `a` | Feed editor | Add feed |
| `n` | Feed editor | New category |
| `r` | Feed editor | Rename selected item |
| `d` | Feed editor | Delete selected item |

## Data storage

All data is stored in the platform data directory:

| Platform | Path |
|----------|------|
| Linux / BSD | `~/.local/share/rssr/` |
| macOS | `~/Library/Application Support/rssr/` |
| Windows | `%APPDATA%\rssr\` |

Files: `feeds.json`, `user_data.json`, `articles.json`, `categories.json`

## License

MIT — see [LICENSE](LICENSE).
