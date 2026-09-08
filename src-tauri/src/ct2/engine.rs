use std::path::{Path, PathBuf};

use ct2rs::sys::{ComputeType, Config, Device};
use ct2rs::{Whisper, WhisperOptions};
use tracing::{info, warn};

use crate::asr::engine::TranscriptionSegment;
use crate::whisper::params::TranscriptionParams;

/// CTranslate2 backed engine, mirroring the `SherpaEngine` / `WhisperEngine`
/// API so it can be swapped in through the `AsrEngine` wrapper.
///
/// Uses ct2rs' Whisper wrapper, which computes the mel spectrogram itself and
/// requires the model directory to contain `model.bin`, `config.json`,
/// `tokenizer.json` and `preprocessor_config.json` (see `ct2::models`).
pub struct Ctranslate2Engine {
    whisper: Option<Whisper>,
    model_path: Option<PathBuf>,
    model_name: Option<String>,
    gpu: bool,
    compute_type: Option<String>,
}

impl Ctranslate2Engine {
    pub fn new() -> Self {
        Self {
            whisper: None,
            model_path: None,
            model_name: None,
            gpu: false,
            compute_type: None,
        }
    }

    /// Desired compute type (e.g. "float16", "int8"). Applied at load time;
    /// if the hardware rejects it the engine falls back to a supported one.
    pub fn set_compute_type(&mut self, compute_type: String) {
        self.compute_type = Some(compute_type);
    }

    /// Load a CTranslate2 Whisper model package. `model_path` must point to a
    /// directory containing the ct2 files (e.g. `models/ct2/base/`).
    pub fn load_model(&mut self, model_path: &Path, gpu: bool) -> anyhow::Result<()> {
        if !model_path.exists() || !model_path.is_dir() {
            return Err(anyhow::anyhow!(
                "CTranslate2 model directory not found: {:?}",
                model_path
            ));
        }

        let name = model_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid model path: {:?}", model_path))?;

        let threads = 4;
        let mut last_err: Option<String> = None;

        let mut try_device = |device: Device, device_label: &str| {
            build_whisper(model_path, device, self.compute_type.as_deref(), threads).map_err(|e| {
                warn!(
                    "Failed to load CTranslate2 model '{}' on {}: {}",
                    name, device_label, e
                );
                last_err = Some(format!("{}: {}", device_label, e));
                e
            })
        };

        let whisper = if gpu {
            try_device(Device::CUDA, "CUDA").or_else(|_| {
                warn!("CTranslate2 model '{}' fell back to CPU", name);
                try_device(Device::CPU, "CPU")
            })
        } else {
            try_device(Device::CPU, "CPU")
        }
        .map_err(|_| {
            anyhow::anyhow!(
                "Failed to load CTranslate2 model '{}' ({})",
                name,
                last_err.as_deref().unwrap_or("unknown error")
            )
        })?;

        self.whisper = Some(whisper);
        self.model_path = Some(model_path.to_path_buf());
        self.model_name = Some(name.to_string());
        self.gpu = gpu;

        info!("CTranslate2 model '{}' loaded (gpu={})", name, gpu);
        Ok(())
    }

    pub fn transcribe(
        &self,
        audio: &[f32],
        params: &TranscriptionParams,
    ) -> anyhow::Result<Vec<TranscriptionSegment>> {
        let whisper = self
            .whisper
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("CTranslate2 model not loaded"))?;

        let sampling_rate = whisper.sampling_rate() as f64;
        let n_samples = whisper.n_samples().max(1);
        let chunk_len = n_samples as f64 / sampling_rate;
        let audio_duration = audio.len() as f64 / sampling_rate;

        let language = match params.language.as_deref() {
            Some(lang) if !lang.is_empty() && lang != "auto" => Some(lang),
            _ => None,
        };

        let options = WhisperOptions {
            beam_size: params.beam_size,
            suppress_blank: true,
            repetition_penalty: 1.0,
            ..Default::default()
        };

