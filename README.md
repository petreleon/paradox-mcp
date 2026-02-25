# Paradox MCP Server (Rust)

A Model Context Protocol (MCP) server implementations specifically built to interact with Paradox DB files using the C library `pxlib`.

The server runs entirely through stdio using JSON-RPC.

## Usage

The easiest way to consume this MCP server is via Docker, as it encapsulates the Linux `pxlib` dependencies smoothly without requiring system-level C dependency installations on your host machine.

### Building the Docker Image

```bash
docker build -t paradox-mcp .
```

### Running the MCP Server

```bash
docker run -i --rm -v /path/to/host/paradox/db:/data paradox-mcp --location /data
```

## Available Tools

- `get_server_status`: Returns the current mounted location and editing flags.
- `list_tables`: Scans the provided directory for `.db` Paradox files.

## Future Enhancements
- Extract table configurations using `pxlib` directly.
- Enable `query_table` to output rows in JSON format.
