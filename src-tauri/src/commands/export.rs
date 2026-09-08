use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::transcription::merge::{crear_parrafos, MergeSegment};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEntry {
    pub id: i64,
    #[serde(default)]
    pub timestamp: String,
    pub language: String,
    pub original_text: String,
    pub translation: Option<String>,
    #[serde(default)]
    pub speaker: Option<String>,
    #[serde(default)]
    pub duration: Option<f64>,
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

/// Duración efectiva de una entrada: la real si existe, si no 3 segundos.
fn entry_duration(duration: Option<f64>) -> f64 {
    duration.unwrap_or(3.0)
}

/// Etiqueta legible de un hablante: `SPEAKER_00` → `Speaker 0`.
fn speaker_label(speaker: &str) -> String {
    if let Some(idx) = speaker.strip_prefix("SPEAKER_") {
        return match idx.parse::<u32>() {
            Ok(n) => format!("Speaker {}", n),
            Err(_) => format!("Speaker {}", idx),
        };
    }
    speaker.to_string()
}

/// Prefijo `[Speaker X] ` para insertar en el texto cuando hay hablante.
fn speaker_prefix(speaker: &Option<String>) -> String {
    speaker
        .as_ref()
        .map(|s| format!("[{}] ", speaker_label(s)))
        .unwrap_or_default()
}

/// Fin de la entrada como RFC3339 (start + duration, o fallback +3s).
fn end_timestamp(entry: &ExportEntry) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
        let duration_ms = (entry_duration(entry.duration) * 1000.0) as i64;
        let end_dt = dt + chrono::Duration::milliseconds(duration_ms);
        end_dt.to_rfc3339()
    } else {
        let duration_s = entry_duration(entry.duration);
        format!(
            "1970-01-01T00:00:{:02}.{:03}Z",
            (duration_s as u64) % 60,
            ((duration_s.fract() * 1000.0) as u32)
        )
    }
}

pub fn to_srt(entries: &[ExportEntry]) -> String {
    let mut output = String::new();
    for (i, entry) in entries.iter().enumerate() {
        let start = format_timestamp_srt(&entry.timestamp);
        let end = format_timestamp_srt(&end_timestamp(entry));
        let prefix = speaker_prefix(&entry.speaker);

        output.push_str(&format!("{}\n", i + 1));
        output.push_str(&format!("{} --> {}\n", start, end));
        output.push_str(&format!("{}{}\n\n", prefix, entry.original_text));
    }
    output
}

pub fn to_vtt(entries: &[ExportEntry]) -> String {
    let mut output = String::from("WEBVTT\n\n");
    for entry in entries {
        let start = format_timestamp_vtt(&entry.timestamp);
        let end = format_timestamp_vtt(&end_timestamp(entry));
        let prefix = speaker_prefix(&entry.speaker);

        output.push_str(&format!("{} --> {}\n", start, end));
        output.push_str(&format!("{}{}\n\n", prefix, entry.original_text));
    }
    output
}

