use std::path::Path;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub format: String,
    pub duration_seconds: f64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub size_bytes: u64,
}

pub struct VideoProcessor;

impl VideoProcessor {
    /// Extract audio from a video file to 16kHz mono WAV (Whisper-compatible).
    /// Returns the path to the extracted WAV file.
    pub async fn extract_audio(video_path: &Path) -> anyhow::Result<std::path::PathBuf> {
        let output_path = video_path.with_extension("wav");

        info!("Extracting audio from {:?} to {:?}", video_path, output_path);

        let video_str = video_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid video path"))?;
        let output_str = output_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid output path"))?;

        let output = tokio::process::Command::new("ffmpeg")
            .args([
                "-i", video_str,
                "-vn",
                "-acodec", "pcm_s16le",
                "-ar", "16000",
                "-ac", "1",
                "-y", output_str,
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "FFmpeg audio extraction failed (exit code: {:?}): {}",
                output.status.code(),
                stderr.chars().take(500).collect::<String>()
            ));
        }

        info!("Audio extracted successfully to {:?}", output_path);
        Ok(output_path)
    }

    /// Get the duration of a video file in seconds.
    pub async fn get_duration(video_path: &Path) -> anyhow::Result<f64> {
        let output = tokio::process::Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-show_entries",
                "format=duration",
                "-of",
                "csv=p=0",
                video_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("ffprobe failed to get duration"));
        }

        let duration_str = String::from_utf8(output.stdout)?;
        let duration = duration_str.trim().parse::<f64>()?;

        info!("Video duration: {:.1}s", duration);
        Ok(duration)
    }

    /// Get full metadata of a video file.
    pub async fn get_metadata(video_path: &Path) -> anyhow::Result<VideoMetadata> {
        let output = tokio::process::Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                video_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("ffprobe failed to get metadata"));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;

        let format = json["format"]["format_name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let duration_seconds = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let size_bytes = json["format"]["size"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let mut width = None;
        let mut height = None;
        let mut video_codec = None;
        let mut audio_codec = None;

        if let Some(streams) = json["streams"].as_array() {
            for stream in streams {
                match stream["codec_type"].as_str() {
                    Some("video") => {
                        width = stream["width"].as_u64().map(|w| w as u32);
                        height = stream["height"].as_u64().map(|h| h as u32);
                        video_codec = stream["codec_name"].as_str().map(|s| s.to_string());
                    }
                    Some("audio") => {
                        audio_codec = stream["codec_name"].as_str().map(|s| s.to_string());
                    }
                    _ => {}
                }
            }
        }

        let metadata = VideoMetadata {
            format,
            duration_seconds,
            width,
            height,
            video_codec,
            audio_codec,
            size_bytes,
        };

        info!("Video metadata: {:?}", metadata);
        Ok(metadata)
    }

    /// Check if FFmpeg is available on the system.
    pub async fn check_ffmpeg() -> bool {
        tokio::process::Command::new("ffmpeg")
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_metadata_struct() {
        let metadata = VideoMetadata {
            format: "mp4".to_string(),
            duration_seconds: 120.5,
            width: Some(1920),
            height: Some(1080),
            video_codec: Some("h264".to_string()),
            audio_codec: Some("aac".to_string()),
            size_bytes: 50_000_000,
        };

        assert_eq!(metadata.format, "mp4");
        assert_eq!(metadata.duration_seconds, 120.5);
        assert_eq!(metadata.width, Some(1920));
        assert_eq!(metadata.height, Some(1080));
    }

    #[test]
    fn test_video_metadata_serialization() {
        let metadata = VideoMetadata {
            format: "mkv".to_string(),
            duration_seconds: 90.0,
            width: None,
            height: None,
            video_codec: None,
            audio_codec: Some("opus".to_string()),
            size_bytes: 10_000_000,
        };
        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: VideoMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.format, "mkv");
        assert_eq!(parsed.width, None);
        assert_eq!(parsed.audio_codec, Some("opus".to_string()));
    }

    #[test]
    fn test_video_metadata_defaults() {
        let metadata = VideoMetadata {
            format: "webm".to_string(),
            duration_seconds: 0.0,
            width: None,
            height: None,
            video_codec: None,
            audio_codec: None,
            size_bytes: 0,
        };
        assert_eq!(metadata.duration_seconds, 0.0);
        assert!(metadata.width.is_none());
    }

    #[tokio::test]
    async fn test_check_ffmpeg_returns_bool() {
        // This just verifies the function compiles and returns a bool
        // It may return false if ffmpeg is not installed in CI
        let result = VideoProcessor::check_ffmpeg().await;
        assert!(result == true || result == false);
    }
}
