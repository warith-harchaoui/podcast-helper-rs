//! Source classification: given an arbitrary string, decide whether it names a
//! local file, a direct audio URL, an RSS/Atom feed, a yt-dlp-compatible platform,
//! or a DRM-gated catalog URL we refuse outright. Pure and network-free — every
//! branch is a string/URL inspection, so it's exhaustively unit-testable without a
//! network connection (see the tests module below).

use std::path::PathBuf;
use url::Url;

/// The outcome of classifying a source string, before any I/O happens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    /// A path on the local filesystem (bare path or `file://` URL). Existence is
    /// *not* checked here — `ffmpeg` will report a missing file with a clearer
    /// message than a `stat()` call in this module ever could.
    LocalFile(PathBuf),
    /// An HTTP(S) URL that names an audio container directly (`.mp3`, `.m4a`,
    /// `.opus`, `.wav`, `.m3u8`, ...), or any other HTTP(S) URL we don't otherwise
    /// recognize — handed to `ffmpeg` as-is, since real podcast enclosure URLs
    /// routinely have no file extension at all (e.g. `traffic.megaphone.fm/...`).
    DirectAudioUrl(String),
    /// An HTTP(S) URL whose extension marks it as an RSS/Atom feed document.
    Feed(String),
    /// A URL on a host `yt-dlp` knows how to resolve (YouTube, Vimeo, SoundCloud,
    /// Twitch, ...). Not yet implemented locally — see `PodcastHelperError::YtDlpNotImplemented`.
    YtDlp(String),
    /// A DRM-gated catalog URL (Spotify, Apple Podcasts) refused outright.
    Drm { platform: &'static str, hint: String },
    /// A URL with a scheme we don't handle at all (e.g. `ftp://`).
    Unrecognized(String),
}

const FEED_EXTENSIONS: &[&str] = &["xml", "rss", "atom", "json"];

fn extension_of(path: &str) -> Option<String> {
    path.rsplit('.')
        .next()
        .filter(|ext| !ext.is_empty() && *ext != path)
        .map(|ext| ext.to_ascii_lowercase())
}

fn is_spotify_host(host: &str) -> bool {
    host == "spotify.com" || host.ends_with(".spotify.com")
}

fn is_apple_podcasts_host(host: &str) -> bool {
    host == "podcasts.apple.com" || host.ends_with(".podcasts.apple.com")
}

fn is_ytdlp_host(host: &str) -> bool {
    const HOSTS: &[&str] = &[
        "youtube.com",
        "youtu.be",
        "vimeo.com",
        "soundcloud.com",
        "twitch.tv",
    ];
    HOSTS
        .iter()
        .any(|h| host == *h || host.ends_with(&format!(".{h}")))
}

