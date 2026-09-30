use thiserror::Error;

/// Every fallible operation in this crate crosses this type — one distinct variant
/// per failure mode a caller needs to branch on (missing binary vs. network vs. a
/// feed that doesn't parse vs. a source we refuse on purpose), rather than a single
/// opaque string.
#[derive(Debug, Error)]
pub enum PodcastHelperError {
    /// The `ffmpeg` binary could not be spawned at all (not on `PATH`), as opposed
    /// to `FfmpegFailed`, where it ran but exited non-zero.
    #[error("ffmpeg binary not found (tried `{0}`): install ffmpeg, e.g. `brew install ffmpeg` or `apt install ffmpeg`")]
    FfmpegNotFound(String),

    /// `ffmpeg` ran but exited non-zero (bad source, unsupported codec, network
    /// error mid-stream, ...); `stderr` is ffmpeg's own diagnostic.
    ///
    /// Field named `input_source` rather than `source`: thiserror treats a field
    /// literally named `source` as `#[source]` (the error's cause), which must
    /// implement `std::error::Error` — a plain `String` doesn't.
    #[error("ffmpeg failed decoding `{input_source}`: {stderr}")]
    FfmpegFailed {
        /// What ffmpeg was pointed at, echoed back so the message identifies
        /// the failing source without the caller having to correlate.
        input_source: String,
        /// ffmpeg's own stderr, verbatim.
        stderr: String,
    },

    /// An HTTP request (feed fetch or, in the future, enclosure probing) failed
    /// before any bytes could be interpreted.
    #[error("network request to `{url}` failed: {message}")]
    Network {
        /// The URL that was requested.
        url: String,
        /// What the transport reported.
        message: String,
    },

    /// The bytes at `url` were fetched but are not a feed `feed-rs` can parse
    /// (RSS 0.9x/1.0/2.0, Atom, JSON Feed).
    #[error("RSS/Atom feed at `{url}` is invalid or unparseable: {message}")]
    InvalidFeed {
        /// The URL the bytes came from.
        url: String,
        /// What the feed parser objected to.
        message: String,
    },

    /// The feed parsed cleanly but none of its entries carried an audio enclosure.
    #[error("feed `{0}` has no episodes with an audio enclosure")]
    EmptyFeed(String),

    /// Spotify / Apple Podcasts catalog URLs: DRM-gated, impossible to process
    /// locally. Mirrors the Python library's behaviour, including the suggested
    /// workaround (find the show's public RSS feed).
    #[error("`{platform}` cannot be processed locally: {hint}")]
    DrmProtected {
        /// The catalog that was recognized, e.g. `Spotify`.
        platform: &'static str,
        /// The suggested way around it, usually "find the show's public RSS
        /// feed". Carried in the error so the caller can show it as-is.
        hint: String,
    },

    /// The string is not a local file path, a recognizable direct audio URL, a
    /// recognizable feed URL, or a known DRM/yt-dlp host.
    #[error("source not recognized as a local file, direct audio URL, or feed: {0}")]
    UnrecognizedSource(String),

    /// Detected as a yt-dlp-compatible source (YouTube, Vimeo, SoundCloud,
    /// Twitch, ...) and delegated to `youtube-helper-rs` for the download; this
    /// variant wraps whatever failure it reported (missing `yt-dlp` binary,
    /// invalid/unsupported URL, network failure, ...).
    #[error("failed to fetch `{url}` via youtube-helper-rs: {source}")]
    YtDlp {
        /// The URL that was delegated.
        url: String,
        /// Whatever `youtube-helper-rs` reported.
        #[source]
        source: youtube_helper_rs::YoutubeHelperError,
    },

    /// An underlying I/O error, passed through unchanged.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