pub fn to_txt(entries: &[ExportEntry]) -> String {
    let has_durations = entries.iter().any(|e| e.duration.is_some());
    let has_translations = entries.iter().any(|e| e.translation.is_some());

    // Modo párrafos: solo para transcripciones de vídeo (con duraciones reales)
    // y sin traducciones por segmento.
    if has_durations && !has_translations {
        let blocks: Vec<MergeSegment> = entries
            .iter()
            .map(|e| {
                let start = parse_timestamp_to_seconds(&e.timestamp);
                MergeSegment {
                    start,
                    end: start + entry_duration(e.duration),
                    text: format!("{}{}", speaker_prefix(&e.speaker), e.original_text.trim()),
                }
            })
            .collect();

        return crear_parrafos(&blocks, 45.0)
            .iter()
            .map(|p| p.text.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
    }

    entries
        .iter()
        .map(|e| {
            let prefix = speaker_prefix(&e.speaker);
            if let Some(ref translation) = e.translation {
                format!(
                    "{}{}\n[Translation: {}]",
                    prefix, e.original_text, translation
                )
            } else {
                format!("{}{}", prefix, e.original_text)
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
        let end_sec = start_sec + entry_duration(entry.duration);

        let start = format_timestamp_ass(start_sec);
        let end = format_timestamp_ass(end_sec);

        // Escape ASS special characters
        let text = entry.original_text.replace('\n', "\\N");

        let name = match &entry.speaker {
            Some(s) => speaker_label(s),
            None => String::new(),
        };

        body.push_str(&format!(
            "Dialogue: 0,{},{},Default,{},0,0,0,,{}\n",
            start, end, name, text
        ));
    }

    body
}

fn parse_timestamp_to_seconds(timestamp: &str) -> f64 {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.time().num_seconds_from_midnight() as f64
            + dt.time().nanosecond() as f64 / 1_000_000_000.0
    } else {
        0.0
    }
}

pub fn export_entries(
    entries: &[ExportEntry],
    format: &ExportFormat,
    path: &PathBuf,
) -> anyhow::Result<()> {
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
                speaker: None,
                duration: None,
            },
            ExportEntry {
                id: 2,
                timestamp: "2024-01-15T10:30:05.000Z".to_string(),
                language: "en".to_string(),
                original_text: "The second segment.".to_string(),
                translation: Some("El segundo segmento.".to_string()),
                speaker: None,
                duration: None,
            },
        ]
    }

    fn video_entries() -> Vec<ExportEntry> {
        vec![
            ExportEntry {
                id: 1,
                timestamp: "1970-01-01T00:00:05.000Z".to_string(),
                language: "es".to_string(),
                original_text: "Primera frase completa.".to_string(),
                translation: None,
                speaker: Some("SPEAKER_00".to_string()),
                duration: Some(2.5),
            },
            ExportEntry {
                id: 2,
                timestamp: "1970-01-01T00:00:09.000Z".to_string(),
                language: "es".to_string(),
                original_text: "Segunda frase.".to_string(),
                translation: None,
                speaker: Some("SPEAKER_01".to_string()),
                duration: Some(3.0),
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
    fn test_video_txt_paragraphs() {
        let entries = video_entries();
        let txt = to_txt(&entries);
        // Cada bloque tiene el prefijo de hablante dentro del párrafo.
        assert!(txt.contains("[Speaker 0] Primera frase completa."));
        assert!(txt.contains("[Speaker 1] Segunda frase."));
    }

    #[test]
    fn test_txt_speaker_flat_mode() {
        let mut entries = test_entries();
        // Speaker sin duración: modo plano conservado con prefijo.
        entries[1].speaker = Some("SPEAKER_00".to_string());
        let txt = to_txt(&entries);
        assert!(txt.contains("The second segment."));
        assert!(txt.contains("[Speaker 0] The second segment."));
    }

    #[test]
    fn test_srt_uses_real_duration() {
        let entries = video_entries();
        let srt = to_srt(&entries);
        assert!(srt.contains("00:00:05,000 --> 00:00:07,500"));
        assert!(srt.contains("[Speaker 0] Primera frase completa."));
        assert!(srt.contains("00:00:09,000 --> 00:00:12,000"));
        assert!(srt.contains("[Speaker 1] Segunda frase."));
    }

    #[test]
    fn test_srt_falls_back_when_no_duration() {
        let entries = test_entries();
        let srt = to_srt(&entries);
        assert!(srt.contains("10:30:00,000 --> 10:30:03,000"));
    }

    #[test]
    fn test_vtt_uses_real_duration() {
        let entries = video_entries();
        let vtt = to_vtt(&entries);
        assert!(vtt.contains("00:00:05.000 --> 00:00:07.500"));
        assert!(vtt.contains("[Speaker 0] Primera frase completa."));
    }

    #[test]
    fn test_ass_uses_name_field_and_duration() {
        let entries = video_entries();
        let ass = to_ass(&entries);
        assert!(ass.contains(
            "Dialogue: 0,0:00:05.00,0:00:07.50,Default,Speaker 0,0,0,0,,Primera frase completa."
        ));
        assert!(ass
            .contains("Dialogue: 0,0:00:09.00,0:00:12.00,Default,Speaker 1,0,0,0,,Segunda frase."));
    }

    #[test]
    fn test_json_includes_speaker_and_duration() {
        let entries = video_entries();
        let json = to_json(&entries);
        assert!(json.contains("\"speaker\": \"SPEAKER_00\""));
        assert!(json.contains("\"duration\": 2.5"));
    }

    #[test]
    fn test_json_missing_fields_default() {
        // JSON antiguo sin speaker/duration sigue parseándose.
        let json = r#"[{"id":1,"timestamp":"2024-01-15T10:30:00.000Z","language":"en","original_text":"hi","translation":null}]"#;
        let parsed: Vec<ExportEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].speaker, None);
        assert_eq!(parsed[0].duration, None);
    }

    #[test]
    fn test_speaker_label() {
        assert_eq!(speaker_label("SPEAKER_00"), "Speaker 0");
        assert_eq!(speaker_label("SPEAKER_01"), "Speaker 1");
        assert_eq!(speaker_label("persona"), "persona");
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
