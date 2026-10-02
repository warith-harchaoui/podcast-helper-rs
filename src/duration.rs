//! `itunes:duration` extraction — the one episode field `feed-rs` hands back
//! wrong often enough to be worth reading ourselves.
//!
//! feed-rs does surface the tag: its RSS 2.0 parser routes every `itunes:*` item
//! element into a synthesized `MediaObject`, so `<itunes:duration>` lands on
//! `MediaObject::duration`. The problem is how it reads the text — with the RTSP
//! "normal play time" grammar, which knows only `HH:MM:SS` and a bare seconds
//! count. The iTunes podcast spec also allows `MM:SS`, and on that form the npt
//! fallback matches the leading number instead: `45:30` comes back as 45 seconds
//! rather than 2730. Only the parsed `Duration` is returned, so the original text
//! is unrecoverable and the value cannot be repaired after the fact.
//!
//! Hence this second pass over the raw bytes. It is cheap — one streaming scan
//! with the same `quick-xml` reader feed-rs itself pulls in — and it is keyed on
//! the enclosure URL, the one field every item that interests us must carry and
//! that [`crate::Episode`] already stores.

use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::Reader;

/// Map each item's audio URL to the duration it declares, in seconds.
///
/// Items with no `itunes:duration`, no enclosure, or an unparseable value are
/// simply absent from the map — a missing duration is normal, and guessing one
/// would be worse than leaving it `None`.
pub(crate) fn durations_by_enclosure_url(xml_bytes: &[u8]) -> HashMap<String, f64> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut durations = HashMap::new();
    let mut buf = Vec::new();
    // Per-item state, reset at every `<item>`/`<entry>` boundary.
    let mut url: Option<String> = None;
    let mut seconds: Option<f64> = None;
    let mut in_duration = false;

    loop {
        match reader.read_event_into(&mut buf) {
            // A malformed tail is not a reason to discard the durations already
            // collected: feed-rs is the parser of record here, and it has its own
            // opinion about whether the document is usable at all.
            Err(_) | Ok(Event::Eof) => break,

            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                match e.local_name().as_ref() {
                    b"item" | b"entry" => {
                        url = None;
                        seconds = None;
                    }
                    // RSS `<enclosure url="...">` and MediaRSS `<media:content url="...">`.
                    // The guard keeps the first enclosure in an item, the same one
                    // `Episode::try_from_entry` picks, so the two agree on the key.
                    b"enclosure" | b"content" if url.is_none() => {
                        url = attribute(&e, b"url");
                    }
                    // Atom's equivalent: `<link rel="enclosure" href="...">`.
                    b"link"
                        if url.is_none()
                            && attribute(&e, b"rel").as_deref() == Some("enclosure") =>
                    {
                        url = attribute(&e, b"href");
                    }
                    b"duration" => in_duration = true,
                    _ => {}
                }
            }

            Ok(Event::Text(e)) if in_duration => {
                if let Ok(text) = e.decode() {
                    seconds = parse_itunes_duration(&text);
                }
            }

            Ok(Event::End(e)) => match e.local_name().as_ref() {
                b"duration" => in_duration = false,
                b"item" | b"entry" => {
                    if let (Some(url), Some(seconds)) = (url.take(), seconds.take()) {
                        durations.insert(url, seconds);
                    }
                }
                _ => {}
            },

            Ok(_) => {}
        }
        buf.clear();
    }

    durations
}

/// One attribute of a start tag, decoded, or `None` if absent or not valid UTF-8.
fn attribute(element: &quick_xml::events::BytesStart<'_>, name: &[u8]) -> Option<String> {
    let attr = element.try_get_attribute(name).ok().flatten()?;
    String::from_utf8(attr.value.into_owned()).ok()
}

