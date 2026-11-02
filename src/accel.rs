//! Which decoder turns this source into PCM — and the measurement that says the
//! default is already the right one.
//!
//! # The short version
//!
//! Nothing here needs accelerating. That is a finding, not an omission, and the
//! numbers are below so the next person does not have to re-derive them.
//!
//! # What was measured, on Apple Silicon with ffmpeg 8
//!
//! macOS exposes its AudioToolbox codecs to ffmpeg as `mp3_at`, `aac_at`,
//! `alac_at` and friends, and Windows exposes Media Foundation the same way. The
//! obvious move is to prefer them, the way a video pipeline prefers a hardware
//! decoder. Timed on 600 s of audio, decoded to 16 kHz mono PCM:
//!
//! | source | decoder | time |
//! |---|---|---|
//! | mp3 | `mp3` (ffmpeg's own) | 0.03 s · 0.04 s |
//! | mp3 | `mp3_at` (AudioToolbox) | 0.07 s · 0.05 s |
//! | m4a | `aac` (ffmpeg's own) | 0.04 s · 0.05 s |
//! | m4a | `aac_at` (AudioToolbox) | 0.06 s · 0.05 s |
//!
//! The platform decoder is **slower**, consistently. It is not a hardware block
//! being used badly; it is a wrapper around an API call whose overhead is larger
//! than the decode itself, because the decode is tiny: 600 s of audio in 0.03 s
//! is roughly twenty-thousand times real time.
//!
//! Which is the real point. Audio decoding is not a bottleneck worth optimising
//! on any machine this crate will meet. For a network source, the download is
//! three orders of magnitude slower than the decode; for a local file, the disk
//! is. Adding an accelerator here would buy nothing and cost a dependency on the
//! platform behaving.
//!
//! # And video, which is not decoded at all
//!
//! The other suspicion worth killing: a podcast source is often a video
//! container, and decoding 1080p frames only to throw them away would be a real
//! waste. ffmpeg does not do it. Raw PCM output cannot carry a video stream, so
//! the stream is never mapped, so it is never decoded. Measured on a 117 MB
//! 1080p MP4: **0.03–0.04 s either way**, with `-vn` and without. The flag would
//! be decoration.
//!
//! # So what this module is for
//!
//! One thing: letting a caller overrule all of the above. The default is the
//! best choice on every machine tested, but "every machine tested" is not "every
//! machine", and a build where the platform decoder genuinely wins must not be
//! unreachable. Naming a decoder is therefore honoured exactly as given — never
//! measured, never second-guessed, never silently replaced.

/// The environment variable that pins a decoder for every extraction.
pub const DECODER_ENV: &str = "PODCAST_HELPER_DECODER";

/// Why a decoder is in use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// Named by the caller, or by [`DECODER_ENV`]. Taken as given.
    Explicit,
}

/// A decoder, and why it was chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// The ffmpeg decoder name, to pass as `-c:a` *before* the input.
    pub decoder: String,
    /// Which rung answered. Only one rung exists here, by design.
    pub reason: Reason,
}

/// The decoder to force, or `None` to let ffmpeg pick — which, per the
/// measurements above, is what you want unless you know otherwise about this
/// particular machine.
pub fn decoder(explicit: Option<&str>) -> Option<Choice> {
    let named = explicit
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var(DECODER_ENV)
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })?;
    Some(Choice {
        decoder: named,
        reason: Reason::Explicit,
    })
}

/// The arguments a forced decoder contributes, which must sit **before** `-i`:
/// after the input, `-c:a` would be read as an *output* codec and would re-encode
/// the PCM instead of selecting how it is read.
pub fn input_args(choice: Option<&Choice>) -> Vec<String> {
    match choice {
        Some(c) => vec!["-c:a".to_string(), c.decoder.clone()],
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_named_decoder_is_taken_as_given() {
        let chosen = decoder(Some("mp3_at")).expect("an explicit name always answers");
        assert_eq!(chosen.decoder, "mp3_at");
        assert_eq!(chosen.reason, Reason::Explicit);
    }

    #[test]
    fn no_name_means_no_opinion() {
        // The ordinary path: say nothing to ffmpeg, which then uses the native
        // decoder that measured fastest anyway.
        assert!(decoder(None).is_none() || std::env::var(DECODER_ENV).is_ok());
        assert!(decoder(Some("  ")).is_none() || std::env::var(DECODER_ENV).is_ok());
    }

    #[test]
    fn the_decoder_goes_before_the_input() {
        // Guards the one way this can be wired wrongly: after `-i`, `-c:a` names
        // an output codec, and the PCM would come back re-encoded.
        let choice = decoder(Some("aac_at")).unwrap();
        assert_eq!(input_args(Some(&choice)), vec!["-c:a", "aac_at"]);
        assert!(input_args(None).is_empty());
    }
}
