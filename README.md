# tmui

[한국어](README_ko.md)

A fast, vim-keyed TUI for managing tmux sessions. Built with Rust, [ratatui](https://ratatui.rs), and [nucleo](https://github.com/helix-editor/nucleo) fuzzy matching.

## Features

- **Session list** with live preview of pane content (ANSI color support)
- **Vim-style navigation** (`j`/`k`, `G`/`gg`)
- **Fuzzy search** (`/`) powered by nucleo-matcher with match highlighting
- **Session tagging** and tag-based filtering
- **Window expansion** (Tab) to inspect windows inside each session
- **Help overlay** (`?`) with keybinding cheat sheet
- **CJK/Unicode support** in session names and preview

## Installation

```bash
git clone https://github.com/sm-kim1/tmui.git
cd tmui
./install.sh
```

`install.sh` performs the following:

1. Builds the binary with `cargo build --release`
2. Installs the binary to `~/.cargo/bin/`
3. Copies `.tmux.conf` to `~/` (backs up existing file as `.tmux.conf.bak`)
4. Reloads tmux config automatically if tmux is running

### Requirements

- Rust 1.70+
- tmux (any recent version)

## tmux Integration

After installation, pressing `prefix + s` in tmux launches tmui as a popup instead of the default session list (`choose-tree`).

```tmux
# .tmux.conf
bind s display-popup -E -w 80% -h 80% tmui
```

> The default prefix is `Ctrl + a`.

## Usage

```bash
# Launch inside or outside tmux
tmui
```

### Keybindings

| Key     | Action                   |
|---------|--------------------------|
| `j`/`k` | Move down/up            |
| `G`     | Jump to last             |
| `gg`    | Jump to first            |
| `Enter` | Attach/switch to session |
| `n`     | Create new session       |
| `r`     | Rename session           |
| `dd`    | Kill session (confirm)   |
| `D`     | Detach clients           |
| `/`     | Fuzzy search             |
| `t`     | Add tag to session       |
| `T`     | Filter by tag / clear    |
| `Tab`   | Expand/collapse windows  |
| `?`     | Toggle help overlay      |
| `q`     | Quit                     |

### Inside vs Outside tmux

- **Inside tmux**: uses `switch-client` to switch sessions seamlessly
- **Outside tmux**: uses `attach-session` via exec to replace the process

## Configuration

Config is stored at `~/.config/tmui/config.toml` (XDG). Tags and groups are persisted automatically.

```toml
[tags]
work = ["important", "dev"]
personal = ["home"]

[groups]
```

## Development

```bash
cargo test          # Run 73+ tests
cargo clippy        # Lint (0 warnings)
cargo fmt --check   # Format check
```

## License

MIT
