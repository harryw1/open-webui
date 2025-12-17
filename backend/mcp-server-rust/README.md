# MCP Server (Rust)

A Rust-based implementation of a Model Context Protocol (MCP) server, designed to provide local filesystem access and shell execution capabilities to MCP clients (like the Open WebUI TUI).

## Tools Implemented

*   `read_file(path: string)`: Reads content from a local file.
*   `list_directory(path: string)`: Lists entries in a directory.
*   `shell_command(cmd: string)`: Executes a whitelisted set of shell commands (`ls`, `cat`, `grep`, `pwd`, `echo`, `find`, `whoami`).

## Build and Run

You can build the server using Cargo:

```bash
cd backend/mcp-server-rust
cargo build
```

Or from the root using Make:

```bash
make build-backend
```

The binary will be located at `backend/mcp-server-rust/target/debug/mcp-server-rust`.

## Usage

This server is intended to be spawned by an MCP Client using stdio transport.
