#!/bin/bash
set -e

# Setup environment
if [ -z "$OPEN_WEBUI_API_KEY" ]; then
    echo "Error: OPEN_WEBUI_API_KEY is not set."
    echo "Please export your API key:"
    echo "  export OPEN_WEBUI_API_KEY=your_key_here"
    echo "You can generate one in the Open WebUI settings."
    exit 1
fi

# Default Base URL
export OPEN_WEBUI_BASE_URL="${OPEN_WEBUI_BASE_URL:-http://localhost:3000}"

echo "Using Open WebUI at: $OPEN_WEBUI_BASE_URL"

# Build components
echo "Building MCP Server..."
if make build-backend; then
    echo "MCP Server built successfully."
else
    echo "Failed to build MCP Server. Attempting manual cargo build..."
    cargo build --manifest-path backend/mcp-server-rust/Cargo.toml --release
fi

echo "Building TUI Client..."
if make build-tui; then
    echo "TUI Client built successfully."
else
    echo "Failed to build TUI Client. Attempting manual cargo build..."
    cargo build --manifest-path clients/rust-tui/Cargo.toml --release
fi

# Locate binaries
# The makefile/cargo build might produce debug or release depending on flags,
# but my manual fallback uses --release. The makefile in this repo uses default (debug).
# Let's try to find the newest one or default to debug if Make was used.

TUI_BIN="./clients/rust-tui/target/debug/rust-tui"
if [ ! -f "$TUI_BIN" ]; then
    TUI_BIN="./clients/rust-tui/target/release/rust-tui"
fi

if [ ! -f "$TUI_BIN" ]; then
    echo "Error: Could not find rust-tui binary."
    exit 1
fi

echo "Starting TUI..."
"$TUI_BIN"
