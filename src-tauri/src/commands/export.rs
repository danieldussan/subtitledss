use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEntry {
    pub id: i64,
    pub timestamp: String,
    pub language: String,
    pub original_text: String,
    pub translation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Srt,
    Vtt,
    Txt,
    Json,
    Ass,
}

fn format_timestamp_srt(timestamp: &str) -> String {
    // Convert RFC3339 to SRT format (HH:MM:SS,mmm)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        let time = dt.time();
        format!(
            "{:02}:{:02}:{:02},{:03}",
            time.hour(),
            time.minute(),
            time.second(),
            time.nanosecond() / 1_000_000
        )
    } else {
        "00:00:00,000".to_string()
    }
}

fn format_timestamp_vtt(timestamp: &str) -> String {
    // Convert RFC3339 to VTT format (HH:MM:SS.mmm)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        let time = dt.time();
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            time.hour(),
            time.minute(),
            time.second(),
            time.nanosecond() / 1_000_000
        )
    } else {
        "00:00:00.000".to_string()
    }
}

pub fn to_srt(entries: &[ExportEntry]) -> String {
    let mut output = String::new();
    for (i, entry) in entries.iter().enumerate() {
        let start = format_timestamp_srt(&entry.timestamp);
        // Add 3 seconds for end time (approximate)
        let end = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
            let end_dt = dt + chrono::Duration::seconds(3);
            format_timestamp_srt(&end_dt.to_rfc3339())
        } else {
            "00:00:03,000".to_string()
        };

        output.push_str(&format!("{}\n", i + 1));
        output.push_str(&format!("{} --> {}\n", start, end));
        output.push_str(&format!("{}\n\n", entry.original_text));
    }
    output
}

pub fn to_vtt(entries: &[ExportEntry]) -> String {
    let mut output = String::from("WEBVTT\n\n");
    for entry in entries {
        let start = format_timestamp_vtt(&entry.timestamp);
        let end = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
            let end_dt = dt + chrono::Duration::seconds(3);
            format_timestamp_vtt(&end_dt.to_rfc3339())
        } else {
            "00:00:03.000".to_string()
        };

        output.push_str(&format!("{} --> {}\n", start, end));
        output.push_str(&format!("{}\n\n", entry.original_text));
    }
    output
}

pub fn to_txt(entries: &[ExportEntry]) -> String {
    entries
        .iter()
        .map(|e| {
            if let Some(ref translation) = e.translation {
                format!("{}\n[Translation: {}]\n", e.original_text, translation)
            } else {
                e.original_text.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn to_json(entries: &[ExportEntry]) -> String {
    serde_json::to_string_pretty(entries).unwrap_or_else(|_| "[]".to_string())
}

fn format_timestamp_ass(seconds: f64) -> String {
    let total_cs = (seconds * 100.0) as u64;
    let h = total_cs / 3_600_00;
    let m = (total_cs % 3_600_00) / 6_000;
    let s = (total_cs % 6_000) / 100;
    let cs = total_cs % 100;
    format!("{}:{:02}:{:02}.{:02}", h, m, s, cs)
}

pub fn to_ass(entries: &[ExportEntry]) -> String {
    let header = "\
[Script Info]
Title: SubtitledSS Export
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: None
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H64000000,0,0,0,0,100,100,0,0,1,2,1,2,10,10,40,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
";

    let mut body = String::from(header);

    for entry in entries {
        let start_sec = parse_timestamp_to_seconds(&entry.timestamp);
        let end_sec = start_sec + 3.0; // approximate 3-second duration

        let start = format_timestamp_ass(start_sec);
        let end = format_timestamp_ass(end_sec);

        // Escape ASS special characters
        let text = entry.original_text.replace('\n', "\\N");

        body.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start, end, text
        ));
    }

    body
}

fn parse_timestamp_to_seconds(timestamp: &str) -> f64 {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.time().num_seconds_from_midnight() as f64 + dt.time().nanosecond() as f64 / 1_000_000_000.0
    } else {
        0.0
    }
}

pub fn export_entries(entries: &[ExportEntry], format: &ExportFormat, path: &PathBuf) -> anyhow::Result<()> {
    let content = match format {
        ExportFormat::Srt => to_srt(entries),
        ExportFormat::Vtt => to_vtt(entries),
        ExportFormat::Txt => to_txt(entries),
        ExportFormat::Json => to_json(entries),
        ExportFormat::Ass => to_ass(entries),
    };

    fs::write(path, content)?;
    Ok(())
}

#[tauri::command]
pub async fn export_history(
    entries: Vec<ExportEntry>,
    format: String,
    path: String,
) -> Result<(), String> {
    let export_format = match format.as_str() {
        "srt" => ExportFormat::Srt,
        "vtt" => ExportFormat::Vtt,
        "txt" => ExportFormat::Txt,
        "json" => ExportFormat::Json,
        "ass" => ExportFormat::Ass,
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    let path = PathBuf::from(path);
    export_entries(&entries, &export_format, &path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entries() -> Vec<ExportEntry> {
        vec![
            ExportEntry {
                id: 1,
                timestamp: "2024-01-15T10:30:00.000Z".to_string(),
                language: "en".to_string(),
                original_text: "Hello, this is a test.".to_string(),
                translation: None,
            },
            ExportEntry {
                id: 2,
                timestamp: "2024-01-15T10:30:05.000Z".to_string(),
                language: "en".to_string(),
                original_text: "The second segment.".to_string(),
                translation: Some("El segundo segmento.".to_string()),
            },
        ]
    }

    #[test]
    fn test_srt_format() {
        let entries = test_entries();
        let srt = to_srt(&entries);
        assert!(srt.contains("1\n"));
        assert!(srt.contains("10:30:00,000 --> 10:30:03,000"));
        assert!(srt.contains("Hello, this is a test."));
        assert!(srt.contains("2\n"));
        assert!(srt.contains("10:30:05,000 --> 10:30:08,000"));
    }

    #[test]
    fn test_vtt_format() {
        let entries = test_entries();
        let vtt = to_vtt(&entries);
        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("10:30:00.000 --> 10:30:03.000"));
        assert!(vtt.contains("Hello, this is a test."));
    }

    #[test]
    fn test_txt_format() {
        let entries = test_entries();
        let txt = to_txt(&entries);
        assert!(txt.contains("Hello, this is a test."));
        assert!(txt.contains("The second segment."));
        assert!(txt.contains("[Translation: El segundo segmento.]"));
    }

    #[test]
    fn test_json_format() {
        let entries = test_entries();
        let json = to_json(&entries);
        let parsed: Vec<ExportEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].original_text, "Hello, this is a test.");
    }

    #[test]
    fn test_ass_format() {
        let entries = test_entries();
        let ass = to_ass(&entries);
        assert!(ass.contains("[Script Info]"));
        assert!(ass.contains("ScriptType: v4.00+"));
        assert!(ass.contains("[V4+ Styles]"));
        assert!(ass.contains("[Events]"));
        assert!(ass.contains("Dialogue: 0,"));
        assert!(ass.contains("Hello, this is a test."));
        assert!(ass.contains("The second segment."));
    }

    #[test]
    fn test_ass_timestamp_format() {
        assert_eq!(format_timestamp_ass(0.0), "0:00:00.00");
        assert_eq!(format_timestamp_ass(65.5), "0:01:05.50");
        assert_eq!(format_timestamp_ass(3661.12), "1:01:01.12");
    }
}
