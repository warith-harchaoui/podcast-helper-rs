# Landscape

[🇫🇷 PAYSAGE.md](https://github.com/warith-harchaoui/podcast-helper-rs/blob/master/PAYSAGE.md) · 🇬🇧 English

Related and competing crates in the "podcast or feed audio to PCM, in Rust" space, benchmarked against `podcast-helper-rs`.
Ratings are ⭐ (1) to ⭐⭐⭐⭐⭐ (5), scored on `podcast-helper-rs`'s intended job:
URL in, PCM out: classify a source (local file, direct audio URL, RSS/Atom feed, yt-dlp host, DRM-gated catalogue), pick the latest episode when it is a feed, and decode it through ffmpeg. A crate optimised for a very different job is not penalised; the score
just reflects fit to *this* niche.

## At a glance

<!-- TABLE:START -->
| Rust Podcast Audio | RSS / Atom parsing | iTunes duration correctness | Source classification | PCM decode | Live stream resolve | Feed authoring | Async API | Typed errors |
| --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **podcast-helper-rs** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ |
| feed-rs | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐ | ⭐ | ⭐ | ⭐ | ⭐⭐⭐ |
| feedparser-rs | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐ | ⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| rss | ⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐⭐ |
| podfeed | ⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐ | ⭐ | ⭐⭐ |
| ffmpeg-next | ⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ | ⭐ | ⭐⭐ |
<!-- TABLE:END -->

## Positioning map

<!-- FIGURE:START -->
2D representation of the table above.

![Positioning map](https://raw.githubusercontent.com/warith-harchaoui/podcast-helper-rs/master/assets/landscape.png)

The map is a 2-D summary of the eight criteria, so read it as a shape, not a scoreboard. `podcast-helper-rs` is at the top-right corner. The axes read **Horizontal: Connected ↔ Robust** and **Vertical: Creative ↔ Precise**.

The horizontal axis measures whether a crate reaches the audio at all — PCM decoding and live-stream resolution are its two strongest loadings — while the vertical axis measures whether it reads the feed correctly and says so in types, led by iTunes duration correctness and typed errors.

The consequence is a map with a crowded left column and only two crates on the right: `podcast-helper-rs` and `ffmpeg-next`. Everything else — `feed-rs`, `feedparser-rs`, `rss`, `podfeed` — stops at the feed document and hands you an enclosure URL. That is not a criticism; it is the category boundary, and this crate crosses it by delegating decoding to ffmpeg.

Those two right-hand crates then split vertically, which is the point worth taking away: `ffmpeg-next` decodes anything and knows nothing about feeds, while this crate parses the feed, re-reads `itunes:duration` through `quick-xml` where `feed-rs` mis-reads the `MM:SS` form, and refuses a DRM-gated page outright rather than failing obscurely. `feedparser-rs` is the strongest pure parser on the map and the one to take ideas from.
<!-- FIGURE:END -->

## Where the claims come from

Versions and publication dates read from the crates.io API on 29 October 2026:

| Crate | Version | Last published |
|---|---|---|
| `feed-rs` | 3.0.0 | 2026-09-27 |
| `feedparser-rs` | 0.7.1 | 2026-10-21 |
| `rss` | 2.1.2 | 2026-09-10 |
| `podfeed` | 0.3.5 | 2026-08-09 |
| `ffmpeg-next` | 9.0.0 | 2026-08-05 |

`podcast-helper-rs`'s own ratings are from this repository at version 0.1.6.

- **This crate scores 4, not 5, on RSS/Atom parsing, and `feed-rs` scores 5.** That is not modesty: the parsing *is* `feed-rs`. The one point of difference runs the other way — `feed-rs` scores 2 on `iTunes duration correctness` because it mis-reads `itunes:duration` in `MM:SS` form, which is why `src/duration.rs` re-reads that one field through `quick-xml`.

- **`feed-rs` 3.0.0 is a major ahead of the `feed-rs = "2"` this crate pins.** Recorded here as a fact rather than argued: the pin is current-at-time-of-release, not current-at-time-of-reading, and closing that gap is its own change with its own test run.

- **The two 1s are feed authoring and an async API**, both absent on purpose. `rss` scores 5 on authoring and is the crate to reach for when the job is to *write* a feed; `feedparser-rs` scores 4 on async.

Deliberately left as opinion rather than sourced: the *pipeline ergonomics*
style columns, and the ordering of the rows below the reference.

## Ideas worth stealing

Researched rather than guessed: what each neighbour does better, and what this crate would have to change to match it. Nothing here is a commitment — it is the reading list the next version starts from.

- **`feedparser-rs` — Podcast 2.0 and JSON Feed** — It covers RSS 0.9x through 2.0, Atom 0.3 and 1.0, JSON Feed 1.0 and 1.1, and both the iTunes and Podcast 2.0 namespaces. `feed-rs` 2, which this crate pins, covers neither Podcast 2.0 nor JSON Feed — and the namespace is where chapters, transcripts and funding tags live.

- **`rss` — writing a feed, not only reading one** — This crate's `Feed authoring` column is a 1 on purpose, but the asymmetry is worth naming: anything that resolves episodes can usually also describe them, and `rss` scores 5 here.

- **`rustypipe` — a channel's RSS feed beats fetching the channel page** — Its `rss` feature exists because the feed is simply faster than the page. The `SourceKind::YtDlp` branch here goes straight to a subprocess; for a channel, the feed would answer sooner and without one.
