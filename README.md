# tmui

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

`install.sh`는 다음을 수행합니다:

1. `cargo build --release`로 바이너리 빌드
2. `~/.cargo/bin/`에 바이너리 설치
3. `.tmux.conf`를 `~/`에 복사 (기존 파일은 `.tmux.conf.bak`으로 백업)
4. tmux 실행 중이면 설정 자동 리로드

### Requirements

- Rust 1.70+
- tmux (any recent version)

## tmux Integration

설치 후 tmux에서 `prefix + s`를 누르면 기본 세션 목록(`choose-tree`) 대신 tmui가 팝업으로 실행됩니다.

```tmux
# .tmux.conf
bind s display-popup -E -w 80% -h 80% tmui
```

> 기본 prefix는 `Ctrl + a`입니다.

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