        let lines = whisper
            .generate(audio, language, true, &options)
            .map_err(|e| anyhow::anyhow!("CTranslate2 transcribe failed: {}", e))?;

        let mut segments: Vec<TranscriptionSegment> = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let base = i as f64 * chunk_len;
            segments.extend(parse_timestamped(line, base, chunk_len));
        }

        // Clamp the trailing segment so it never outlives the source audio.
        if let Some(last) = segments.last_mut() {
            if last.end > audio_duration {
                last.end = (audio_duration - last.start).max(0.5) + last.start;
            }
        }

        Ok(segments)
    }

    pub fn is_loaded(&self) -> bool {
        self.whisper.is_some()
    }

    pub fn model_path(&self) -> Option<PathBuf> {
        self.model_path.clone()
    }

    pub fn model_name(&self) -> Option<String> {
        self.model_name.clone()
    }
}

impl Default for Ctranslate2Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// Try loading the model with the requested compute type, falling back to
/// increasingly safe ones the hardware is likely to support.
fn build_whisper(
    model_path: &Path,
    device: Device,
    requested: Option<&str>,
    threads: usize,
) -> anyhow::Result<Whisper> {
    let requested = requested.unwrap_or("float16");

    let candidates: Vec<&str> = match requested {
        "float16" => vec!["float16", "float32", "int8"],
        "int8" => vec!["int8", "float32"],
        "int8_float16" => vec!["int8_float16", "float32", "int8"],
        "int8_float32" => vec!["int8_float32", "float32", "int8"],
        other => vec![other, "float32"],
    };

    let mut last_err: Option<String> = None;
    for ct in candidates {
        let mut config = Config {
            device,
            compute_type: compute_type_from_str(ct),
            ..Default::default()
        };
        if device == Device::CPU {
            config.num_threads_per_replica = threads;
        }

        match Whisper::new(model_path, config) {
            Ok(w) => {
                info!("CTranslate2 loaded on device with compute_type='{}'", ct);
                return Ok(w);
            }
            Err(e) => {
                warn!("CTranslate2 failed with compute_type='{}': {}", ct, e);
                last_err = Some(e.to_string());
            }
        }
    }

    Err(anyhow::anyhow!(
        "no supported compute type: {}",
        last_err.unwrap_or_default()
    ))
}

fn compute_type_from_str(s: &str) -> ComputeType {
    match s {
        "int8_float16" => ComputeType::INT8_FLOAT16,
        "int8_float32" => ComputeType::INT8_FLOAT32,
        "int16" => ComputeType::INT16,
        "int8" => ComputeType::INT8,
        "float32" => ComputeType::FLOAT32,
        "bfloat16" => ComputeType::BFLOAT16,
        "auto" => ComputeType::AUTO,
        _ => ComputeType::FLOAT16,
    }
}

