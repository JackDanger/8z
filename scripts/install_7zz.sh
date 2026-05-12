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
    # Ensure 7zz is available in PATH by creating a symlink if needed
    if [ ! -f /usr/bin/7zz ] && [ -f /usr/lib/p7zip/7zz ]; then
        sudo ln -sf /usr/lib/p7zip/7zz /usr/bin/7zz
    fi
else
    echo "ERROR: unsupported OS '$os'" >&2
    exit 1
fi

if command -v 7zz >/dev/null 2>&1; then
    echo "7zz installed: $(command -v 7zz)"
    echo "Version check:"
    7zz --help 2>&1 | head -1
else
    echo "ERROR: 7zz install failed or not in PATH" >&2
    exit 1
fi
