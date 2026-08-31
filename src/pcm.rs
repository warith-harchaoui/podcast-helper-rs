//! The output shape: raw PCM samples plus the parameters needed to interpret them.
//! "URL in, PCM out" — this is the "out" half.

/// Decoded, uncompressed audio: raw pulse-code-modulation samples as `f32` in
/// `[-1.0, 1.0]`, interleaved across channels when `channels > 1`.
#[derive(Debug, Clone, PartialEq)]
pub struct PcmStream {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

impl PcmStream {
    /// Number of samples per channel (i.e. frame count), not the raw `samples.len()`.
    pub fn frame_count(&self) -> usize {
        if self.channels == 0 {
            0
        } else {
            self.samples.len() / self.channels as usize
        }
    }

    /// Duration implied by `frame_count()` and `sample_rate`.
    pub fn duration_seconds(&self) -> f64 {
        if self.sample_rate == 0 {
            0.0
        } else {
            self.frame_count() as f64 / self.sample_rate as f64
        }
    }
}

/// Decoding parameters. Defaults match the Python original's quick-start example:
/// 16 kHz mono, the shape most speech-recognition/VAD models expect.
#[derive(Debug, Clone, Copy)]
pub struct ExtractOptions {
    pub sample_rate: u32,
    pub to_mono: bool,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            sample_rate: 16_000,
            to_mono: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_count_and_duration_for_mono() {
        let stream = PcmStream {
            sample_rate: 16_000,
            channels: 1,
            samples: vec![0.0; 16_000],
        };
        assert_eq!(stream.frame_count(), 16_000);
        assert!((stream.duration_seconds() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn frame_count_and_duration_for_stereo() {
        let stream = PcmStream {
            sample_rate: 16_000,
            channels: 2,
            samples: vec![0.0; 32_000],
        };
        assert_eq!(stream.frame_count(), 16_000);
        assert!((stream.duration_seconds() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn zero_channels_does_not_panic() {
        let stream = PcmStream {
            sample_rate: 16_000,
            channels: 0,
            samples: vec![],
        };
        assert_eq!(stream.frame_count(), 0);
    }

    #[test]
    fn zero_sample_rate_does_not_panic() {
        let stream = PcmStream {
            sample_rate: 0,
            channels: 1,
            samples: vec![1.0, 2.0],
        };
        assert_eq!(stream.duration_seconds(), 0.0);
    }

    #[test]
    fn default_options_match_python_quick_start() {
        let opts = ExtractOptions::default();
        assert_eq!(opts.sample_rate, 16_000);
        assert!(opts.to_mono);
    }
}
