# Next Stage Development Prompt

**Task:** Advance the Rust-based TUI Client and MCP Server implementation to support full tool execution and enhanced UI rendering.

**Role:** Senior Rust Engineer & LLM Integration Specialist.

**Context:**
We have established the foundation:
1.  **MCP Server (`backend/mcp-server-rust`)**: Implements `read_file`, `list_directory`, `shell_command`.
2.  **TUI Client (`clients/rust-tui`)**: A Ratatui application that connects to Open WebUI and the local MCP server. It currently sends chat messages but does not yet process tool calls.

**Objectives:**

1.  **Implement Tool Execution Logic in TUI:**
    *   **Parsing**: Modify the TUI's message handling to detect when the LLM wants to call a tool. The Open WebUI API likely returns `tool_calls` in the response structure. You need to parse this (likely adapting the `ChatCompletionChunk` struct).
    *   **Orchestration**: When a tool call is detected:
        *   Pause the user input/stream.
        *   Use the `McpClient` to call the corresponding tool on the MCP server.
        *   Capture the tool result.
    *   **Feedback Loop**: Append the tool result to the chat history as a "tool" role message and send it back to the LLM to get the final response.

2.  **Enhance UI with Markdown Rendering:**
    *   Integrate `tui-markdown` (already in `Cargo.toml`) into `ui.rs`.
    *   Render the `assistant` messages using Markdown parsing to support code blocks, bold text, lists, etc.
    *   Ensure scrolling works correctly with the rendered markdown content.

3.  **Refine State Management:**
    *   The current `AppState` is simple. You may need to handle "waiting for tool execution" states or "streaming" states more explicitly to provide visual feedback to the user (e.g., a spinner or status line).

**Technical Details:**
*   **Repo Root**: `/app`
*   **MCP Server**: `backend/mcp-server-rust`
*   **TUI Client**: `clients/rust-tui`
*   **Build**: Use `make build-tui` and `make build-backend`.

**Deliverables:**
*   Updated `clients/rust-tui/src` code implementing the above logic.
*   Verification that the TUI can ask the LLM to "list files in current directory", the LLM calls the tool, the TUI executes it via MCP, and the LLM summarizes the file list.
