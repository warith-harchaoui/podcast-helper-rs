//! # podcast-helper-rs
//!
//! **URL in, PCM out.** Rust port of the [`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper)
//! Python library: given any of a local file path, a direct audio URL, or an
//! RSS/Atom feed URL (latest episode auto-selected), decode it to raw PCM samples
//! via `ffmpeg`. Spotify and Apple Podcasts catalog URLs are refused outright
//! (DRM-gated, impossible to process locally) with a pointer toward the show's
//! public RSS feed instead.
//!
//! v0.1 scope: local files, direct audio URLs, RSS/Atom feeds, and yt-dlp-compatible
//! sources (YouTube, Vimeo, SoundCloud, Twitch, ...) — the last one delegated to
//! [`youtube_helper_rs`]. See [`PodcastHelperError::YtDlp`] and the crate README.
//!
//! ```no_run
//! # fn main() -> Result<(), podcast_helper_rs::PodcastHelperError> {
//! let pcm = podcast_helper_rs::extract_audio_stream("episode.mp3")?;
//! println!("{} samples at {} Hz", pcm.samples.len(), pcm.sample_rate);
//! # Ok(())
//! # }
//! ```

mod episode;
mod error;
mod feed;
mod ffmpeg;
mod pcm;
mod source;
mod ytdlp;

pub use episode::Episode;
pub use error::PodcastHelperError;
pub use feed::{fetch_feed, latest_episode, parse_feed};
pub use pcm::{ExtractOptions, PcmStream};
pub use source::{classify_source, SourceKind};

/// Decode `source` to PCM at the default rate (16 kHz, mono — the shape most
/// speech-recognition/VAD models expect). See [`extract_audio_stream_with_options`]
/// to customize sample rate or channel handling.
pub fn extract_audio_stream(source: &str) -> Result<PcmStream, PodcastHelperError> {
    extract_audio_stream_with_options(source, &ExtractOptions::default())
}

/// Decode `source` to PCM, distinguishing internally between a local file, a
/// direct audio URL, an RSS/Atom feed (latest episode is fetched and decoded), and
/// refused/unsupported sources (DRM catalogs, yt-dlp platforms not yet wired in).
pub fn extract_audio_stream_with_options(
    source: &str,
    opts: &ExtractOptions,
) -> Result<PcmStream, PodcastHelperError> {
    match source::classify_source(source) {
        SourceKind::Drm { platform, hint } => {
            Err(PodcastHelperError::DrmProtected { platform, hint })
        }
        SourceKind::YtDlp(url) => ytdlp::extract_via_ytdlp(&url, opts),
        SourceKind::Unrecognized(s) => Err(PodcastHelperError::UnrecognizedSource(s)),
        SourceKind::Feed(url) => {
            let episode = feed::latest_episode(&url)?;
            ffmpeg::decode_with_ffmpeg(&episode.enclosure_url, opts)
        }
        SourceKind::LocalFile(path) => ffmpeg::decode_with_ffmpeg(&path.to_string_lossy(), opts),
        SourceKind::DirectAudioUrl(url) => ffmpeg::decode_with_ffmpeg(&url, opts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spotify_source_is_refused_without_touching_ffmpeg_or_network() {
        let err = extract_audio_stream("https://open.spotify.com/show/abc123").unwrap_err();
        assert!(matches!(
            err,
            PodcastHelperError::DrmProtected {
                platform: "Spotify",
                ..
            }
        ));
    }

    #[test]
    fn apple_podcasts_source_is_refused() {
        let err = extract_audio_stream("https://podcasts.apple.com/us/podcast/id123").unwrap_err();
        assert!(matches!(
            err,
            PodcastHelperError::DrmProtected {
                platform: "Apple Podcasts",
                ..
            }
        ));
    }

    #[test]
    fn youtube_source_is_delegated_to_youtube_helper_rs() {
        // No network, no real yt-dlp: point youtube-helper-rs at a binary that
        // does not exist, so the delegation path is exercised deterministically
        // and the failure surfaces as `PodcastHelperError::YtDlp`, not a panic
        // or a silent fallback.
        // SAFETY: this crate's tests run in one process; no other test mutates
        // this specific env var concurrently.
        unsafe {
            std::env::set_var("YOUTUBE_HELPER_YTDLP_BIN", "yt-dlp-does-not-exist-anywhere");
        }
        let err = extract_audio_stream("https://www.youtube.com/watch?v=abc123").unwrap_err();
        unsafe {
            std::env::remove_var("YOUTUBE_HELPER_YTDLP_BIN");
        }
        assert!(matches!(err, PodcastHelperError::YtDlp { .. }));
    }

    #[test]
    fn unknown_scheme_is_reported_as_unrecognized() {
        let err = extract_audio_stream("ftp://example.com/ep1.mp3").unwrap_err();
        assert!(matches!(err, PodcastHelperError::UnrecognizedSource(_)));
    }

    #[test]
    fn missing_local_file_surfaces_as_ffmpeg_error_not_a_panic() {
        // No such file: classify_source treats it as LocalFile (existence isn't
        // checked at classification time), so the failure should surface as a
        // clean ffmpeg-side error, never a panic.
        let result = extract_audio_stream("/nonexistent/path/does-not-exist-xyz.mp3");
        assert!(result.is_err());
    }
}