/// Classify a source string. Network-free: never resolves DNS, never opens a
/// socket, never touches the filesystem beyond string/path parsing.
pub fn classify_source(source: &str) -> SourceKind {
    if let Ok(url) = Url::parse(source) {
        match url.scheme() {
            "file" => {
                return match url.to_file_path() {
                    Ok(path) => SourceKind::LocalFile(path),
                    Err(()) => SourceKind::LocalFile(PathBuf::from(url.path())),
                };
            }
            "http" | "https" => {
                let host = url.host_str().unwrap_or("").to_ascii_lowercase();

                if is_spotify_host(&host) {
                    return SourceKind::Drm {
                        platform: "Spotify",
                        hint: "Spotify's catalog is DRM-gated and cannot be processed locally; look for the show's public RSS feed instead.".to_string(),
                    };
                }
                if is_apple_podcasts_host(&host) {
                    return SourceKind::Drm {
                        platform: "Apple Podcasts",
                        hint: "Apple Podcasts URLs point to the catalog, not the audio; look for the show's public RSS feed instead (e.g. via getrssfeed.com or the Podcast Index).".to_string(),
                    };
                }
                if is_ytdlp_host(&host) {
                    return SourceKind::YtDlp(source.to_string());
                }

                return match extension_of(url.path()) {
                    Some(ext) if FEED_EXTENSIONS.contains(&ext.as_str()) => {
                        SourceKind::Feed(source.to_string())
                    }
                    _ => SourceKind::DirectAudioUrl(source.to_string()),
                };
            }
            _ => return SourceKind::Unrecognized(source.to_string()),
        }
    }

    // Not a parseable URL at all: treat the whole string as a filesystem path,
    // exactly like the Python original ("path exists on disk OR file:// scheme" —
    // a relative or absolute path never parses as an absolute URL).
    SourceKind::LocalFile(PathBuf::from(source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_relative_path_is_local_file() {
        assert_eq!(
            classify_source("episode.mp3"),
            SourceKind::LocalFile(PathBuf::from("episode.mp3"))
        );
    }

    #[test]
    fn bare_absolute_path_is_local_file() {
        assert_eq!(
            classify_source("/tmp/episode.wav"),
            SourceKind::LocalFile(PathBuf::from("/tmp/episode.wav"))
        );
    }

    #[test]
    fn file_scheme_url_is_local_file() {
        match classify_source("file:///tmp/episode.mp3") {
            SourceKind::LocalFile(path) => assert_eq!(path, PathBuf::from("/tmp/episode.mp3")),
            other => panic!("expected LocalFile, got {other:?}"),
        }
    }

    #[test]
    fn mp3_url_is_direct_audio() {
        assert_eq!(
            classify_source("https://cdn.example.com/ep1.mp3"),
            SourceKind::DirectAudioUrl("https://cdn.example.com/ep1.mp3".to_string())
        );
    }

    #[test]
    fn m4a_opus_wav_m3u8_are_direct_audio() {
        for url in [
            "https://cdn.example.com/ep1.m4a",
            "https://cdn.example.com/ep1.opus",
            "https://cdn.example.com/ep1.wav",
            "https://cdn.example.com/live.m3u8",
        ] {
            assert!(matches!(classify_source(url), SourceKind::DirectAudioUrl(_)));
        }
    }

    #[test]
    fn mp3_url_with_query_string_is_still_direct_audio() {
        assert_eq!(
            classify_source("https://cdn.example.com/ep1.mp3?dl=1&t=abc"),
            SourceKind::DirectAudioUrl("https://cdn.example.com/ep1.mp3?dl=1&t=abc".to_string())
        );
    }

    #[test]
    fn extensionless_enclosure_url_falls_back_to_direct_audio() {
        // Real podcast enclosures routinely have no extension at all.
        assert_eq!(
            classify_source("https://traffic.megaphone.fm/ABC123456"),
            SourceKind::DirectAudioUrl("https://traffic.megaphone.fm/ABC123456".to_string())
        );
    }

    #[test]
    fn xml_rss_atom_urls_are_feed() {
        for url in [
            "https://feeds.npr.org/510289/podcast.xml",
            "https://example.com/show.rss",
            "https://example.com/show.atom",
        ] {
            assert!(matches!(classify_source(url), SourceKind::Feed(_)));
        }
    }

    #[test]
    fn youtube_hosts_are_ytdlp() {
        for url in [
            "https://www.youtube.com/watch?v=abc123",
            "https://youtu.be/abc123",
            "https://m.youtube.com/watch?v=abc123",
        ] {
            assert!(matches!(classify_source(url), SourceKind::YtDlp(_)));
        }
    }

    #[test]
    fn vimeo_soundcloud_twitch_are_ytdlp() {
        for url in [
            "https://vimeo.com/12345",
            "https://soundcloud.com/artist/track",
            "https://www.twitch.tv/somechannel",
            "https://clips.twitch.tv/someClip",
        ] {
            assert!(matches!(classify_source(url), SourceKind::YtDlp(_)));
        }
    }

    #[test]
    fn spotify_is_drm_refused() {
        match classify_source("https://open.spotify.com/show/abc123") {
            SourceKind::Drm { platform, .. } => assert_eq!(platform, "Spotify"),
            other => panic!("expected Drm, got {other:?}"),
        }
    }

    #[test]
    fn apple_podcasts_is_drm_refused() {
        match classify_source("https://podcasts.apple.com/us/podcast/id123456") {
            SourceKind::Drm { platform, .. } => assert_eq!(platform, "Apple Podcasts"),
            other => panic!("expected Drm, got {other:?}"),
        }
    }

    #[test]
    fn unknown_scheme_is_unrecognized() {
        assert!(matches!(
            classify_source("ftp://example.com/ep1.mp3"),
            SourceKind::Unrecognized(_)
        ));
    }

    #[test]
    fn lookalike_host_is_not_falsely_flagged_as_spotify() {
        // "notspotify.com" must not match ".spotify.com" or "spotify.com".
        assert!(!matches!(
            classify_source("https://notspotify.com/ep1.mp3"),
            SourceKind::Drm { .. }
        ));
    }
}