/// Parse one decoded chunk line from ct2rs' `generate(timestamp=true)`.
///
/// The wrapper decodes raw token ids, so the line is plain text with
/// timestamp markers inline, e.g.:
/// `startoftranscript|><|es|><|transcribe|><|0.00|>Hola<|1.30|>mundo<|2.90|>`
/// (the opening `<|` is consumed by the scanner). Times are relative to the
/// audio passed to `generate`; `base` shifts them to absolute file time.
fn parse_timestamped(line: &str, base: f64, chunk_len: f64) -> Vec<TranscriptionSegment> {
    let mut segments: Vec<TranscriptionSegment> = Vec::new();
    let mut cur_start: Option<f64> = None;
    let mut text = String::new();
    let mut plain: Vec<char> = Vec::new();

    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        if chars[i] == '<' && i + 1 < n && chars[i + 1] == '|' {
            // Read the marker body up to the closing "|>".
            let mut j = i + 2;
            let mut marker = String::new();
            while j + 1 < n {
                if chars[j] == '|' && chars[j + 1] == '>' {
                    break;
                }
                marker.push(chars[j]);
                j += 1;
            }
            if j + 1 >= n {
                break;
            }

            // Flush any accumulated plain text before this marker.
            if !plain.is_empty() {
                let t: String = plain.iter().collect();
                let t = t.trim();
                if !t.is_empty() {
                    if text.is_empty() {
                        text = t.to_string();
                    } else {
                        text.push(' ');
                        text.push_str(t);
                    }
                }
                plain.clear();
            }

            if let Some(seconds) = parse_ts_marker(&marker) {
                if let Some(start) = cur_start.take() {
                    let end = seconds.max(start + 0.1);
                    segments.push(TranscriptionSegment {
                        start: base + start,
                        end: base + end,
                        text: text.trim().to_string(),
                    });
                    text.clear();
                }
                cur_start = Some(seconds);
            }
            // Special markers (startoftranscript, language, transcribe, ...)
            // carry no timing and are dropped.

            i = j + 2;
        } else {
            plain.push(chars[i]);
            i += 1;
        }
    }

    if !plain.is_empty() {
        let t: String = plain.iter().collect();
        let t = t.trim();
        if !t.is_empty() {
            if text.is_empty() {
                text = t.to_string();
            } else {
                text.push(' ');
                text.push_str(t);
            }
        }
    }

    // Closing segment: no end marker; estimate from text length.
    if let Some(start) = cur_start {
        if !text.trim().is_empty() {
            let est = (text.chars().count() as f64 / 14.0).max(1.0);
            let end = (start + est).min(chunk_len).max(start + 0.5);
            segments.push(TranscriptionSegment {
                start: base + start,
                end: base + end,
                text: text.trim().to_string(),
            });
        }
    }

    segments
}

/// Parse a numeric Whisper timestamp marker like `0.00`, `1.05` or `12.34`
/// (format `{minutes}.{seconds % 60}`) into absolute seconds.
fn parse_ts_marker(marker: &str) -> Option<f64> {
    let (min_part, sec_part) = marker.split_once('.')?;
    let minutes: f64 = min_part.parse().ok()?;
    let seconds: f64 = sec_part.parse().ok()?;
    Some(minutes * 60.0 + seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ts_marker_seconds() {
        assert_eq!(parse_ts_marker("0.00"), Some(0.0));
        assert_eq!(parse_ts_marker("0.05"), Some(5.0));
        assert_eq!(parse_ts_marker("1.05"), Some(65.0));
        assert_eq!(parse_ts_marker("12.34"), Some(754.0));
        assert_eq!(parse_ts_marker("es"), None);
    }

    #[test]
    fn parse_line_with_timestamps() {
        let line = "<|startoftranscript|><|es|><|transcribe|><|0.00|>Hola<|0.05|>mundo<|0.09|>";
        let segs = parse_timestamped(line, 30.0, 30.0);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].text, "Hola");
        assert_eq!(segs[0].start, 30.0);
        assert_eq!(segs[0].end, 35.0);
        assert_eq!(segs[1].text, "mundo");
        assert_eq!(segs[1].start, 35.0);
        assert_eq!(segs[1].end, 39.0);
    }

    #[test]
    fn parse_line_no_timestamps_falls_back() {
        let segs = parse_timestamped("texto sin marcas", 0.0, 30.0);
        assert!(segs.is_empty());
    }

    #[test]
    fn final_segment_gets_estimated_end() {
        let line = "<|0.00|>ultimo segmento sin cierre";
        let segs = parse_timestamped(line, 0.0, 30.0);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].text, "ultimo segmento sin cierre");
        assert!(segs[0].end > segs[0].start);
    }

    #[test]
    fn compute_type_mapping() {
        assert_eq!(compute_type_from_str("float16"), ComputeType::FLOAT16);
        assert_eq!(compute_type_from_str("int8"), ComputeType::INT8);
        assert_eq!(
            compute_type_from_str("int8_float16"),
            ComputeType::INT8_FLOAT16
        );
        assert_eq!(compute_type_from_str("float32"), ComputeType::FLOAT32);
    }
}
