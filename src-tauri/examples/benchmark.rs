//! RTF / WER benchmark comparing whisper.cpp against sherpa-onnx models.
//!
//! Usage:
//!   cargo run --release --example benchmark -- <audio.wav> <model> [<model> ...] [--lang es] [--ref ref.txt]
//!
//! Each <model> is either:
//!   - a path to a whisper ggml-*.bin file  (e.g. ~/.local/share/subtitledss/models/ggml-base.bin)
//!   - a sherpa-onnx model directory        (e.g. ~/.local/share/subtitledss/models/sherpa/parakeet-tdt-0.6b-v3-int8)
//!
//! Output per model: full-file RTF, simulated 1.5s-chunk RTF (what the live
//! pipeline does), and WER against an optional reference transcript.

use std::path::PathBuf;
use std::time::Instant;

use subtitledss_lib::sherpa::SherpaEngine;
use subtitledss_lib::whisper::WhisperEngine;
use subtitledss_lib::whisper::params::TranscriptionParams;

const SAMPLE_RATE: usize = 16000;
/// 1.5 s at 16 kHz — mirrors CHUNK_SAMPLES in the live pipeline.
const CHUNK_SAMPLES: usize = 24000;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: benchmark <audio.wav> <model> [<model> ...] [--lang <code>] [--ref <ref.txt>]"
        );
        eprintln!("  <model> = whisper ggml-*.bin OR a sherpa-onnx model directory");
        std::process::exit(1);
    }

    let wav_path = &args[1];
    let mut models: Vec<String> = Vec::new();
    let mut lang = "auto".to_string();
    let mut ref_path: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--lang" => {
                i += 1;
                if i < args.len() {
                    lang = args[i].clone();
                }
            }
            "--ref" => {
                i += 1;
                if i < args.len() {
                    ref_path = Some(args[i].clone());
                }
            }
            a => models.push(a.to_string()),
        }
        i += 1;
    }

    let samples = read_wav_to_f32(PathBuf::from(wav_path)).expect("failed to read WAV");
    let duration_s = samples.len() as f64 / SAMPLE_RATE as f64;
    let reference = ref_path.map(|p| {
        std::fs::read_to_string(&p)
            .unwrap_or_else(|e| panic!("failed to read ref '{}': {}", p, e))
            .trim()
            .to_string()
    });

    println!("=== ASR benchmark ===");
    println!(
        "Audio: {} ({:.1}s, {} samples)",
        wav_path,
        duration_s,
        samples.len()
    );
    println!("Language: {} | RTF = decode_time / audio_duration", lang);
    println!("Models: {}\n", models.join(", "));

    for m in &models {
        let path = PathBuf::from(m);
        let is_whisper = !path.is_dir();
        let label = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| m.clone());
        let backend = if is_whisper { "whisper" } else { "sherpa" };
        println!("--- {} ({}) ---", label, backend);

        let params = TranscriptionParams {
            language: Some(lang.clone()),
            threads: 4,
            gpu: false,
            translate: false,
            ..Default::default()
        };

        let (full_text, full_rtf, chunk_rtf) = if is_whisper {
            let mut eng = WhisperEngine::new();
            eng.load_model(&path).expect("load whisper model");
            run_whisper(&eng, &samples, &params)
        } else {
            let mut eng = SherpaEngine::new();
            eng.load_model(&path).expect("load sherpa model");
            run_sherpa(&mut eng, &samples, &params)
        };

        println!(
            "  full-file : RTF {:.3}  ({:.1}x realtime)",
            full_rtf / duration_s,
            duration_s / full_rtf
        );
        println!(
            "  1.5s chunks: RTF {:.3}  ({:.1}x realtime)",
            chunk_rtf / duration_s,
            duration_s / chunk_rtf
        );
        if let Some(ref_text) = &reference {
            let wer = word_error_rate(ref_text, &full_text);
            println!("  WER: {:.2}%", wer * 100.0);
            println!("  ref: {}", ref_text);
        }
        println!("  text: {}", full_text);
        println!();
    }
}

fn run_whisper(
    eng: &WhisperEngine,
    samples: &[f32],
    params: &TranscriptionParams,
) -> (String, f64, f64) {
    let start = Instant::now();
    let full_segs = eng.transcribe(samples, params).expect("full transcribe");
    let full_rtf = start.elapsed().as_secs_f64();

    let chunk_start = Instant::now();
    let mut chunk_text = String::new();
    for chunk in samples.chunks(CHUNK_SAMPLES) {
        if let Ok(segs) = eng.transcribe(chunk, params) {
            for s in segs {
                chunk_text.push_str(s.text.trim());
                chunk_text.push(' ');
            }
        }
    }
    let chunk_rtf = chunk_start.elapsed().as_secs_f64();

    let full_text = join_segments(&full_segs);
    if chunk_text.trim().len() > full_text.len() {
        // Chunking can produce repeated/spurious text; keep the fuller variant.
        (chunk_text.trim().to_string(), full_rtf, chunk_rtf)
    } else {
        (full_text, full_rtf, chunk_rtf)
    }
}

fn run_sherpa(
    eng: &mut SherpaEngine,
    samples: &[f32],
    params: &TranscriptionParams,
) -> (String, f64, f64) {
    let start = Instant::now();
    let full_segs = eng.transcribe(samples, params).expect("full transcribe");
    let full_rtf = start.elapsed().as_secs_f64();

    let chunk_start = Instant::now();
    let mut chunk_text = String::new();
    for chunk in samples.chunks(CHUNK_SAMPLES) {
        if let Ok(segs) = eng.transcribe(chunk, params) {
            for s in segs {
                chunk_text.push_str(s.text.trim());
                chunk_text.push(' ');
            }
        }
    }
    let chunk_rtf = chunk_start.elapsed().as_secs_f64();

    let full_text = join_segments(&full_segs);
    (full_text, full_rtf, chunk_rtf)
}

fn join_segments(segs: &[subtitledss_lib::asr::TranscriptionSegment]) -> String {
    segs.iter()
        .map(|s| s.text.trim())
        .filter(|s| !s.is_empty() && *s != "[BLANK_AUDIO]")
        .collect::<Vec<_>>()
        .join(" ")
}

fn read_wav_to_f32(path: PathBuf) -> anyhow::Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)?;
    let samples: Result<Vec<f32>, _> = reader
        .samples::<i16>()
        .map(|s| s.map(|s| s as f32 / 32768.0))
        .collect();
    Ok(samples?)
}

fn word_error_rate(reference: &str, hypothesis: &str) -> f64 {
    let ref_words: Vec<&str> = reference.split_whitespace().collect();
    let hyp_words: Vec<&str> = hypothesis.split_whitespace().collect();
    if ref_words.is_empty() {
        return if hyp_words.is_empty() { 0.0 } else { 1.0 };
    }
    let d = levenshtein(&ref_words, &hyp_words);
    d as f64 / ref_words.len() as f64
}

fn levenshtein(a: &[&str], b: &[&str]) -> usize {
    let (n, m) = (a.len(), b.len());
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}
