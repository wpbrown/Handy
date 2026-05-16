use anyhow::Result;
use std::path::Path;

use vad_rs::Vad;

use super::{VadFrame, VoiceActivityDetector};
use crate::audio_toolkit::constants;

const SILERO_FRAME_MS: u32 = 30;
const SILERO_FRAME_SAMPLES: usize =
    (constants::WHISPER_SAMPLE_RATE * SILERO_FRAME_MS / 1000) as usize;

pub struct SileroVad {
    engine: Vad,
    threshold: f32,
    frame_index: u64,
}

impl SileroVad {
    pub fn new<P: AsRef<Path>>(model_path: P, threshold: f32) -> Result<Self> {
        if !(0.0..=1.0).contains(&threshold) {
            anyhow::bail!("threshold must be between 0.0 and 1.0");
        }

        Ok(Self {
            engine: Vad::new(&model_path, constants::WHISPER_SAMPLE_RATE as usize)
                .map_err(|e| anyhow::anyhow!("Failed to create VAD: {e}"))?,
            threshold,
            frame_index: 0,
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

        let result = self
            .engine
            .compute(frame)
            .map_err(|e| anyhow::anyhow!("Silero VAD error: {e}"))?;

        let idx = self.frame_index;
        self.frame_index += 1;

        // ~33 frames/sec at 16kHz/30ms; this is a debug-level firehose
        // useful for diagnosing why speech is being dropped. Enable with
        // RUST_LOG=handy_app_lib::audio_toolkit::vad::silero=debug
        let is_speech = result.prob > self.threshold;
        log::debug!(
            "silero frame={} t={:.2}s prob={:.3} thr={:.2} -> {}",
            idx,
            idx as f32 * (SILERO_FRAME_MS as f32 / 1000.0),
            result.prob,
            self.threshold,
            if is_speech { "SPEECH" } else { "noise" }
        );

        if is_speech {
            Ok(VadFrame::Speech(frame))
        } else {
            Ok(VadFrame::Noise)
        }
    }
}
