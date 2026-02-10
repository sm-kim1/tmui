# tmx

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
# Clone and build
git clone https://github.com/youruser/tmx.git
cd tmx
cargo build --release

# Install to PATH
cargo install --path .
```

### Requirements

- Rust 1.70+
- tmux (any recent version)

## Usage

```bash
# Launch inside or outside tmux
tmx
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

Config is stored at `~/.config/tmx/config.toml` (XDG). Tags and groups are persisted automatically.

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
