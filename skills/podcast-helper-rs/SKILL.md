---
name: podcast-helper-rs
description: >-
  From a URL to PCM, in Rust. Parse an RSS or Atom podcast feed into typed episodes, take just the latest one, classify what a string actually names (local file, direct audio URL, feed, a yt-dlp host, or a DRM catalogue page), resolve it to something ffmpeg can read, and extract its audio as f32 PCM samples at a chosen sample rate. Use when the user is writing Rust and says "parse this podcast feed", "list the episodes of this RSS", "get the newest episode", "is this a feed or a direct audio URL", "resolve the enclosure", "decode this episode to PCM", "extract the audio from this episode", "16 kHz mono for a speech model", or names cargo add podcast-helper-rs or podcast_helper_rs. Do NOT use to transcribe (vocal-helper), to download from YouTube (youtube-helper-rs), to convert or trim an audio file (audio-helper-rs), to record a microphone (capture-helper-rs), or for anything behind DRM -- Spotify and Apple Podcasts are refused by design.
license: BSD-3-Clause
metadata:
  author: Warith Harchaoui
  contact: warithmetics@deraison.ai
  repository: https://github.com/warith-harchaoui/podcast-helper-rs
  crate: https://crates.io/crates/podcast-helper-rs
  language: rust
---

# podcast-helper-rs

URL in, PCM out. Feed parsing, source classification, stream resolution and
decoding, with typed values and a `thiserror` enum throughout.

## The five operations

| Intent | CLI | Library |
|--------|-----|---------|
| Parse a feed | `feed` | `fetch_feed`, `parse_feed` |
| Newest episode only | `latest` | `latest_episode` |
| What kind of source | `classify` | `classify_source`, `SourceKind` |
| Resolve for ffmpeg | `resolve` | `resolve_stream_url` |
| Decode to PCM | `extract` | `extract_audio_stream_with_options` |

## Writing Rust against it

```rust
use podcast_helper_rs::{latest_episode, extract_audio_stream_with_options, ExtractOptions};

let episode = latest_episode("https://feeds.example.com/show")?;
let pcm = extract_audio_stream_with_options(
    &episode.enclosure_url,
    &ExtractOptions { sample_rate: 16_000, to_mono: true },
)?;
```

`ExtractOptions` defaults to 16 kHz mono, because the usual next step is a
speech model and that is what they want.

## Classify before you assume

`classify_source` is pure string work — no network, no filesystem — and it is
worth calling first, because the five answers need different handling:

- **local file**: existence is deliberately *not* checked. `ffmpeg` reports a
  missing file far more clearly than a `stat` here could.
- **direct audio URL**: any HTTP(S) URL not recognised as something else. Real
  podcast enclosures routinely carry no file extension, so the permissive
  default is correct.
- **feed**: an RSS or Atom document.
- **yt-dlp**: recognised, not yet handled locally — you get
  `YtDlpNotImplemented`, not a silent failure. Route those to
  `youtube-helper-rs`.
- **DRM**: Spotify and Apple Podcasts catalogue pages are refused outright.
  They are not downloadable, and attempting anyway would fail later and less
  clearly. No flag changes this — do not look for one, and do not suggest a
  workaround.

## On deduplicating episodes

`Episode.guid` is the only field suitable for deduplicating across refetches.
Titles repeat, publication dates get rewritten, enclosure URLs move between
CDNs. Use the guid.

## Choosing this crate over the Python twin

Pick this one for Rust calling code, typed PCM, or a single binary. Pick the
Python `podcast-helper` for the HTTP API, MCP, or anything not ported here.

## References

- `references/triggers.md` — the full catalogue, mirroring the root `TRIGGERS.md`.
- `references/cli-reference.md` — every subcommand, flag and exit code.
- `references/surfaces.md` — which surfaces exist here, and which do not.
