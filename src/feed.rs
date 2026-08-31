//! RSS/Atom feed handling: parse bytes into episodes (pure, network-free, tested
//! against local fixtures) and fetch-then-parse over HTTP (network, exercised only
//! by the `#[ignore]`d integration test). We lean entirely on `feed-rs` for XML
//! parsing across RSS 0.9x/1.0/2.0, Atom and JSON Feed — no hand-rolled parser.

use crate::episode::Episode;
use crate::error::PodcastHelperError;
use std::io::Read;

/// Parse feed bytes (any of the formats `feed-rs` understands) into a list of
/// episodes, most-recent-first. Entries without an audio enclosure are dropped —
/// they aren't episodes as far as this crate is concerned.
///
/// Pure and network-free: safe to call on a fixture file in a unit test.
pub fn parse_feed(xml_bytes: &[u8], feed_url: &str) -> Result<Vec<Episode>, PodcastHelperError> {
    let feed = feed_rs::parser::parse(xml_bytes).map_err(|e| PodcastHelperError::InvalidFeed {
        url: feed_url.to_string(),
        message: e.to_string(),
    })?;

    let mut episodes: Vec<Episode> = feed
        .entries
        .iter()
        .filter_map(Episode::try_from_entry)
        .collect();

    // Most feeds already list entries newest-first, but that's a convention, not a
    // guarantee. Sort explicitly when we have dates to sort by; otherwise trust
    // feed order (RSS convention: first entry is the latest).
    if episodes.iter().any(|e| e.published_at.is_some()) {
        episodes.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    }

    Ok(episodes)
}

/// Fetch a feed over HTTP(S) and parse it. Network I/O — not exercised by the
/// default test run; see `tests/network_integration.rs` for the `#[ignore]`d test
/// that hits a real feed.
pub fn fetch_feed(url: &str) -> Result<Vec<Episode>, PodcastHelperError> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| PodcastHelperError::Network {
            url: url.to_string(),
            message: e.to_string(),
        })?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(PodcastHelperError::Io)?;

    parse_feed(&bytes, url)
}

/// Fetch a feed and return only its most recent episode.
pub fn latest_episode(url: &str) -> Result<Episode, PodcastHelperError> {
    let episodes = fetch_feed(url)?;
    episodes
        .into_iter()
        .next()
        .ok_or_else(|| PodcastHelperError::EmptyFeed(url.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSS_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/rss2_feed.xml");
    const ATOM_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/atom_feed.xml");
    const RSS_NO_DATES_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/rss2_no_dates.xml");
    const INVALID_FIXTURE: &[u8] = b"this is not xml at all {{{";

    #[test]
    fn parses_rss2_fixture_newest_first() {
        let episodes = parse_feed(RSS_FIXTURE, "https://example.com/feed.xml").unwrap();
        assert_eq!(episodes.len(), 3);
        assert_eq!(episodes[0].title.as_deref(), Some("Episode 3: The Finale"));
        assert_eq!(
            episodes[0].enclosure_url,
            "https://cdn.example.com/ep3.mp3"
        );
        assert_eq!(episodes[0].enclosure_type.as_deref(), Some("audio/mpeg"));
        assert_eq!(episodes[0].enclosure_size_bytes, Some(15_728_640));
        assert_eq!(episodes[2].title.as_deref(), Some("Episode 1: The Beginning"));
    }

    #[test]
    fn parses_atom_fixture_and_finds_enclosure_link() {
        let episodes = parse_feed(ATOM_FIXTURE, "https://example.com/feed.atom").unwrap();
        assert_eq!(episodes.len(), 2);
        assert_eq!(episodes[0].title.as_deref(), Some("Atom Episode 2"));
        assert_eq!(
            episodes[0].enclosure_url,
            "https://cdn.example.com/atom-ep2.m4a"
        );
    }

    #[test]
    fn latest_from_feed_without_dates_keeps_feed_order() {
        let episodes = parse_feed(RSS_NO_DATES_FIXTURE, "https://example.com/nodates.xml").unwrap();
        assert_eq!(episodes.len(), 2);
        // No published dates anywhere: first entry in document order wins, per RSS
        // convention (newest first).
        assert_eq!(episodes[0].title.as_deref(), Some("Undated Episode A"));
    }

    #[test]
    fn invalid_xml_is_a_distinct_error_variant() {
        let err = parse_feed(INVALID_FIXTURE, "https://example.com/broken.xml").unwrap_err();
        assert!(matches!(err, PodcastHelperError::InvalidFeed { .. }));
    }

    #[test]
    fn feed_with_no_enclosures_yields_no_episodes() {
        let xml = br#"<?xml version="1.0"?>
<rss version="2.0"><channel><title>Text-only feed</title>
<item><title>Just text, no audio</title></item>
</channel></rss>"#;
        let episodes = parse_feed(xml, "https://example.com/textonly.xml").unwrap();
        assert!(episodes.is_empty());
    }
}
