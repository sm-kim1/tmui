#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "==> Building tmui..."
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

echo "==> Installing tmui binary..."
INSTALL_DIR="$HOME/.cargo/bin"
mkdir -p "$INSTALL_DIR"
cp "$SCRIPT_DIR/target/release/tmui" "$INSTALL_DIR/tmui"

echo "==> Installing .tmux.conf..."
if [ -f "$HOME/.tmux.conf" ]; then
    cp "$HOME/.tmux.conf" "$HOME/.tmux.conf.bak"
    echo "    (backed up existing .tmux.conf to .tmux.conf.bak)"
fi
cp "$SCRIPT_DIR/.tmux.conf" "$HOME/.tmux.conf"

if [ -n "$TMUX" ]; then
    tmux source-file "$HOME/.tmux.conf" 2>/dev/null && echo "==> Reloaded tmux config." || true
fi

echo "==> Done! tmui installed to $INSTALL_DIR/tmui"
