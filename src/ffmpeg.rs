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
/// Schemes ffmpeg reads over a socket, and therefore the ones that can stall.
const NETWORK_SCHEMES: [&str; 8] = [
    "http://", "https://", "rtmp://", "rtmps://", "rtsp://", "tcp://", "tls://", "srt://",
];

/// True when ffmpeg will read `source` over a socket rather than from disk.
fn is_network_source(source: &str) -> bool {
    let lowered = source.to_ascii_lowercase();
    NETWORK_SCHEMES
        .iter()
        .any(|scheme| lowered.starts_with(scheme))
}

/// Microseconds a socket read may stall before ffmpeg gives up.
///
/// 60 s is far past any healthy stall, including a slow live source, while
/// still being a bound. `PODCAST_HELPER_RW_TIMEOUT` is read in **seconds** and
/// is the same variable the Python twin honours; `0` disables the bound for a
/// caller who would genuinely rather wait forever.
fn rw_timeout_micros() -> u64 {
    parse_rw_timeout(std::env::var("PODCAST_HELPER_RW_TIMEOUT").ok().as_deref())
}

/// The parsing half, taking the raw value as an argument.
///
/// Split out so the tests never touch a process-wide env var: this crate
/// already learned that lesson for the ffmpeg binary path, which is passed as
/// a function argument precisely "so it stays safe under parallel test
/// execution". Anything unparseable or negative falls back to the default
/// rather than removing the bound -- an exported-but-empty variable is a
/// common shell accident and must not silently mean "wait forever".
fn parse_rw_timeout(raw: Option<&str>) -> u64 {
    let seconds = raw
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0)
        .unwrap_or(60.0);
    (seconds * 1_000_000.0) as u64
}

/// env var, so it stays safe under parallel test execution.
fn decode_with_binary(
    binary: &str,
    source: &str,
    opts: &ExtractOptions,
) -> Result<PcmStream, PodcastHelperError> {
    let channels: u16 = if opts.to_mono { 1 } else { 2 };

    // Bound the read before -i, or a stalled source is waited on forever.
    //
    // ffmpeg applies no I/O timeout unless asked: a connection the remote has
    // half-closed leaves ffmpeg at 0 % CPU on a socket that will never deliver
    // another byte, and `.output()` below waits for a process that will never
    // exit. The Python twin was observed doing exactly this, and the same
    // failure reached four other packages in this suite.
    //
    // -rw_timeout is a protocol option, so it has to precede -i, and it means
    // nothing for a local path -- a file on disk cannot stall on a socket.
    // Same env var as the Python twin, so one knob covers both.
    let mut args: Vec<String> = vec!["-hide_banner".into(), "-loglevel".into(), "error".into()];
    if is_network_source(source) {
        let micros = rw_timeout_micros();
        if micros > 0 {
            args.push("-rw_timeout".into());
            args.push(micros.to_string());
        }
    }

    let spawned = Command::new(binary)
        .args(&args)
        .args([
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

    /// Whether `ffmpeg` can actually be executed on this machine. Probed once:
    /// several tests ask, and spawning a process per test would dominate them.
    fn ffmpeg_available() -> bool {
        use std::sync::OnceLock;
        static AVAILABLE: OnceLock<bool> = OnceLock::new();
        *AVAILABLE.get_or_init(|| {
            std::process::Command::new("ffmpeg")
                .arg("-version")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
        })
    }

    /// `true` when the caller should end early because ffmpeg is not installed.
    ///
    /// A contributor without ffmpeg should see a one-line skip, not a panic that
    /// reads like the crate is broken. Under CI the opposite is true: a missing
    /// ffmpeg means the workflow forgot to install it, and skipping would make
    /// the run green while proving nothing — so there it fails loudly instead.
    #[must_use]
    fn skip_without_ffmpeg() -> bool {
        if ffmpeg_available() {
            return false;
        }
        // Reading the environment is the other half of the race: see
        // `crate::ENV_MUTEX`. Taking it here is what makes the writer's lock
        // in `lib.rs` mean anything.
        let _guard = crate::ENV_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(
            std::env::var_os("CI").is_none(),
            "ffmpeg is missing under CI — the workflow must install it \
             (see .github/workflows/ci.yml)"
        );
        eprintln!("skipped: ffmpeg not on PATH (install ffmpeg to run the full suite)");
        true
    }

    #[test]
    fn decodes_local_wav_file_via_real_ffmpeg() {
        if skip_without_ffmpeg() {
            return;
        }
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
        if skip_without_ffmpeg() {
            return;
        }
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

#[cfg(test)]
mod timeout_tests {
    use super::{is_network_source, parse_rw_timeout};

    // ffmpeg applies no I/O timeout unless asked, so a half-closed connection
    // is waited on forever -- 0 % CPU, socket open, and `.output()` blocking on
    // a process that will never exit. The same root cause reached five packages
    // in this suite before it was found.

    #[test]
    fn network_schemes_are_recognised() {
        assert!(is_network_source("https://cdn.example.com/ep1.mp3"));
        assert!(is_network_source("HTTP://CDN.EXAMPLE.COM/ep1.mp3"));
        assert!(is_network_source("rtsp://host/stream"));
    }

    #[test]
    fn local_paths_are_not_network_sources() {
        // A file on disk cannot stall on a socket, so the flag would be noise.
        assert!(!is_network_source("/tmp/episode.mp3"));
        assert!(!is_network_source("relative/episode.wav"));
    }

    #[test]
    fn the_default_bound_is_finite() {
        // The property that matters is simply that it is never unbounded.
        assert!(parse_rw_timeout(None) > 0);
    }

    #[test]
    fn the_bound_is_overridable_in_seconds() {
        assert_eq!(parse_rw_timeout(Some("5")), 5_000_000);
    }

    #[test]
    fn zero_disables_the_bound_deliberately() {
        // Waiting forever is defensible as a choice, not as a default.
        assert_eq!(parse_rw_timeout(Some("0")), 0);
    }

    #[test]
    fn a_nonsense_override_falls_back_to_the_default() {
        // An exported-but-empty variable is a common shell accident; it must
        // not turn into `-rw_timeout 0` and silently remove the bound.
        for bad in ["", "   ", "not-a-number", "-5"] {
            assert!(
                parse_rw_timeout(Some(bad)) > 0,
                "{bad:?} should fall back, not disable"
            );
        }
    }
}