/// Parse an `itunes:duration` value — `SSSS`, `MM:SS` or `HH:MM:SS`, each field
/// optionally fractional — into seconds.
///
/// Returns `None` for anything it cannot read with certainty, including a zero
/// duration: feeds use `0` as a placeholder for "not filled in", and reporting
/// that as a real zero-length episode would be a worse answer than silence.
fn parse_itunes_duration(text: &str) -> Option<f64> {
    let fields: Vec<&str> = text.trim().split(':').collect();
    if fields.len() > 3 {
        return None;
    }

    let mut total = 0.0_f64;
    for (index, field) in fields.iter().enumerate() {
        let value: f64 = field.trim().parse().ok()?;
        if !value.is_finite() || value < 0.0 {
            return None;
        }
        // Only the leading field may exceed 59: `90:00` is a legitimate ninety
        // minutes, but the 90 in `1:90:00` is not a minute count any reader would
        // agree on, so the whole value is suspect.
        if index > 0 && value >= 60.0 {
            return None;
        }
        total = total * 60.0 + value;
    }

    (total > 0.0).then_some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_shape_the_itunes_spec_allows() {
        assert_eq!(parse_itunes_duration("1800"), Some(1800.0));
        assert_eq!(parse_itunes_duration("45:30"), Some(2730.0));
        assert_eq!(parse_itunes_duration("1:02:03"), Some(3723.0));
        assert_eq!(parse_itunes_duration("01:02:03"), Some(3723.0));
        assert_eq!(parse_itunes_duration(" 90:00 "), Some(5400.0));
        assert_eq!(parse_itunes_duration("12.5"), Some(12.5));
    }

    #[test]
    fn mm_ss_is_the_form_feed_rs_gets_wrong() {
        // feed-rs's npt fallback matches the leading number and calls `45:30`
        // forty-five seconds. This is the whole reason the module exists, so the
        // expectation is pinned here rather than left implicit.
        assert_eq!(parse_itunes_duration("45:30"), Some(2730.0));
    }

    #[test]
    fn rejects_values_it_cannot_read_with_certainty() {
        assert_eq!(parse_itunes_duration(""), None);
        assert_eq!(parse_itunes_duration("about an hour"), None);
        assert_eq!(parse_itunes_duration("1:2:3:4"), None);
        assert_eq!(parse_itunes_duration("-30"), None);
        assert_eq!(parse_itunes_duration("inf"), None);
        assert_eq!(parse_itunes_duration("NaN"), None);
        // A minutes field that is not a minute count.
        assert_eq!(parse_itunes_duration("1:90:00"), None);
        // Placeholder zero, in each of its spellings.
        assert_eq!(parse_itunes_duration("0"), None);
        assert_eq!(parse_itunes_duration("00:00"), None);
    }

    #[test]
    fn maps_rss_items_to_their_declared_durations() {
        let xml = br#"<?xml version="1.0"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
  <channel>
    <item>
      <title>Plain seconds</title>
      <itunes:duration>1800</itunes:duration>
      <enclosure url="https://cdn.example.com/a.mp3" type="audio/mpeg"/>
    </item>
    <item>
      <title>Minutes and seconds</title>
      <itunes:duration>45:30</itunes:duration>
      <enclosure url="https://cdn.example.com/b.mp3" type="audio/mpeg"/>
    </item>
    <item>
      <title>No duration at all</title>
      <enclosure url="https://cdn.example.com/c.mp3" type="audio/mpeg"/>
    </item>
  </channel>
</rss>"#;
        let durations = durations_by_enclosure_url(xml);
        assert_eq!(durations.len(), 2);
        assert_eq!(durations["https://cdn.example.com/a.mp3"], 1800.0);
        assert_eq!(durations["https://cdn.example.com/b.mp3"], 2730.0);
        assert!(!durations.contains_key("https://cdn.example.com/c.mp3"));
    }

    #[test]
    fn a_duration_never_leaks_from_one_item_into_the_next() {
        // The per-item state is reset at the `<item>` boundary; if it were not,
        // the second item would inherit the first one's 1800 seconds.
        let xml = br#"<?xml version="1.0"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
  <channel>
    <item><itunes:duration>1800</itunes:duration>
      <enclosure url="https://cdn.example.com/a.mp3"/></item>
    <item><enclosure url="https://cdn.example.com/b.mp3"/></item>
  </channel>
</rss>"#;
        let durations = durations_by_enclosure_url(xml);
        assert_eq!(durations.len(), 1);
        assert_eq!(durations["https://cdn.example.com/a.mp3"], 1800.0);
    }

    #[test]
    fn reads_an_atom_enclosure_link() {
        let xml = br#"<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
  <entry>
    <title>Atom episode</title>
    <itunes:duration>12:00</itunes:duration>
    <link rel="enclosure" href="https://cdn.example.com/atom.m4a" type="audio/mp4"/>
  </entry>
</feed>"#;
        let durations = durations_by_enclosure_url(xml);
        assert_eq!(durations["https://cdn.example.com/atom.m4a"], 720.0);
    }

    #[test]
    fn a_feed_with_no_durations_yields_an_empty_map() {
        let xml = br#"<?xml version="1.0"?>
<rss version="2.0"><channel>
  <item><enclosure url="https://cdn.example.com/a.mp3"/></item>
</channel></rss>"#;
        assert!(durations_by_enclosure_url(xml).is_empty());
    }

    #[test]
    fn malformed_xml_yields_what_was_read_before_the_break_instead_of_panicking() {
        let xml = br#"<rss><channel>
  <item><itunes:duration>60</itunes:duration>
    <enclosure url="https://cdn.example.com/a.mp3"/></item>
  <item><itunes:duration>oops"#;
        // No panic, and the complete item before the truncation still counts.
        let durations = durations_by_enclosure_url(xml);
        assert_eq!(durations["https://cdn.example.com/a.mp3"], 60.0);
    }
}

#[cfg(test)]
mod feed_rs_premise {
    //! The premise this module rests on, pinned as a test: if a future `feed-rs`
    //! learns to read `MM:SS`, this fails and the second pass can be deleted.

    const RSS_FIXTURE: &[u8] = include_bytes!("../tests/fixtures/rss2_feed.xml");

    #[test]
    fn feed_rs_still_reads_45_30_as_forty_five_seconds() {
        let feed = feed_rs::parser::parse(RSS_FIXTURE).expect("fixture parses");
        let entry = feed
            .entries
            .iter()
            .find(|e| {
                e.title
                    .as_ref()
                    .is_some_and(|t| t.content.contains("Middle"))
            })
            .expect("episode 2 is in the fixture");
        let parsed = entry.media.iter().find_map(|m| m.duration);
        assert_eq!(
            parsed,
            Some(std::time::Duration::from_secs(45)),
            "feed-rs now reads MM:SS correctly — drop src/duration.rs and take \
             `MediaObject::duration` instead"
        );
    }
}
