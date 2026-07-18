use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerTurn {
    pub start: f64,
    pub end: f64,
    pub speaker: String,
}

/// Diarization engine using speakrs for speaker identification.
///
/// Models (~30MB) are downloaded on first use from HuggingFace.
/// Requires audio as f32 samples at 16kHz mono.
pub struct DiarizationEngine {
    initialized: bool,
}

impl DiarizationEngine {
    pub fn new() -> Self {
        info!("Initializing diarization engine (speakrs)");
        Self { initialized: true }
    }

    pub fn new_placeholder() -> Self {
        Self { initialized: false }
    }

    /// Perform speaker diarization on a WAV audio file.
    /// Reads the WAV file, runs speakrs pipeline, and returns speaker turns.
    pub fn diarize(&self, audio_path: &std::path::Path) -> anyhow::Result<Vec<SpeakerTurn>> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Diarization engine not initialized"));
        }

        info!("Starting diarization on {:?}", audio_path);

        // Read WAV file to f32 samples
        let mut reader = hound::WavReader::open(audio_path)
            .map_err(|e| anyhow::anyhow!("Failed to open WAV for diarization: {}", e))?;

        let spec = reader.spec();
        if spec.channels != 1 {
            return Err(anyhow::anyhow!(
                "Diarization requires mono audio, got {} channels",
                spec.channels
            ));
        }

        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / (1i32 << (spec.bits_per_sample - 1)) as f32))
                .collect::<Result<Vec<_>, _>>()?,
            hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?,
        };

        // Resample to 16kHz if needed
        let target_rate = 16000u32;
        let audio = if spec.sample_rate != target_rate {
            info!(
                "Resampling from {}Hz to {}Hz",
                spec.sample_rate, target_rate
            );
            resample(&samples, spec.sample_rate, target_rate)
        } else {
            samples
        };

        // Run speakrs pipeline
        let mut pipeline = speakrs::OwnedDiarizationPipeline::from_pretrained(
            speakrs::ExecutionMode::Cpu,
        )
        .map_err(|e| anyhow::anyhow!("Failed to init speakrs pipeline: {}", e))?;

        let result = pipeline
            .run(&audio)
            .map_err(|e| anyhow::anyhow!("Diarization failed: {}", e))?;

        let turns: Vec<SpeakerTurn> = result
            .segments
            .into_iter()
            .map(|seg| SpeakerTurn {
                start: seg.start,
                end: seg.end,
                speaker: seg.speaker,
            })
            .collect();

        info!("Diarization complete: {} speaker turns", turns.len());
        Ok(turns)
    }
}

/// Simple linear resampling from one sample rate to another.
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        let s0 = samples[idx.min(samples.len() - 1)];
        let s1 = samples[(idx + 1).min(samples.len() - 1)];
        output.push(s0 + (s1 - s0) * frac as f32);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speaker_turn_struct() {
        let turn = SpeakerTurn {
            start: 1.5,
            end: 5.2,
            speaker: "Speaker_0".to_string(),
        };
        assert_eq!(turn.start, 1.5);
        assert_eq!(turn.end, 5.2);
        assert_eq!(turn.speaker, "Speaker_0");
    }

    #[test]
    fn test_speaker_turn_serialization() {
        let turn = SpeakerTurn {
            start: 0.0,
            end: 10.5,
            speaker: "SPEAKER_01".to_string(),
        };
        let json = serde_json::to_string(&turn).unwrap();
        let parsed: SpeakerTurn = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.start, 0.0);
        assert_eq!(parsed.end, 10.5);
        assert_eq!(parsed.speaker, "SPEAKER_01");
    }

    #[test]
    fn test_resample_identity() {
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        let resampled = resample(&samples, 16000, 16000);
        assert_eq!(resampled, samples);
    }

    #[test]
    fn test_resample_downsample() {
        let samples: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let resampled = resample(&samples, 48000, 16000);
        assert_eq!(resampled.len(), 33);
    }

    #[test]
    fn test_resample_empty() {
        let resampled = resample(&[], 48000, 16000);
        assert!(resampled.is_empty());
    }

    #[test]
    fn test_diarization_engine_placeholder() {
        let engine = DiarizationEngine::new_placeholder();
        let result = engine.diarize(std::path::Path::new("/nonexistent.wav"));
        assert!(result.is_err());
    }
}
