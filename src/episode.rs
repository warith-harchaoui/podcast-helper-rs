//! Normalized episode schema shared by RSS 0.9x/1.0/2.0, Atom and JSON Feed, so
//! callers never have to branch on which flavor the source feed used. Mirrors the
//! Python original's `Episode` dict (`guid, title, description, link, published_at,
//! duration_seconds, enclosure_url, enclosure_type, enclosure_size_bytes, image_url`).

use serde::{Deserialize, Serialize};

/// One episode, normalized regardless of the source feed's format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub guid: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    /// RFC 3339 / ISO 8601 UTC timestamp, e.g. `2026-08-31T12:00:00Z`.
    pub published_at: Option<String>,
    /// Not populated in v0.1: `itunes:duration` is a podcast-namespace extension
    /// feed-rs doesn't surface in its unified model, and parsing it would mean a
    /// second raw-XML pass. TODO: add a small itunes:duration extractor if a
    /// downstream caller needs it.
    pub duration_seconds: Option<f64>,
    pub enclosure_url: String,
    pub enclosure_type: Option<String>,
    pub enclosure_size_bytes: Option<u64>,
    pub image_url: Option<String>,
}

/// Pull the first usable audio enclosure out of an entry: prefer the MediaRSS
/// `media:content` model (feed-rs normalizes RSS `<enclosure>` into this too),
/// falling back to an Atom `<link rel="enclosure">`.
fn enclosure_from_entry(entry: &feed_rs::model::Entry) -> Option<(String, Option<String>, Option<u64>)> {
    for media in &entry.media {
        for content in &media.content {
            if let Some(url) = &content.url {
                let mime = content.content_type.as_ref().map(|m| m.to_string());
                return Some((url.to_string(), mime, content.size));
            }
        }
    }
    entry
        .links
        .iter()
        .find(|l| l.rel.as_deref() == Some("enclosure"))
        .map(|l| (l.href.clone(), l.media_type.clone(), l.length))
}

impl Episode {
    /// Build an `Episode` from a raw feed-rs entry, or `None` if it carries no
    /// audio enclosure at all (e.g. a text-only blog-style entry in a mixed feed) —
    /// those aren't episodes as far as this crate is concerned.
    pub(crate) fn try_from_entry(entry: &feed_rs::model::Entry) -> Option<Episode> {
        let (enclosure_url, enclosure_type, enclosure_size_bytes) = enclosure_from_entry(entry)?;
        Some(Episode {
            guid: Some(entry.id.clone()).filter(|s| !s.is_empty()),
            title: entry.title.as_ref().map(|t| t.content.clone()),
            description: entry.summary.as_ref().map(|s| s.content.clone()),
            link: entry.links.first().map(|l| l.href.clone()),
            published_at: entry.published.or(entry.updated).map(|dt| dt.to_rfc3339()),
            duration_seconds: None,
            enclosure_url,
            enclosure_type,
            enclosure_size_bytes,
            image_url: entry
                .media
                .iter()
                .flat_map(|m| m.thumbnails.iter())
                .next()
                .map(|t| t.image.uri.clone()),
        })
    }
}
