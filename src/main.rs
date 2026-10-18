//! `podcast-helper-rs` — the command-line front end for [`podcast_helper_rs`].
//!
//! Thin by design, like the rest of this suite's binaries: parse arguments,
//! call one library function, print, exit.
//!
//! Built only with the `cli` feature, which is on by default so
//! `cargo install podcast-helper-rs` works, and off for anyone depending on
//! the crate as a library with `default-features = false`.
//!
//! The binary is named after the crate rather than after the Python twin's
//! `podcast-helper` command: the two are meant to coexist, and identical names
//! would let `PATH` order decide which one runs.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use podcast_helper_rs::{
    classify_source, extract_audio_stream_with_options, fetch_feed, latest_episode,
    resolve_stream_url, ExtractOptions, SourceKind,
};

/// Podcast feeds and the audio behind them: parse, resolve, decode to PCM.
#[derive(Parser, Debug)]
#[command(name = "podcast-helper-rs", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Fetch an RSS or Atom feed and print its episodes as JSON.
    Feed {
        /// The feed URL.
        url: String,
        /// Indent the JSON instead of emitting one dense line.
        #[arg(short, long)]
        pretty: bool,
    },
    /// Print only the most recent episode of a feed, as JSON.
    Latest {
        url: String,
        #[arg(short, long)]
        pretty: bool,
    },
    /// Say what kind of source this is, without fetching anything.
    Classify {
        /// A local path, a direct audio URL, a feed URL, or a catalog page.
        source: String,
    },
    /// Resolve a source to a URL ffmpeg can read.
    Resolve { source: String },
    /// Decode a source to PCM and write it as a WAV.
    Extract {
        source: String,
        output: PathBuf,
        /// Target sample rate in Hz.
        #[arg(long, default_value_t = 16_000)]
        sample_rate: u32,
        /// Keep both channels instead of downmixing to mono.
        #[arg(long)]
        stereo: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let outcome: Result<String, String> = match cli.command {
        Command::Feed { url, pretty } => fetch_feed(&url)
            .map_err(|e| e.to_string())
            .and_then(|episodes| json(&episodes, pretty)),
        Command::Latest { url, pretty } => latest_episode(&url)
            .map_err(|e| e.to_string())
            .and_then(|episode| json(&episode, pretty)),
        // Classification is pure string work: no network, no filesystem, so it
        // answers even for a path that does not exist yet.
        Command::Classify { source } => Ok(describe(&classify_source(&source))),
        Command::Resolve { source } => resolve_stream_url(&source).map_err(|e| e.to_string()),
        Command::Extract {
            source,
            output,
            sample_rate,
            stereo,
        } => {
            let options = ExtractOptions {
                sample_rate,
                to_mono: !stereo,
            };
            extract_audio_stream_with_options(&source, &options)
                .map_err(|e| e.to_string())
                .and_then(|pcm| write_wav(&pcm, &output))
        }
    };

    match outcome {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("podcast-helper-rs: {message}");
            ExitCode::FAILURE
        }
    }
}

fn json<T: serde::Serialize>(value: &T, pretty: bool) -> Result<String, String> {
    if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .map_err(|e| e.to_string())
}

/// One line naming the variant and what it points at.
///
/// `SourceKind` is `#[non_exhaustive]`-shaped in spirit — new kinds of source
/// keep appearing — so the catch-all arm is deliberate rather than an
/// oversight.
fn describe(kind: &SourceKind) -> String {
    match kind {
        SourceKind::LocalFile(path) => format!("local-file\t{}", path.display()),
        SourceKind::DirectAudioUrl(url) => format!("direct-audio-url\t{url}"),
        SourceKind::Feed(url) => format!("feed\t{url}"),
        SourceKind::YtDlp(url) => format!("yt-dlp\t{url}"),
        SourceKind::Drm { platform, .. } => format!("drm\t{platform}"),
        other => format!("unknown\t{other:?}"),
    }
}

/// Write the decoded samples as a 32-bit float WAV.
///
/// Float, because that is what the decoder produced and quantising on the way
/// out would be a loss the caller never asked for. Anything that needs integer
/// PCM can convert with `audio-helper-rs convert`.
fn write_wav(pcm: &podcast_helper_rs::PcmStream, output: &PathBuf) -> Result<String, String> {
    let spec = hound::WavSpec {
        channels: pcm.channels,
        sample_rate: pcm.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(output, spec).map_err(|e| e.to_string())?;
    for sample in &pcm.samples {
        writer.write_sample(*sample).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(output.display().to_string())
}
