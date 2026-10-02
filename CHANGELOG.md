# Changelog

All notable changes to `podcast-helper-rs` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project adheres to
[Semantic Versioning](https://semver.org/).

## [0.1.3] — 2026-10-02

`Episode::duration_seconds` is finally populated — and correct, which took more than reading
it off `feed-rs`.

### Added
- **Episode durations.** `Episode::duration_seconds` now carries the feed's `itunes:duration`,
  in seconds, for every episode that declares one. It had shipped as a permanent `None` since
  0.1.0, with a doc comment claiming `feed-rs` did not surface the tag.

  That claim was wrong, and the truth is worse: `feed-rs` *does* surface it (on
  `MediaObject::duration`) but reads the text with the RTSP "normal play time" grammar, which
  knows only `HH:MM:SS` and a bare seconds count. The iTunes podcast spec also allows `MM:SS`,
  and on that form the fallback matches the leading number — `45:30` comes back as 45 seconds
  instead of 2730. Only the parsed `Duration` is returned, so the original text is
  unrecoverable and the value cannot be repaired after the fact. Hence `src/duration.rs`: one
  streaming pass over the raw XML with the `quick-xml` reader `feed-rs` already depends on,
  keyed on the enclosure URL. `feed-rs`'s behaviour is pinned by its own test, so a future
  release that fixes `MM:SS` shows up as a failure saying the workaround can be deleted.

  A duration is `None` when the feed omits it, spells it unreadably, or writes the placeholder
  `0` — guessing would be worse than silence. Like every other field on `Episode`, it is what
  the publisher typed, not what the audio measures.

### Changed
- New direct dependency on `quick-xml` (for the above). It costs no extra compilation: the
  same version was already in the tree underneath `feed-rs`.
- **CI is one Linux job instead of a three-OS matrix**, and it installs `ffmpeg` — which is why
  every job of that matrix was red: no runner image ships it. The two tests that need a real
  `ffmpeg` now skip with a printed reason when it is absent locally, and fail loudly when it is
  absent under CI, where that means the workflow is broken.
- `#![cfg_attr(not(test), forbid(unsafe_code))]` and `#![deny(missing_docs)]` replace
  `#![warn(missing_docs)]`. The `forbid` is lifted only under `cfg(test)`, where one test sets
  a process-wide environment variable.
- `rust-version = "1.88"` is now declared. Verified, not guessed: the suite passes on a real
  1.88.0 toolchain and fails on 1.85.0, for two independent reasons — `slice::as_chunks` in
  `src/ffmpeg.rs`, and the `icu_*` crates underneath `ureq`/`url`.

## [0.1.2] — 2026-10-02

### Added
- **`resolve_stream_url`** — the same source classification as `extract_audio_stream`, but it
  hands back an address instead of PCM. This is the only one of the two that works on a live
  broadcast: decoding requires the media to end, and a live stream never does.

### Changed
- Requires `youtube-helper-rs` 0.1.2 (for `resolve_media_url`).

## [0.1.1] — 2026-09-05

### Changed
- Repo hygiene only, no library change. Added `scripts/check-fresh-resolve.sh` and a versioned
  `.githooks` pre-push gate; depend on `youtube-helper-rs` from crates.io rather than git (a
  git dependency makes a crate unpublishable).
- Every public item documented, so docs.rs carries the whole API.

## [0.1.0] — 2026-09-05

First release. **URL in, PCM out**: local files, direct audio URLs, RSS/Atom feeds (latest
episode auto-selected), and yt-dlp-compatible sources (YouTube, Vimeo, SoundCloud, Twitch, ...)
delegated to `youtube-helper-rs`. Spotify and Apple Podcasts catalog URLs are refused outright
with a pointer to the show's public RSS feed. Rust port of the Python
[`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper).
