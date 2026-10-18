# Surfaces — podcast-helper-rs

The Python suite's rule is five surfaces per helper: a library, two CLIs, a
FastAPI HTTP server, an MCP tool set mounted on that same app, and an
installable agent skill. This crate carries three.

## What exists

**Library** — `cargo add podcast-helper-rs`. The primary surface, and the only
one that hands back a `PcmStream` of `f32` samples in memory rather than a
file on disk.

**CLI** — `cargo install podcast-helper-rs` gives the `podcast-helper-rs`
binary, behind the default-on `cli` feature.

**Agent skill** — this directory.

## What does not exist, and why

**No HTTP API.** The Python twin already serves one. A second with fewer routes
would be another thing to keep in sync, for no gain.

**No MCP.** The suite's rule is that MCP is *never* a second implementation:
`fastapi-mcp` mounts `/mcp` onto the FastAPI app the helper already serves, so
every MCP tool **is** one of its routes and the two cannot drift apart. Rust
has no equivalent of that mounting, so an MCP surface here would have to be a
separate implementation of the same tools.

**Only one CLI, not two.** The Python side ships argparse and click twins
because both are idiomatic there. Rust has one idiomatic answer, `clap`.

**No yt-dlp handling yet.** Those sources are *recognised* — you get
`YtDlpNotImplemented`, a typed error rather than a silent failure — and routed
to `youtube-helper-rs`, which this crate already depends on.

## Choosing between the two implementations

| Need | Reach for |
|---|---|
| Rust calling code, one binary | `podcast-helper-rs` |
| PCM in memory as typed `f32` | `podcast-helper-rs` |
| HTTP API, MCP tools | Python `podcast-helper` |
