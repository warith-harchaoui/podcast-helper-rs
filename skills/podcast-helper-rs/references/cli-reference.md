# CLI reference — `podcast-helper-rs`

Installed with `cargo install podcast-helper-rs`. Named after the crate, not
after the Python twin's `podcast-helper`, so both can sit on `PATH`.

Built behind the default-on `cli` feature. A library consumer who wants neither
`clap`, `hound` nor `serde_json` in their graph depends on the crate with
`default-features = false`.

## Output contract

- Success: the result on **stdout**, exit `0`.
- Failure: one line on **stderr** prefixed `podcast-helper-rs: `, exit `1`.

## `feed` and `latest`

```
podcast-helper-rs feed   <URL> [-p|--pretty]
podcast-helper-rs latest <URL> [-p|--pretty]
```

`feed` prints a JSON array of episodes; `latest` prints one object. One dense
line by default, indented with `--pretty`.

Episode fields: `guid`, `title`, `description`, `link`, `published_at`,
`duration_seconds`, `enclosure_url`, `enclosure_type`, `enclosure_size_bytes`,
`image_url`. All but `enclosure_url` are nullable — feeds vary, and absent is
reported as `null` rather than invented.

**Deduplicate on `guid`, nothing else.** Titles repeat, dates get rewritten,
enclosure URLs move between CDNs.

## `classify`

```
podcast-helper-rs classify <SOURCE>
```

Prints `kind<TAB>detail`, e.g. `feed\thttps://…` or `drm\tSpotify`. Touches
neither the network nor the filesystem, so it answers for a path that does not
exist yet.

Kinds: `local-file`, `direct-audio-url`, `feed`, `yt-dlp`, `drm`.

## `resolve`

```
podcast-helper-rs resolve <SOURCE>
```

Prints one URL `ffmpeg` can read. A `yt-dlp` source yields
`YtDlpNotImplemented` here — send those to `youtube-helper-rs`. A DRM source is
refused outright.

## `extract`

```
podcast-helper-rs extract <SOURCE> <OUTPUT> [--sample-rate HZ] [--stereo]
```

Decodes to PCM and writes a 32-bit float WAV. Defaults to `16000` Hz and mono,
which is what speech models expect; `--stereo` keeps both channels.

Float, because that is what the decoder produced and quantising on the way out
would be a loss nobody asked for. If integer PCM is needed, convert afterwards
with `audio-helper-rs convert`.

## The DRM refusal is not a bug

Spotify and Apple Podcasts catalogue URLs are rejected by design. There is no
flag to force them and none will be added.
