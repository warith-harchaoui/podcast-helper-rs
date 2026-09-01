//! Real-network integration tests, excluded from the default `cargo test` run.
//! Run manually with `cargo test -- --ignored` before committing changes to the
//! feed-fetch or ffmpeg-over-HTTP paths, to confirm they still work against a real
//! podcast feed and a real direct-audio URL.

use podcast_helper_rs::{extract_audio_stream, fetch_feed};

/// NPR's public podcast feed — same one used in the Python original's README
/// quick-start example.
const NPR_FEED_URL: &str = "https://feeds.npr.org/510289/podcast.xml";

#[test]
#[ignore = "hits the real network; run explicitly with `cargo test -- --ignored`"]
fn fetches_and_parses_a_real_rss_feed() {
    let episodes = fetch_feed(NPR_FEED_URL).expect("NPR feed should fetch and parse");
    assert!(!episodes.is_empty(), "expected at least one episode");
    let latest = &episodes[0];
    assert!(!latest.enclosure_url.is_empty());
    println!(
        "latest episode: {:?} -> {}",
        latest.title, latest.enclosure_url
    );
}

#[test]
#[ignore = "hits the real network and shells out to ffmpeg; run explicitly with `cargo test -- --ignored`"]
fn extracts_pcm_from_a_real_feed_url_end_to_end() {
    let pcm = extract_audio_stream(NPR_FEED_URL).expect("should decode the latest episode");
    assert_eq!(pcm.sample_rate, 16_000);
    assert_eq!(pcm.channels, 1);
    assert!(
        pcm.samples.len() > 16_000,
        "expected more than 1 second of audio, got {} samples",
        pcm.samples.len()
    );
}

#[test]
#[ignore = "hits the real network and shells out to ffmpeg; run explicitly with `cargo test -- --ignored`"]
fn spotify_url_is_refused_even_though_it_is_a_real_reachable_host() {
    let err = extract_audio_stream("https://open.spotify.com/show/4rOoJ6Egrf8K2IrywzwOMk")
        .expect_err("Spotify URLs must be refused, not attempted");
    let message = err.to_string();
    assert!(message.contains("Spotify"), "unexpected message: {message}");
}

/// A stable, short, public-domain YouTube video (the same one used in
/// `youtube-helper-rs`'s own test suite) — exercises the full delegation path:
/// classify as `SourceKind::YtDlp` -> `youtube_helper_rs::download_audio` ->
/// decode the resulting WAV via `ffmpeg`.
///
/// Known limitation as of this writing (matches `youtube-helper-rs`'s own
/// `download_audio_real_video_produces_a_file` test): this can fail with an
/// `HTTP 403 Forbidden` from `yt-dlp` in sandboxed/CI-like environments without
/// browser cookies or a PO-token provider configured — YouTube-side anti-bot
/// enforcement on the media CDN, not a bug in this crate's wiring. The error
/// surfaces cleanly as `PodcastHelperError::YtDlp { .. }` either way, which is
/// exactly what this test (and the delegation path) is meant to prove.
#[test]
#[ignore = "hits the real network, shells out to yt-dlp and ffmpeg; run explicitly with `cargo test -- --ignored`"]
fn extracts_pcm_from_a_real_youtube_video_end_to_end() {
    let pcm = extract_audio_stream("https://www.youtube.com/watch?v=jNQXAC9IVRw")
        .expect("should download and decode a real YouTube video via youtube-helper-rs");
    assert_eq!(pcm.sample_rate, 16_000);
    assert_eq!(pcm.channels, 1);
    assert!(
        pcm.samples.len() > 16_000,
        "expected more than 1 second of audio, got {} samples",
        pcm.samples.len()
    );
}
