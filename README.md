# Podcast Helper (Rust)

[🇫🇷](https://github.com/warith-harchaoui/podcast-helper-rs/blob/main/LISEZMOI.md) · [🇬🇧](https://github.com/warith-harchaoui/podcast-helper-rs/blob/main/README.md)

[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](./LICENSE)

Rust rewrite of [`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper). Same promise, **URL in, PCM out**: give it any audio source, get back a PCM stream (pulse-code modulation, the raw uncompressed sampling that speech-recognition models expect) — not a line-by-line port of the Python code, idiomatic Rust throughout.

## v0.1 scope

`extract_audio_stream(source: &str) -> Result<PcmStream, PodcastHelperError>` distinguishes internally between:

| Source | Detection | Behavior |
|---|---|---|
| Local file | existing path, or `file://` scheme | decoded directly via `ffmpeg` |
| Direct audio URL (`.mp3`, `.m4a`, `.opus`, `.wav`, `.m3u8`, or an unrecognized extension) | known extension, or default fallback | decoded directly via `ffmpeg` |
| RSS/Atom feed | `.xml`/`.rss`/`.atom`/`.json` extension | parsed via [`feed-rs`](https://crates.io/crates/feed-rs), latest episode auto-selected, then its enclosure decoded |
| Spotify (`*.spotify.com`) | host match | **refused**: `PodcastHelperError::DrmProtected`, with a hint to look for the show's public RSS feed instead |
| Apple Podcasts (`podcasts.apple.com`) | host match | **refused**, same behavior |
| YouTube / Vimeo / SoundCloud / Twitch | host match | delegated to [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs): downloads the audio track to a temporary WAV file, then decoded through the same `ffmpeg` path as any other source |
| Other scheme (`ftp://`, ...) | — | `PodcastHelperError::UnrecognizedSource` |

Output: `PcmStream { sample_rate, channels, samples: Vec<f32> }`, 16 kHz mono by default (`ExtractOptions::default()`), configurable via `extract_audio_stream_with_options`.

```rust
let pcm = podcast_helper_rs::extract_audio_stream("https://feeds.npr.org/510289/podcast.xml")?;
println!("{} samples at {} Hz", pcm.samples.len(), pcm.sample_rate);
```

### Deliberate differences from the Python original

- No frame-by-frame async iterator: v0.1 decodes into one block (a full `Vec<f32>`). Splitting into frames is left to the caller for now.
- `itunes:duration` isn't extracted (that podcast extension isn't covered by `feed-rs`'s unified model): `Episode.duration_seconds` is always `None` for now.
- No generic `yt-dlp`-style extractor: an HTTP(S) URL that is neither a recognized stream extension nor a known DRM/yt-dlp host is attempted directly via `ffmpeg` (a deliberately permissive choice, since many podcast enclosure URLs have no extension at all).

## `youtube-helper-rs` — wired in

YouTube/Vimeo/SoundCloud/Twitch sources are detected (`SourceKind::YtDlp`, `src/source.rs`) and delegated to [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs) (`src/ytdlp.rs`): it downloads the audio track to a temporary directory, and the resulting WAV file is decoded through the same `ffmpeg` path used for local files — one decode path for every source kind this crate supports, not a special case per platform.

Known limitation, inherited from `youtube-helper-rs` itself: `yt-dlp` can hit an `HTTP 403 Forbidden` from YouTube's media CDN in sandboxed/CI-like environments without browser cookies or a PO-token provider configured (YouTube-side anti-bot enforcement, not a bug in either crate). The error surfaces cleanly as `PodcastHelperError::YtDlp { .. }` in that case — see `tests/network_integration.rs`.

## Installation

Requires `ffmpeg` on `PATH` (macOS: `brew install ffmpeg`).

```toml
[dependencies]
podcast-helper-rs = { git = "https://github.com/warith-harchaoui/podcast-helper-rs" }
```

## Project status

- `cargo build`: clean, no warnings.
- `cargo test`: **32 unit tests + 1 doctest passing**, 0 failures, plus 4 `#[ignore]`d network integration tests (a real RSS feed, an end-to-end `ffmpeg` decode, a Spotify refusal, a real YouTube download+decode via `youtube-helper-rs`) — run manually via `cargo test -- --ignored` before committing changes to any of these paths. The YouTube one can fail in sandboxed environments for reasons unrelated to this crate — see the section above.
- `cargo clippy --all-targets`: clean, no warnings.
- **Measured code coverage** (`cargo llvm-cov`, default test suite, network excluded): **92.37% line coverage** (472 lines, 36 uncovered), 88.46% region coverage, 92.96% function coverage. Per-file breakdown:

  | File | Line coverage |
  |---|---|
  | `pcm.rs` | 100.00% |
  | `ffmpeg.rs` | 98.06% |
  | `episode.rs` | 95.12% |
  | `source.rs` | 97.01% |
  | `ytdlp.rs` | 93.33% |
  | `lib.rs` | 91.84% |
  | `feed.rs` | 69.33% (the network path `fetch_feed`/`latest_episode` is only exercised by the `#[ignore]`d tests, so it's absent from this measurement) |

  To reproduce:

  ```bash
  # once: LLVM tools (this project uses Homebrew LLVM, not rustup)
  export LLVM_COV=/opt/homebrew/opt/llvm/bin/llvm-cov
  export LLVM_PROFDATA=/opt/homebrew/opt/llvm/bin/llvm-profdata

  cargo llvm-cov --summary-only        # text summary
  cargo llvm-cov --html                # HTML report at target/llvm-cov/html/index.html
  ```

## Related

Part of the same author's local-first tooling as [`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper) (Python) and the [AI Helpers](https://github.com/warith-harchaoui/ai-helpers) suite. Independent rewrite, not a binding.

## License

BSD-3-Clause, see [LICENSE](LICENSE).

## Author

[Warith HARCHAOUI](https://linkedin.com/in/warith-harchaoui)
