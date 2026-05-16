use anyhow::Result;

use wavekat_vad::{backends::silero::SileroVad as WkSilero, VoiceActivityDetector as WkVad};

use super::{VadFrame, VoiceActivityDetector};
use crate::audio_toolkit::constants;

// Silero v6 (bundled by wavekat-vad) requires 512-sample / 32ms frames at 16kHz.
const SILERO_FRAME_SAMPLES: usize = 512;

pub struct SileroVad {
    engine: WkSilero,
    threshold: f32,
}

impl SileroVad {
    /// Create a new Silero VAD. The Silero v6 ONNX model is embedded in
    /// the binary at compile time by `wavekat-vad`, so no external model
    /// file is required.
    pub fn new(threshold: f32) -> Result<Self> {
        if !(0.0..=1.0).contains(&threshold) {
            anyhow::bail!("threshold must be between 0.0 and 1.0");
        }

        let engine = WkSilero::new(constants::WHISPER_SAMPLE_RATE)
            .map_err(|e| anyhow::anyhow!("Failed to create Silero VAD: {e}"))?;

        Ok(Self {
            engine,
            threshold,
        })
    }
}

impl VoiceActivityDetector for SileroVad {
    fn push_frame<'a>(&'a mut self, frame: &'a [f32]) -> Result<VadFrame<'a>> {
        if frame.len() != SILERO_FRAME_SAMPLES {
            anyhow::bail!(
                "expected {SILERO_FRAME_SAMPLES} samples, got {}",
                frame.len()
            );
        }

        let samples_i16: Vec<i16> = frame
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();

        let prob = self
            .engine
            .process(&samples_i16, constants::WHISPER_SAMPLE_RATE)
            .map_err(|e| anyhow::anyhow!("Silero VAD error: {e}"))?;

        if prob > self.threshold {
            Ok(VadFrame::Speech(frame))
        } else {
            Ok(VadFrame::Noise)
        }
    }
}
