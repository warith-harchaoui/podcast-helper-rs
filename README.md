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
| YouTube / Vimeo / SoundCloud / Twitch | host match | detected but **not implemented yet**: `PodcastHelperError::YtDlpNotImplemented` (see below) |
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

## `youtube-helper-rs` — not wired in yet

YouTube/Vimeo/SoundCloud/Twitch sources are detected (`SourceKind::YtDlp`) but return `PodcastHelperError::YtDlpNotImplemented`: [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs) was not yet published on GitHub when this crate was written. The integration point is marked `// TODO(youtube-helper-rs)` in `src/error.rs` and `src/lib.rs`. Tracked in issue [#1](https://github.com/warith-harchaoui/podcast-helper-rs/issues/1) on this repository.

## Installation

Requires `ffmpeg` on `PATH` (macOS: `brew install ffmpeg`).

```toml
[dependencies]
podcast-helper-rs = { git = "https://github.com/warith-harchaoui/podcast-helper-rs" }
```

## Project status

- `cargo build`: clean, no warnings.
- `cargo test`: **32 unit tests + 1 doctest passing**, 0 failures, plus 3 `#[ignore]`d network integration tests (a real RSS feed, an end-to-end `ffmpeg` decode, a Spotify refusal) — verified manually once via `cargo test -- --ignored` (all 3 pass) before committing, not run by default.
- `cargo clippy --all-targets`: clean, no warnings.
- **Measured code coverage** (`cargo llvm-cov`, default test suite, network excluded): **92.24% line coverage** (451 lines, 35 uncovered), 88.82% region coverage, 92.75% function coverage. Per-file breakdown:

  | File | Line coverage |
  |---|---|
  | `pcm.rs` | 100.00% |
  | `ffmpeg.rs` | 98.06% |
  | `episode.rs` | 95.12% |
  | `source.rs` | 97.01% |
  | `lib.rs` | 90.70% |
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
