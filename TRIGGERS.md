# TRIGGERS: podcast-helper-rs

The exhaustive catalogue of what `podcast-helper-rs` does and what should reach
for it. The Python twin, [`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper),
has its own catalogue; the section on choosing between them is the part that
matters.

`podcast-helper-rs` goes from **a URL to PCM**: parse an RSS or Atom feed into
typed episodes, work out what kind of source a string names, resolve it to
something `ffmpeg` can read, and decode it to samples in memory.

## The five operations → how to invoke

| Intent | CLI | Library |
|--------|-----|---------|
| Parse a feed into episodes | `podcast-helper-rs feed` | `fetch_feed`, `parse_feed` |
| Just the newest episode | `podcast-helper-rs latest` | `latest_episode` |
| What kind of source is this? | `podcast-helper-rs classify` | `classify_source`, `SourceKind` |
| Resolve to an ffmpeg-readable URL | `podcast-helper-rs resolve` | `resolve_stream_url` |
| Decode to PCM | `podcast-helper-rs extract` | `extract_audio_stream`, `ExtractOptions` |

## Natural-language phrasings that should fire

- **Feed**: "parse this podcast feed", "list the episodes of this RSS",
  "read an Atom feed", "what episodes does this show have".
- **Latest**: "get the newest episode", "what did they publish last",
  "the most recent episode of this podcast".
- **Classify**: "is this a feed or a direct audio URL", "what kind of link is
  this", "can this be played directly".
- **Resolve**: "turn this into a URL ffmpeg can read", "get the stream URL",
  "resolve the enclosure".
- **Extract**: "decode this episode to PCM", "get f32 samples from this
  podcast", "16 kHz mono for a speech model", "turn this episode into a wav".
- **The crate itself**: `cargo add podcast-helper-rs`, `cargo install
  podcast-helper-rs`, "the Rust podcast helper", "podcast_helper_rs".

## What a source can be

`classify_source` answers without touching the network or the filesystem:

- **local file** — a bare path or a `file://` URL. Existence is deliberately
  not checked: `ffmpeg` reports a missing file far more clearly than a `stat`
  in this crate could.
- **direct audio URL** — an HTTP(S) URL naming an audio container, *or* any
  other HTTP(S) URL that is not recognised otherwise. Real podcast enclosures
  routinely have no extension at all, so the permissive default is the right
  one.
- **feed** — an HTTP(S) URL whose extension marks it as RSS or Atom.
- **yt-dlp** — a host `yt-dlp` knows. Recognised, not yet handled locally.
- **DRM** — a Spotify or Apple Podcasts catalogue page, refused outright rather
  than attempted. These are not downloadable, and pretending otherwise would
  fail later and less clearly.

## Choosing between the Rust crate and the Python twin

Reach for **`podcast-helper-rs`** when the calling code is Rust, when you want
PCM in memory as typed `f32` samples, or when a single binary beats shipping an
interpreter.

Reach for the **Python `podcast-helper`** for the HTTP API, the MCP tool set,
or any feature this crate has not ported.

Both install side by side. The binary is deliberately named
`podcast-helper-rs`, so neither shadows the other on `PATH`.

## When NOT to use podcast-helper-rs (SKIP)

- Transcribing an episode once decoded → `vocal-helper`.
- Downloading from YouTube and friends → `youtube-helper-rs`.
- Converting, trimming or splitting an audio **file** → `audio-helper-rs`.
- Recording from a microphone → `capture-helper-rs`.
- Anything behind DRM. Spotify and Apple Podcasts catalogue URLs are refused
  by design, and no flag changes that.
- Publishing or hosting a feed. This crate reads; it does not write feeds.

## See also

- [`README.md`](README.md) / [`LISEZMOI.md`](LISEZMOI.md): features and install.
- [`CHANGELOG.md`](CHANGELOG.md): what changed, version by version.
