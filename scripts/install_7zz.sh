#!/bin/bash
# Install 7zz (7-Zip CLI) on the current system
# Idempotent: checks if 7zz is already available

set -e

if command -v 7zz >/dev/null 2>&1; then
    echo "7zz already installed at $(which 7zz)"
    exit 0
fi

os=$(uname)
if [ "$os" = "Darwin" ]; then
    echo "Installing 7zz on macOS..."
    brew install sevenzip || true
elif [ "$os" = "Linux" ]; then
    echo "Installing 7zz on Linux..."
    sudo apt-get update
    sudo apt-get install -y 7zip
else
    echo "ERROR: unsupported OS '$os'" >&2
    exit 1
fi

if command -v 7zz >/dev/null 2>&1; then
    echo "✓ 7zz installed: $(which 7zz)"
else
    echo "ERROR: 7zz install failed or not in PATH" >&2
    exit 1
fi
