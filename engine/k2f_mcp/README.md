# k2f_mcp — stdio MCP server

Thin stdio adapter over **`k2f_sdk`**. Tool names, descriptions, and input contracts live in one place: [`mcp_tools.json`](./mcp_tools.json). The sibling [`k2f-site`](https://github.com/suzheng/k2f-site) portal renders that JSON at `/roadmap/mcp` for browsing; the file in this repo is the source of truth. **Remote HTTP MCP is not shipped.**

## Run

From the repo root:

```bash
cargo run -p k2f_mcp --release
```

The process speaks MCP over stdin/stdout. **`sign` is not a tool.**

## Environment

| Variable | Default | Purpose |
|----------|---------|---------|
| `K2F_PUBLISH_ORIGIN` | `http://127.0.0.1:3000` | Base URL for the `publish` tool (`POST /api/publish`) |

Publish requires the sibling site DB (`cd ../k2f-site && npm run db:migrate` with `DATABASE_URL`).

## Claude Desktop (example)

```json
{
  "mcpServers": {
    "k2f": {
      "command": "cargo",
      "args": ["run", "-p", "k2f_mcp", "--release"],
      "cwd": "/absolute/path/to/k2f",
      "env": {
        "K2F_PUBLISH_ORIGIN": "https://your-k2f-site.example"
      }
    }
  }
}
```

## Agent-facing contract

Do not duplicate the tool table here. See [`mcp_tools.json`](./mcp_tools.json) for:

- Tool descriptions (must match `tools/list`)
- Required inputs (`save` / `export_pdf` require `path`; never return package/PDF bytes)
- Session rules (`create` / `open` / `markdown_to_k2f` → Editor)
- Forbidden tools (`sign`)

Resources: `k2f://session/{id}` and `k2f://session/{id}/node/{node_id}` — metadata and node JSON only.

## Tests

```bash
cargo test -p k2f_mcp
```
