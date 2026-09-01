//! Delegates yt-dlp-compatible sources (YouTube, Vimeo, SoundCloud, Twitch, ...)
//! to [`youtube_helper_rs`]: download the audio track to a temporary WAV file,
//! then decode it through the same `ffmpeg` path used for local files, so
//! sample-rate/channel handling stays uniform across every source kind this
//! crate supports.

use crate::error::PodcastHelperError;
use crate::ffmpeg::decode_with_ffmpeg;
use crate::pcm::{ExtractOptions, PcmStream};

pub(crate) fn extract_via_ytdlp(
    url: &str,
    opts: &ExtractOptions,
) -> Result<PcmStream, PodcastHelperError> {
    let tmp_dir = tempfile::tempdir().map_err(PodcastHelperError::Io)?;
    let wav_path = youtube_helper_rs::download_audio(url, tmp_dir.path()).map_err(|source| {
        PodcastHelperError::YtDlp {
            url: url.to_string(),
            source,
        }
    })?;
    decode_with_ffmpeg(&wav_path.to_string_lossy(), opts)
}
