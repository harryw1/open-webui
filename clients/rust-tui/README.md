# Rust TUI Client for Open WebUI

A terminal-based user interface (TUI) for interacting with Open WebUI, featuring integration with a local MCP server for tool execution.

## Features

*   **Chat Interface**: Interact with LLMs hosted by Open WebUI directly from your terminal.
*   **MCP Integration**: (In Progress) Connects to a local MCP server to execute tools.
*   **Streaming Responses**: Real-time token streaming from the backend.

## Configuration

The client requires the following environment variables to be set (either in your shell or a `.env` file in the root):

*   `OPEN_WEBUI_BASE_URL`: The base URL of your Open WebUI instance (default: `http://localhost:3000`).
*   `OPEN_WEBUI_API_KEY`: Your Open WebUI API key.

## Build and Run

You can build the client using Cargo:

```bash
cd clients/rust-tui
cargo run
```

Or from the root using Make:

```bash
make build-tui
# Then run the binary
./clients/rust-tui/target/debug/rust-tui
```

## Architecture

*   **UI**: Built with `ratatui` and `crossterm`.
*   **Async Runtime**: Uses `tokio`.
*   **HTTP Client**: Uses `reqwest` to communicate with Open WebUI.
*   **MCP Client**: Uses `rmcp` to communicate with the MCP server via stdio.
