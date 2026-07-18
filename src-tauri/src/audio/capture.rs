use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig, BufferSize};
use std::sync::{Arc, Mutex, atomic::{AtomicU32, AtomicUsize, Ordering}};
use std::sync::atomic::AtomicU64;
use tracing::{error, info};
use serde::{Serialize, Deserialize};

use super::buffer::RingBuffer;

pub static CALLBACK_COUNT: AtomicU64 = AtomicU64::new(0);
pub static SAMPLES_PUSHED: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub channels: u16,
    pub sample_rate: u32,
    pub kind: String, // "mic" or "system"
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioSource {
    System,
    Microphone,
    Both,
}

impl AudioSource {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "microphone" | "mic" => Self::Microphone,
            "both" => Self::Both,
            _ => Self::System,
        }
    }
}

pub struct AudioCapture {
    stream: Option<Stream>,
    config: StreamConfig,
    sample_format: SampleFormat,
}

/// Shared audio level for real-time metering (0.0 to 1.0)
pub static AUDIO_LEVEL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn get_audio_level() -> f32 {
    AUDIO_LEVEL.load(Ordering::Relaxed) as f32 / 1_000_000.0
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            config: StreamConfig {
                channels: 1,
                sample_rate: 16000u32,
                buffer_size: BufferSize::Default,
            },
            sample_format: SampleFormat::F32,
        }
    }

    pub fn list_devices() -> anyhow::Result<Vec<AudioDeviceInfo>> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        let mut seen = std::collections::HashSet::new();

        if let Ok(input_devices) = host.input_devices() {
            for device in input_devices {
                let name = device.to_string();
                if seen.contains(&name) {
                    continue;
                }
                seen.insert(name.clone());
                if let Ok(cfg) = device.default_input_config() {
                    #[cfg(target_os = "macos")]
                    let is_monitor = name.contains("BlackHole")
                        || name.contains("Soundflower")
                        || name.contains("Aggregate Device")
                        || name.contains("System Audio");

                    #[cfg(target_os = "linux")]
                    let is_monitor = name.contains("Monitor")
                        || name.contains("monitor")
                        || name.contains("sink")
                        || name.contains("Sink");

                    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
                    let is_monitor = name.contains("Stereo Mix");

                    let kind = if is_monitor { "system" } else { "mic" };
                    let info = AudioDeviceInfo {
                        name: name.clone(),
                        channels: cfg.channels(),
                        sample_rate: cfg.sample_rate(),
                        kind: kind.to_string(),
                    };
                    info!("  Device: {} [{}ch {}Hz] ({})", name, cfg.channels(), cfg.sample_rate(), kind);
                    devices.push(info);
                }
            }
        }

        info!("Total input devices: {}", devices.len());
        Ok(devices)
    }

    pub fn start(
        &mut self,
        buffer: Arc<Mutex<RingBuffer>>,
        device_name: Option<&str>,
        actual_sample_rate: Arc<AtomicU32>,
        actual_channels: Arc<AtomicUsize>,
    ) -> anyhow::Result<()> {
        let host = cpal::default_host();

        info!("Audio host: {:?}", host.id());

        let device = if let Some(name) = device_name {
            info!("Looking for device: {}", name);
            // Strip format info like " [2ch 48000Hz]" for matching
            let clean_name = name.split('[').next().unwrap_or(name).trim().to_string();
            host.input_devices()?
                .chain(host.output_devices()?)
                .find(|d| {
                    let n = d.to_string();
                    n == clean_name || n == name || clean_name.contains(&n) || n.contains(&clean_name)
                })
                .ok_or_else(|| anyhow::anyhow!("Device '{}' not found", name))?
        } else {
            info!("Using default input device");
            host.default_input_device()
                .ok_or_else(|| anyhow::anyhow!("No default input device"))?
        };

        info!("Using audio device: {}", device);

        // Reset capture stats
        CALLBACK_COUNT.store(0, Ordering::Relaxed);
        SAMPLES_PUSHED.store(0, Ordering::Relaxed);

        let supported_config = device.default_input_config()?;
        let src_channels = supported_config.channels() as usize;
        let src_rate = supported_config.sample_rate();
        info!("Device config: {} channels, {}Hz, {:?}", src_channels, src_rate, supported_config.sample_format());

        actual_sample_rate.store(src_rate, Ordering::SeqCst);
        actual_channels.store(src_channels, Ordering::SeqCst);

        self.sample_format = supported_config.sample_format();
        self.config = supported_config.into();

        let buffer_clone = buffer.clone();
        let sample_format = self.sample_format;

        let stream = match sample_format {
            SampleFormat::F32 => {
                let config = self.config;
                device.build_input_stream(
                    config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        CALLBACK_COUNT.fetch_add(1, Ordering::Relaxed);
                        let mono = to_mono(data, src_channels);
                        // Update audio level meter
                        let rms: f32 = mono.iter().map(|&s| s * s).sum::<f32>() / mono.len() as f32;
                        let level = (rms.sqrt() * 1_000_000.0) as u32;
                        AUDIO_LEVEL.store(level, Ordering::Relaxed);
                        let resampled = simple_resample(&mono, src_rate, 16000);
                        if !resampled.is_empty() {
                            SAMPLES_PUSHED.fetch_add(resampled.len() as u64, Ordering::Relaxed);
                            if let Ok(mut buf) = buffer_clone.lock() {
                                buf.push(&resampled);
                            }
                        }
                    },
                    |err| error!("Audio stream error: {}", err),
                    None,
                )?
            }
            SampleFormat::I16 => {
                let config = self.config;
                device.build_input_stream(
                    config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        CALLBACK_COUNT.fetch_add(1, Ordering::Relaxed);
                        let float_data: Vec<f32> =
                            data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                        let mono = to_mono(&float_data, src_channels);
                        // Update audio level meter
                        let rms: f32 = mono.iter().map(|&s| s * s).sum::<f32>() / mono.len() as f32;
                        let level = (rms.sqrt() * 1_000_000.0) as u32;
                        AUDIO_LEVEL.store(level, Ordering::Relaxed);
                        let resampled = simple_resample(&mono, src_rate, 16000);
                        if !resampled.is_empty() {
                            SAMPLES_PUSHED.fetch_add(resampled.len() as u64, Ordering::Relaxed);
                            if let Ok(mut buf) = buffer_clone.lock() {
                                buf.push(&resampled);
                            }
                        }
                    },
                    |err| error!("Audio stream error: {}", err),
                    None,
                )?
            }
            _ => return Err(anyhow::anyhow!("Unsupported sample format: {:?}", sample_format)),
        };

        stream.play()?;
        self.stream = Some(stream);

        info!("Audio capture started ({}ch {}Hz → mono 16kHz)", src_channels, src_rate);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(stream) = self.stream.take() {
            drop(stream);
            info!("Audio capture stopped");
        }
    }

    pub fn is_running(&self) -> bool {
        self.stream.is_some()
    }
}

/// Convert interleaved multi-channel audio to mono by averaging channels.
fn to_mono(data: &[f32], channels: usize) -> Vec<f32> {
    if channels == 1 {
        return data.to_vec();
    }
    data.chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Simple resampler using linear interpolation.
/// Works well for integer ratios (e.g., 48000→16000 = 3:1).
fn simple_resample(data: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || data.is_empty() {
        return data.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (data.len() as f64 / ratio) as usize;
    if out_len == 0 {
        return Vec::new();
    }

    let mut output = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;

        let sample = if src_idx + 1 < data.len() {
            data[src_idx] * (1.0 - frac as f32) + data[src_idx + 1] * frac as f32
        } else if src_idx < data.len() {
            data[src_idx]
        } else {
            0.0
        };
        output.push(sample);
    }
    output
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self::new()
    }
}
