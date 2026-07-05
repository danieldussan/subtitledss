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

pub fn export_entries(entries: &[ExportEntry], format: &ExportFormat, path: &PathBuf) -> anyhow::Result<()> {
    let content = match format {
        ExportFormat::Srt => to_srt(entries),
        ExportFormat::Vtt => to_vtt(entries),
        ExportFormat::Txt => to_txt(entries),
        ExportFormat::Json => to_json(entries),
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
}
