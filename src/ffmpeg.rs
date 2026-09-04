//! Decodes any ffmpeg-readable source (local file or URL) to PCM `f32` by shelling
//! out to the system `ffmpeg` binary. One decode path for every container/codec
//! instead of vendoring codec crates — the same tradeoff scribe-reunion's `sr-io`
//! makes for the same reason.

use crate::error::PodcastHelperError;
use crate::pcm::{ExtractOptions, PcmStream};
use std::io;
use std::process::{Command, Stdio};

pub(crate) fn decode_with_ffmpeg(
    source: &str,
    opts: &ExtractOptions,
) -> Result<PcmStream, PodcastHelperError> {
    decode_with_binary("ffmpeg", source, opts)
}

/// Same as `decode_with_ffmpeg`, parameterized on the binary name. Split out so
/// tests can point at a deliberately-missing binary to deterministically exercise
/// the `FfmpegNotFound` path — as a plain function argument rather than a process
/// env var, so it stays safe under parallel test execution.
fn decode_with_binary(
    binary: &str,
    source: &str,
    opts: &ExtractOptions,
) -> Result<PcmStream, PodcastHelperError> {
    let channels: u16 = if opts.to_mono { 1 } else { 2 };

    let spawned = Command::new(binary)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            source,
            "-f",
            "f32le",
            "-ac",
            &channels.to_string(),
            "-ar",
            &opts.sample_rate.to_string(),
            "pipe:1",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let output = match spawned {
        Ok(output) => output,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(PodcastHelperError::FfmpegNotFound(binary.to_string()));
        }
        Err(e) => return Err(PodcastHelperError::Io(e)),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(PodcastHelperError::FfmpegFailed {
            input_source: source.to_string(),
            stderr,
        });
    }

    let samples: Vec<f32> = output
        .stdout
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b))
        .collect();

    Ok(PcmStream {
        sample_rate: opts.sample_rate,
        channels,
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Writes a short synthetic sine-wave WAV via `hound` and decodes it with the
    /// real system `ffmpeg` — a genuine subprocess call, no network involved.
    fn write_sine_wav(path: &std::path::Path, seconds: f32, sample_rate: u32) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let total_samples = (seconds * sample_rate as f32) as u32;
        for i in 0..total_samples {
            let t = i as f32 / sample_rate as f32;
            let value = (t * 440.0 * 2.0 * std::f32::consts::PI).sin();
            writer
                .write_sample((value * i16::MAX as f32) as i16)
                .unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn decodes_local_wav_file_via_real_ffmpeg() {
        let dir = tempfile::tempdir().unwrap();
        let wav_path = dir.path().join("tone.wav");
        write_sine_wav(&wav_path, 1.0, 44_100);

        let opts = ExtractOptions {
            sample_rate: 16_000,
            to_mono: true,
        };
        let stream = decode_with_ffmpeg(wav_path.to_str().unwrap(), &opts).unwrap();

        assert_eq!(stream.sample_rate, 16_000);
        assert_eq!(stream.channels, 1);
        // ~1 second at 16kHz; allow slack for ffmpeg's resampler filter edges.
        assert!(
            stream.samples.len() > 15_000 && stream.samples.len() < 17_000,
            "unexpected sample count: {}",
            stream.samples.len()
        );
        // A 440Hz sine at full scale should have real signal, not silence.
        let peak = stream.samples.iter().cloned().fold(0.0_f32, f32::max);
        assert!(peak > 0.1, "expected non-trivial signal, got peak={peak}");
    }

    #[test]
    fn missing_ffmpeg_binary_is_a_distinct_error() {
        let result = decode_with_binary(
            "definitely-not-a-real-binary-xyz",
            "whatever.mp3",
            &ExtractOptions::default(),
        );
        assert!(matches!(result, Err(PodcastHelperError::FfmpegNotFound(_))));
    }

    #[test]
    fn ffmpeg_failure_on_bad_input_is_a_distinct_error() {
        let dir = tempfile::tempdir().unwrap();
        let bogus_path = dir.path().join("not_audio.mp3");
        let mut f = std::fs::File::create(&bogus_path).unwrap();
        f.write_all(b"this is not an audio file at all").unwrap();

        let result = decode_with_ffmpeg(bogus_path.to_str().unwrap(), &ExtractOptions::default());
        assert!(matches!(
            result,
            Err(PodcastHelperError::FfmpegFailed { .. })
        ));
    }
}
