# Test Checklist for Open WebUI TUI & MCP Server

This checklist is designed to verify the functionality of the new Rust-based TUI client and the local MCP server integration.

## 1. Prerequisites & Setup

- [ ] **Environment Variables**:
    - Ensure `OPEN_WEBUI_API_KEY` is set in your environment or a `.env` file.
    - Value should be a valid API key for your Open WebUI instance.
- [ ] **Build Backend**:
    - Run `make build-backend` (or build `backend/mcp-server-rust` manually).
    - Verify `backend/mcp-server-rust/target/debug/mcp-server-rust` (or release) exists.
- [ ] **Build TUI**:
    - Run `make build-tui` (or build `clients/rust-tui` manually).
    - Verify the binary is created.

## 2. TUI Basic Functionality

- [ ] **Launch**:
    - Run the TUI client (e.g., `cargo run` in `clients/rust-tui` or `./clients/rust-tui/target/debug/rust-tui`).
    - Verify the TUI opens without crashing.
    - Verify the "Chat" and "Input" areas are visible.
    - Verify the initial system message: "Welcome to the Open WebUI TUI! Type your message below."
- [ ] **Input Handling**:
    - Type text into the input field.
    - Verify text appears correctly.
    - Use `Backspace` to delete characters.
    - Verify `Enter` sends the message.
- [ ] **Scrolling**:
    - Generate enough chat history to require scrolling.
    - Use `PageUp` / `PageDown` to scroll by pages.
    - Use `Ctrl+Up` / `Ctrl+Down` to scroll line by line.
    - Verify the chat view updates accordingly.
- [ ] **Exit**:
    - Press `Esc`.
    - Verify the application terminates gracefully and restores the terminal.

## 3. Chat Interaction

- [ ] **Sending Messages**:
    - Send a simple message like "Hello".
    - Verify the message appears in the chat with "**User**:" prefix.
    - Verify the "Thinking..." status appears in blue at the bottom.
- [ ] **Streaming Response**:
    - Verify the assistant's response appears token by token (streaming).
    - Verify the final response is complete and formatted correctly.
    - Verify the status bar clears after the response is finished.
- [ ] **Markdown Rendering**:
    - Ask the model to generate some markdown (e.g., "Show me a list and some bold text").
    - Verify the output is rendered with appropriate formatting (bolding, lists, etc.) in the TUI.

## 4. MCP Server & Tools

- [ ] **Server Connection**:
    - Verify the TUI automatically starts the MCP server (`backend/mcp-server-rust`).
    - *Note: If the server fails to start, a warning should appear in the logs/stderr.*
- [ ] **Tool Discovery**:
    - The TUI should silently fetch tools on startup.
- [ ] **`read_file` Tool**:
    - Ask: "Read the file `README.md` in the current directory."
    - Verify status shows "Executing read_file...".
    - Verify the tool output indicates success or displays content.
    - Verify the assistant summarizes or displays the file content.
- [ ] **`list_directory` Tool**:
    - Ask: "List the files in the current directory."
    - Verify status shows "Executing list_directory...".
    - Verify the assistant lists the files found.
- [ ] **`shell_command` Tool (Whitelist)**:
    - Ask: "Run `ls` command."
    - Verify it works.
    - Ask: "Run `whoami`."
    - Verify it works.
    - Ask: "Run `rm -rf .`" (or a harmless non-whitelisted command like `uptime`).
    - Verify the tool returns an error or permission denied message (the server implements a whitelist).

## 5. Edge Cases & Error Handling

- [ ] **Missing API Key**:
    - Unset `OPEN_WEBUI_API_KEY`.
    - Run the TUI.
    - Verify it exits with a clear error message.
- [ ] **MCP Server Missing**:
    - Rename or move the `mcp-server-rust` binary.
    - Run the TUI.
    - Verify the TUI starts but warns about the missing server.
    - Verify chat still works (without tools).
- [ ] **Invalid Tool Arguments**:
    - Ask the model to read a non-existent file.
    - Verify the tool returns an error (handled by the MCP server) and the assistant reports the error to the user.
