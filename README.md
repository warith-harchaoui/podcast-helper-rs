# Podcast Helper (Rust)

[🇫🇷](https://github.com/warith-harchaoui/podcast-helper-rs/blob/master/LISEZMOI.md) · [🇬🇧](https://github.com/warith-harchaoui/podcast-helper-rs/blob/master/README.md)

[![crates.io](https://img.shields.io/crates/v/podcast-helper-rs.svg)](https://crates.io/crates/podcast-helper-rs) [![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](./LICENSE)

Rust rewrite of [`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper). Same promise, **URL in, PCM out**: give it any audio source, get back a PCM stream (pulse-code modulation, the raw uncompressed sampling that speech-recognition models expect) — not a line-by-line port of the Python code, idiomatic Rust throughout.

## v0.1 scope

**Two entry points, and the difference matters for live media.**

- `extract_audio_stream(source: &str) -> Result<PcmStream, PodcastHelperError>` decodes the whole thing and hands back PCM. Requires the media to end.
- `resolve_stream_url(source: &str) -> Result<String, PodcastHelperError>` hands back an address instead, for a caller that reads at its own pace. **The only one of the two that works on a live broadcast**, which never finishes downloading.

Both run the same classification, which distinguishes internally between:

| Source | Detection | Behavior |
|---|---|---|
| Local file | existing path, or `file://` scheme | decoded directly via `ffmpeg` |
| Direct audio URL (`.mp3`, `.m4a`, `.opus`, `.wav`, `.m3u8`, or an unrecognized extension) | known extension, or default fallback | decoded directly via `ffmpeg` |
| RSS/Atom feed | `.xml`/`.rss`/`.atom`/`.json` extension | parsed via [`feed-rs`](https://crates.io/crates/feed-rs), latest episode auto-selected, then its enclosure decoded |
| Spotify (`*.spotify.com`) | host match | **refused**: `PodcastHelperError::DrmProtected`, with a hint to look for the show's public RSS feed instead |
| Apple Podcasts (`podcasts.apple.com`) | host match | **refused**, same behavior |
| YouTube / Vimeo / SoundCloud / Twitch | host match | delegated to [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs): `extract_audio_stream` downloads the audio track to a temporary WAV file and decodes it through the same `ffmpeg` path as any other source; `resolve_stream_url` asks for the signed media address instead, and never downloads |
| Other scheme (`ftp://`, ...) | — | `PodcastHelperError::UnrecognizedSource` |

Output: `PcmStream { sample_rate, channels, samples: Vec<f32> }`, 16 kHz mono by default (`ExtractOptions::default()`), configurable via `extract_audio_stream_with_options`.

```rust
let pcm = podcast_helper_rs::extract_audio_stream("https://feeds.npr.org/510289/podcast.xml")?;
println!("{} samples at {} Hz", pcm.samples.len(), pcm.sample_rate);

// A live broadcast: resolve, then let a streaming client follow it.
let url = podcast_helper_rs::resolve_stream_url("https://www.youtube.com/watch?v=<live id>")?;
println!("stream it with: ffmpeg -i '{url}' ...");
```

`resolve_stream_url` yields a local path unchanged, a direct URL unchanged, a feed's latest enclosure after one HTTP fetch of the feed document and no media transfer, and a yt-dlp source's signed media URL. A DRM catalog is refused exactly as the decoding path refuses it — the two entry points agree on what is impossible, so a Spotify URL never reaches `ffmpeg` to fail there as what looks like a network error.

**A resolved URL can expire.** YouTube signs its media addresses and invalidates them within hours: resolve immediately before use, and never persist the result.

### Deliberate differences from the Python original

- No frame-by-frame async iterator: v0.1 decodes into one block (a full `Vec<f32>`). Splitting into frames is left to the caller for now.
- `Episode.duration_seconds` carries `itunes:duration` since 0.1.3, read from the raw XML rather than from `feed-rs` — which does surface the tag but parses the `MM:SS` form as a bare seconds count (`45:30` becomes 45). It is `None` when the feed omits the tag, spells it unreadably, or writes the placeholder `0`.
- No generic `yt-dlp`-style extractor: an HTTP(S) URL that is neither a recognized stream extension nor a known DRM/yt-dlp host is attempted directly via `ffmpeg` (a deliberately permissive choice, since many podcast enclosure URLs have no extension at all).

## `youtube-helper-rs` — wired in

YouTube/Vimeo/SoundCloud/Twitch sources are detected (`SourceKind::YtDlp`, `src/source.rs`) and delegated to [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs) (`src/ytdlp.rs`): it downloads the audio track to a temporary directory, and the resulting WAV file is decoded through the same `ffmpeg` path used for local files — one decode path for every source kind this crate supports, not a special case per platform.

Known limitation, inherited from `youtube-helper-rs` itself: `yt-dlp` can hit an `HTTP 403 Forbidden` from YouTube's media CDN in sandboxed/CI-like environments without browser cookies or a PO-token provider configured (YouTube-side anti-bot enforcement, not a bug in either crate). The error surfaces cleanly as `PodcastHelperError::YtDlp { .. }` in that case — see `tests/network_integration.rs`.

## Installation

Requires `ffmpeg` on `PATH` (macOS: `brew install ffmpeg`), and Rust 1.88 or newer.

```toml
[dependencies]
podcast-helper-rs = "0.1"
```

## Project status

- `cargo build`: clean, no warnings.
- `cargo test`: **46 unit tests + 1 doctest passing**, 0 failures, plus 4 `#[ignore]`d network integration tests (a real RSS feed, an end-to-end `ffmpeg` decode, a Spotify refusal, a real YouTube download+decode via `youtube-helper-rs`) — run manually via `cargo test -- --ignored` before committing changes to any of these paths. The YouTube one can fail in sandboxed environments for reasons unrelated to this crate — see the section above.
- `cargo clippy --all-targets`: clean, no warnings.
- **Measured code coverage** (`cargo llvm-cov`, default test suite, network excluded; re-measured 2026-10-02 for 0.1.3): **93.15% line coverage** (715 lines, 49 uncovered), 90.08% region coverage, 93.75% function coverage. Per-file breakdown:

  | File | Line coverage |
  |---|---|
  | `pcm.rs` | 100.00% |
  | `duration.rs` | 100.00% |
  | `source.rs` | 97.04% |
  | `episode.rs` | 95.35% |
  | `ffmpeg.rs` | 93.33% |
  | `ytdlp.rs` | 93.33% |
  | `lib.rs` | 87.50% |
  | `feed.rs` | 72.94% (the network path `fetch_feed`/`latest_episode` is only exercised by the `#[ignore]`d tests, so it's absent from this measurement) |

  To reproduce:

  ```bash
  # once: LLVM tools (this project uses Homebrew LLVM, not rustup)
  export LLVM_COV=/opt/homebrew/opt/llvm/bin/llvm-cov
  export LLVM_PROFDATA=/opt/homebrew/opt/llvm/bin/llvm-profdata

  cargo llvm-cov --summary-only        # text summary
  cargo llvm-cov --html                # HTML report at target/llvm-cov/html/index.html
  ```

## Checks before pushing

```bash
scripts/install-hooks.sh   # once per clone: installs the pre-push gate below
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all
scripts/check-fresh-resolve.sh   # builds with no Cargo.lock, like a downstream consumer
```

The last one exists because the other three, and CI, all build against the committed
`Cargo.lock` — which no consumer of the published crate ever sees. A dependency range that
has gone bad stays green here while breaking every fresh `cargo add` / `cargo install`, and
`cargo publish --dry-run` doesn't catch it either (it verifies with the same lock). CI runs
this check once, on Linux.

## Related

Part of the same author's local-first tooling as [`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper) (Python) and the [AI Helpers](https://github.com/warith-harchaoui/ai-helpers) suite. Independent rewrite, not a binding.

## License

BSD-3-Clause, see [LICENSE](LICENSE).

## Author

[Warith HARCHAOUI](https://linkedin.com/in/warith-harchaoui)
