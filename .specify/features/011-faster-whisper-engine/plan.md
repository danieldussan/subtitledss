# Implementation Plan: Faster-Whisper (CTranslate2) Engine & Transcription Export Quality

**Feature Branch**: `011-faster-whisper-engine`
**Created**: 2026-08-09
**Spec**: `.specify/features/011-faster-whisper-engine/spec.md`
**Status**: Draft

## Summary

Add Faster-Whisper (CTranslate2, via `ct2rs`) as a third ASR backend, make video transcription robust on long audio (GPU + VAD preprocessing), merge raw segments into natural subtitle blocks/paragraphs, preserve real end timestamps and speaker labels in every export format, and surface diarization status to the user. The single `[NEEDS CLARIFICATION]` in the spec (FR-032, compute type UI exposure) is resolved in **Resolved Clarifications** below. Work is split into four independently testable milestones: **M1** export quality + segment merge (no new engine), **M2** ct2 engine + models + settings, **M3** video transcription integration (config-driven GPU/threads + VAD chunking + fusion), **M4** diarization status + speaker matching coverage.

## Resolved Clarifications

| Spec item | Resolution (decision) |
|-----------|----------------------|
| FR-032 / FR-031 `compute_type` ([NEEDS CLARIFICATION]) | `compute_type` becomes a **new string field in `WhisperConfig`** (and `TranscriptionParams`) with `#[serde(default = "float16")]`. Default value is `"float16"` on GPU builds; `"int8"` is also selectable. Exposed in the Whisper Settings UI as a small select with options **`float16` / `int8` / `int8_float16`**. Only the ctranslate2 engine consumes it. At engine load: `float16` + no GPU → warn + fall back to `int8`; invalid values → warn + `"float16"`. `beam_size` default `5` per FR-031. |

---

## Technical Context

**Language/Version**: Rust 2021 edition (crate `subtitledss` v1.3.0), TypeScript 5.x / React 19

**Primary Dependencies**:
- `ct2rs` **0.9** (new; `default-features=false, features=["whisper","all-tokenizers","ruy"]` — NON-optional, see AD-1)
- Existing: whisper-rs 0.16, sherpa-onnx 1.13.4, tauri 2, reqwest 0.12, hound 3.5, serde, speakrs 0.5

**Storage**: SQLite + FTS5 (existing `video_transcriptions` table, additive columns in M4), TOML config (existing)

**Testing**: `cargo test` (76 existing + new), `bun run typecheck`, `bun run lint` (oxlint), `bun run fmt:check` (oxfmt); reference port verifiable against `~/faster-whisper/transcribir.py`

**Target Platform**: Linux (Arch, Wayland, Hyprland); macOS/Windows continue to compile (ct2 on CPU; whisper/sherpa untouched)

**Project Type**: Desktop app (Tauri 2), Rust backend + React/TS frontend

**Performance Goals**: ct2 on NVIDIA GPU ≥4x faster than whisper.cpp batch path (SC-001); 1h40m single-pass without crash (SC-002); realtime overlay latency <500ms unaffected (FR-010); VAD skips silence to improve RTF (SC-003)

**Constraints**: 100% offline after model download; single binary; no cloud; `cargo build` must succeed both with default features (CPU ct2) and with `cuda`/`gpu` (SC-009); `export_history` payload backward compat (SC-007); graceful GPU→CPU fallback (FR-008)

**Scale/Scope**: 3 ASR engines (whisper/sherpa/ctranslate2), 6 ct2 models (~75MB–3GB downloads), 4 milestones, ~16 Rust/TS files

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Offline-First | ✅ PASS | ct2 models downloaded once via the existing ModelDownloader pattern with explicit consent; inference is 100% local via CTranslate2. Diarization stays local (speakrs). |
| II. Real-Time Performance | ✅ PASS | Real-time pipeline keeps the unified `AsrEngine::transcribe` API (FR-010); `ct2rs::Whisper::generate` takes `&self` so `Arc<Mutex<AsrEngine>>` holds it without new locking hazards. VAD chunking is batch/video-only and never applied to realtime chunks. |
| III. Modular Architecture | ✅ PASS | New `ct2/` and `transcription/` modules mirror the existing `sherpa/` boundary (engine.rs + models.rs); `AsrEngine` variant is additive (FR-003). No circular deps. |
| IV. Linux-Native | ✅ PASS | ct2rs supports Linux (openmp/dnnl/cuda); GPU is gated behind the existing `cuda`/`gpu` features; PipeWire/Arch workflow unchanged. |
| V. Test-First | ⚠️ ACTION NEEDED | Plan adds unit tests for: merge port (M1), export formats (M1), EngineKind parsing + WhisperOptions mapping + ct2 catalog (M2), config serde defaults (M2), VAD chunking math + speaker matcher (M3/M4). Engine *load* tests need a real model → manual/`#[ignore]`, matching the sherpa pattern. Frontend verified via typecheck. |

No constitution violations require justification → Complexity Tracking below is empty except the documented AD-1 trade-off (non-optional `ct2rs`), which is a build-cost risk, not a structural violation.

---

## Project Structure

### Documentation (this feature)

```text
.specify/features/011-faster-whisper-engine/
├── plan.md              # This file
├── spec.md              # Input feature spec
├── research.md          # Phase 0 output (future /speckit.plan run)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── contracts/           # Phase 1 output
    └── ct2-engine.md    # (future) Ctranslate2Engine contract
```

### Source Code (repository root)

```text
src-tauri/src/
├── transcription/           # NEW module — shared segment post-processing
│   ├── mod.rs               # NEW — module declarations
│   └── merge.rs             # NEW — fusionar_segmentos, crear_parrafos, es_final_de_frase
├── ct2/                     # NEW module — CTranslate2 backend
│   ├── mod.rs               # NEW — module declarations
│   ├── engine.rs            # NEW — Ctranslate2Engine wrapping ct2rs::Whisper
│   └── models.rs            # NEW — Ct2ModelInfo catalog + download/verify (mirrors sherpa/models.rs)
├── asr/
│   └── engine.rs            # MODIFY — EngineKind::Ctranslate2 + AsrEngine::Ctranslate2 arms
├── whisper/
│   └── params.rs            # MODIFY — TranscriptionParams += beam_size, compute_type
├── settings/
│   └── config.rs            # MODIFY — WhisperConfig += beam_size, compute_type (serde default)
├── commands/
│   ├── models.rs            # MODIFY — ct2 routing in list/download/delete/load/switch/list_downloaded
│   ├── export.rs            # MODIFY — ExportEntry += speaker/duration; to_srt/vtt/ass/txt/json
│   └── video_transcription.rs # MODIFY — config-driven params, transcribe_with_vad, merge, matcher, status
├── history/
│   └── db.rs                # MODIFY — additive diarization_status/speaker_count columns (M4)
└── lib.rs                   # MODIFY — mod transcription; mod ct2; per-kind model path resolution

src-tauri/
└── Cargo.toml               # MODIFY — ct2rs dep + cuda/gpu feature wiring

src/
├── hooks/
│   ├── useSettings.ts           # MODIFY — AppConfig.whisper += beam_size, compute_type
│   └── useVideoTranscription.ts # MODIFY — result/entry += diarization_status, speaker_count
├── components/
│   ├── Settings/
│   │   └── WhisperSettings.tsx  # MODIFY — compute_type select + beam size control
│   ├── ModelManager/
│   │   └── ModelList.tsx        # MODIFY (verify) — ct2 rows appear via catalog; badge styling
│   ├── VideoTranscription/
│   │   └── VideoTranscriptionPage.tsx  # MODIFY — diarization status badge
│   └── Onboarding/steps/
│       └── StepModelSelection.tsx      # MODIFY (verify) — ct2 models in onboarding list
```

**Structure Decision**: Single-project Tauri app. The CTranslate2 backend is a new `ct2/` module that mirrors the proven `sherpa/` layout (engine struct + model catalog), so the engine-switching machinery in `asr::engine` and `commands::models` extends by copy-pattern, not re-architecture. Shared export post-processing lives in a new `transcription/` module so both the video path and the export module can call `fusionar_segmentos`/`crear_parrafos` without circular deps.

---

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | — | — |

> AD-1 (non-optional `ct2rs`) is *not* a structural violation: it always compiles (CPU backend) and GPU is toggled by the existing `cuda`/`gpu` features. It does add Windows/macOS build time (CTranslate2 compiled from source) — mitigated in Risks.

---

## Architecture Decisions

| # | Decision |
|---|----------|
| **AD-1** | **ct2rs dependency strategy**: add `ct2rs` NON-optional with `default-features = false, features = ["whisper", "all-tokenizers", "ruy"]` so the `ct2` module always compiles and CPU consumers still build. The existing `cuda` feature extends to `ct2rs/cuda + ct2rs/cudnn + ct2rs/cuda-dynamic-loading`; `gpu = ["cuda"]` unchanged. This is simpler than a separate `ct2` cargo feature and guarantees FR-007/SC-009 both ways. |
| **AD-2** | **`compute_type` (resolved clarification)**: new string field on `WhisperConfig` + `TranscriptionParams`, serde default `"float16"`; UI select exposes `float16 / int8 / int8_float16`; consumed only by `Ctranslate2Engine`. Resolution: no-GPU + `float16` → warn + `int8`; invalid → warn + `"float16"`. `beam_size` default `5`. |
| **AD-3** | **Ctranslate2Engine API mirrors SherpaEngine**: `load_model(&mut self, &Path, gpu) -> anyhow::Result<()>`, `transcribe(&self, &[f32], &TranscriptionParams) -> Result<Vec<TranscriptionSegment>>`, `is_loaded()`, `model_path()`, `model_name()`. Wraps `ct2rs::Whisper` (`Whisper::new(model_dir, ct2rs::Config)`). `ct2rs::Whisper` is `Send + Sync` and `generate` takes `&self`, so it fits the app's `Arc<Mutex<AsrEngine>>` (a `Mutex` suffices). |
| **AD-4** | **WhisperOptions mapping**: `beam_size = params.beam_size` (min 1), `suppress_blank = true`, `repetition_penalty = 1.0`, everything else `..Default::default()`; `language = Some(params.language)` unless `"auto"`/`None` → `None` (ct2 auto-detects). Golden-device-fallback mirrors sherpa's provider fallback (FR-008). |
| **AD-5** | **Segment conversion**: `generate(samples, language, true, &opts)` → `Vec<ct2rs::Segment { id, text, start: f32, end: f32, words }>` mapped to `TranscriptionSegment { start: f64, end: f64, text }` (seconds, f32→f64). |
| **AD-6** | **Model layout**: `models/ct2/<name>/` directory (mirrors `models/sherpa/<name>/`), HF repos `Systran/faster-whisper-*`; required files `config.json, model.bin, tokenizer.json, preprocessor_config.json, vocabulary.json, generation_config.json`; a model counts as *downloaded* only when **all** files exist (FR-012). Rejects ggml `.bin` with a clear message (FR-005). |
| **AD-7** | **Per-kind model path resolution** in `lib.rs` setup and `commands::models`: `Whisper → models/ggml-<name>.bin`, `Sherpa → models/sherpa/<name>/`, `Ctranslate2 → models/ct2/<name>/` (a directory). Missing ct2 dir → info log + skip auto-load (edge case). |
| **AD-8** | **Export schema**: `ExportEntry` adds `speaker: Option<String>` and `duration: Option<f64>` (real end−start seconds), **both `#[serde(default)]`** so the existing `export_history` payload still deserializes; `end = start + duration`, falling back to `+3s` when `duration` is `None` (SC-007). |
| **AD-9** | **VAD chunking is batch/ct2-only**: new `transcribe_with_vad` fn slices the 16kHz mono samples into speech windows (energy-based `VadDetector` over 1s frames), pads 0.3s each side, transcribes each chunk via the engine, offsets timestamps into the full-file timeline, and emits per-chunk progress. whisper/sherpa keep their single-pass path but still get fusion (FR-019). |
| **AD-10** | **Speaker matcher + status**: replace midpoint-only lookup with nearest-turn assignment (overlap first, then nearest start, then most-recent-speaker propagation across short gaps, never inventing speakers); `diarization_status ∈ {disabled, succeeded, failed}` + `speaker_count` returned, persisted, and surfaced in the UI. |
| **AD-11** | **Milestone order M1→M4** chosen so each milestone is independently testable without the others (M1 needs no new engine; M2 needs no video-path changes; M3 builds on M1+M2; M4 is pure matching/status on top of M3). |

---

## Milestone M1 — Export Quality & Segment Merge (no new engine)

**Goal**: Port `fusionar_segmentos`/`crear_parrafos`/`es_final_de_frase` from `~/faster-whisper/transcribir.py`, apply merging in the video path, and fix SRT/VTT/ASS/TXT/JSON exports to carry real end timestamps and speaker labels. Delivers US3 + US4 with the existing whisper/sherpa engines and existing diarization.
**Dependencies**: none (foundation).
**Estimated scope**: 2 new files, 2 modified files.

### M1.1 — Segment post-processing module (`transcription/`)

**New files**: `src-tauri/src/transcription/mod.rs`, `src-tauri/src/transcription/merge.rs`

```rust
// merge.rs — direct port of transcribir.py:103-322
pub const MAX_DURATION: f64 = 15.0;          // fusionar_segmentos max block duration
pub const PAUSA_MIN: f64 = 1.5;              // notable-pause close threshold
pub const MAX_PARAGRAPH_DURATION: f64 = 45.0; // crear_parrafos paragraph target
pub const PARAGRAPH_HARD_DURATION: f64 = 90.0; // tope_duracion_hard = max * 2
pub const PARAGRAPH_HARD_CHARS: usize = 2000;  // tope_caracteres

pub fn es_final_de_frase(texto: &str) -> bool;   // trims quotes/parens, handles "...", ends in .!?;:
pub fn fusionar_segmentos(segs: &[TranscriptionSegment], max_duration: f64) -> Vec<MergedBlock>;
pub fn crear_parrafos(blocks: &[MergedBlock], max_paragraph_duration: f64) -> Vec<MergedBlock>;

pub struct MergedBlock { pub start: f64, pub end: f64, pub text: String }
```

- Port rules exactly: sentence-ending set `. ! ? ; :`; weak conjunctions `y, o, pero, que, porque, como, cuando, si, de, del, en, con, para, por, a`; close at `max_duration` only if the block already ends a sentence or a strong word (junction rule only applies *before* the cap, per spec edge case); close before the cap on a notable pause (≥1.5s) or a sentence end; block inherits first-segment start and last-segment end (no synthetic timestamps, FR-022).
- `crear_parrafos` rules: new paragraph on pause ≥1.5s; or duration ≥ `max/2` AND closed sentence; or hard caps (90s / 2000 chars).
- Register `pub mod transcription;` in `src-tauri/src/lib.rs`.

**Tests** (ported from the script's behavior):
- `test_es_final_de_frase` — `"."`, `"!"`, `"?"`, `";"`, `":"`, `"..."`, `'."'` (quotes stripped) → true; `"y"`, `"que"`, `"hola"` → false.
- `test_no_cut_after_weak_conjunction_before_cap` — segments ending in `"y"` merge on.
- `test_close_at_cap_even_on_weak_conjunction` — block closes at `max_duration`.
- `test_close_on_pause` — 1.5s+ gap closes the block.
- `test_block_real_boundaries` — block start/end = first/last segment.
- `test_paragraph_pause_and_hard_caps`.

**Verification**: `cargo check && cargo test transcription`

### M1.2 — Exports: real end timestamps + speakers (`commands/export.rs`)

**Modified file**: `src-tauri/src/commands/export.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEntry {
    pub id: i64,
    pub timestamp: String,          // start (RFC3339)
    pub language: String,
    pub original_text: String,
    pub translation: Option<String>,
    #[serde(default)]
    pub speaker: Option<String>,    // NEW
    #[serde(default)]
    pub duration: Option<f64>,      // NEW — real end−start seconds
}
```

- `to_srt` / `to_vtt`: `end = start + duration` when `duration` is `Some`; else current `+3s` fallback (backward compat, FR-025). Prefix text with `[Speaker X] ` when `speaker` is `Some` (FR-026).
- `to_ass`: `end = start + duration` (or `+3s`); when `speaker` present → write speaker into the Dialogue `Name` field **and** prefix `[Speaker X] ` in Text (covers rstyle-less players, FR-026).
- `to_txt`: when any entry carries `duration` (video-origin), build paragraphs via `crear_parrafos` (port) and prefix each paragraph's first-speaker with `[Speaker X] `; history-origin entries (no duration/speaker) keep the current flat join (per decision #8 — "keep to_txt simple for history").
- `to_json`: unchanged serialization — serde now emits `speaker` and `duration` automatically (FR-026 `start/end/text/translation/speaker` satisfied via `timestamp`/`duration`/`original_text`/`translation`/`speaker`).
- Update `test_entries()` fixture with `speaker: None, duration: None`; add:
  - `test_srt_end_uses_real_duration`, `test_srt_fallback_plus_3s`, `test_srt_speaker_prefix`, `test_vtt_speaker_prefix`, `test_ass_speaker_name_field`, `test_txt_paragraphs_with_speaker`, `test_txt_legacy_flat`, `test_json_includes_speaker`, `test_export_entry_serde_defaults` (JSON without `speaker`/`duration` parses → `None`/`None`, SC-007).

**Verification**: `cargo test export`

### M1.3 — Wire merge + real timestamps into the video path

**Modified file**: `src-tauri/src/commands/video_transcription.rs`

- In `transcribe_video`, after the engine returns raw segments (Step 4) and before speaker assignment (Step 5): run `fusionar_segmentos(&segments, MAX_DURATION)` → `Vec<MergedBlock>`; map each block to `DiarizedSegment { start: block.start, end: block.end, text: block.text, speaker: <existing midpoint matcher> }`. Existing whisper/sherpa path behavior stays except now merged (US3 acceptance).
- `export_video_transcription`: map each `DiarizedSegment` to `ExportEntry { id, timestamp: seconds_to_rfc3339(seg.start), language, original_text: seg.text, translation: None, speaker: seg.speaker.clone(), duration: Some(seg.end - seg.start) }` (FR-024).
- Keep the `merging` progress step (already exists).

**Verification**: `cargo check && cargo test`; manual: transcribe a video with whisper engine + diarization → SRT/VTT/ASS show real ends (`end ≥ start`, duration ≠ 3s) and `[Speaker X]` prefixes; TXT has paragraphs.

**Milestone M1 gate**:
```bash
cargo check && cargo test
bun run typecheck && bun run lint
```

---

## Milestone M2 — CTranslate2 Engine, Models & Settings

**Goal**: Third ASR backend via `ct2rs`, model catalog/download, engine-kind wiring, config fields, and UI controls. Delivers US1 (FR-001..FR-015) fully testable by switching engine to `ctranslate2`.
**Dependencies**: none beyond M0 baseline (M1 not strictly required).
**Estimated scope**: 3 new files, 6 modified files (Rust) + 2 frontend.

### M2.1 — Cargo dependency (`Cargo.toml`)

**Modified file**: `src-tauri/Cargo.toml`

```toml
[dependencies]
ct2rs = { version = "0.9", default-features = false, features = ["whisper", "all-tokenizers", "ruy"] }

[features]
cuda = ["whisper-rs/cuda", "ct2rs/cuda", "ct2rs/cudnn", "ct2rs/cuda-dynamic-loading"]
gpu = ["cuda"]
```

- AD-1: non-optional so the module always compiles; GPU support rides the existing `cuda`/`gpu` flags (FR-007). Default build = CPU ct2 (SC-009).

**Verification**: `cargo check` (default) and `cargo check --features gpu` (if a CUDA toolchain is present; otherwise note as CI job with CUDA runner).

### M2.2 — Ctranslate2Engine (`ct2/engine.rs`)

**New files**: `src-tauri/src/ct2/mod.rs`, `src-tauri/src/ct2/engine.rs`

```rust
use ct2rs::{Whisper, Config, WhisperOptions, Device};

pub struct Ctranslate2Engine {
    whisper: Option<Whisper>,        // ct2rs::Whisper is Send + Sync
    model_path: Option<PathBuf>,
    model_name: Option<String>,
    compute_type: String,            // resolved at load: "float16" | "int8" | "int8_float16"
    device: Device,                  // Cuda | Cpu (actual after fallback)
    threads: i32,                    // threads used at load
}

impl Ctranslate2Engine {
    pub fn new() -> Self;
    pub fn load_model(&mut self, model_path: &Path, gpu: bool) -> anyhow::Result<()>;
    pub fn transcribe(&self, audio: &[f32], params: &TranscriptionParams) -> anyhow::Result<Vec<TranscriptionSegment>>;
    pub fn is_loaded(&self) -> bool;
    pub fn model_path(&self) -> Option<PathBuf>;
    pub fn model_name(&self) -> Option<String>;
}
```

- `load_model`: require a **directory** containing `model.bin` + `config.json` + `tokenizer.json`; a bare ggml `.bin` path → `Err("CTranslate2 model directory required (e.g. Systran/faster-whisper-turbo); got <...>")` (FR-005, spec US1-3).
  - Resolve compute type over `&params.compute_type`-independent config… actually config comes at transcribe time; at load store `gpu` intent; device = `Cuda` if `gpu && cuda feature` else `Cpu`.
  - Effective compute type at load: from `WhisperConfig` value passed via `params.compute_type` at first transcribe; engine caches last value. If `!gpu && compute_type != "int8"*` → warn + use `"int8"`; invalid → warn + `"float16"`.
  - **GPU fallback** (FR-008): try `Whisper::new(dir, config)` with `Device::Cuda`; on error log `warn!("CUDA unavailable ({}), falling back to CPU int8", e)` and retry `Device::Cpu` (mirrors sherpa `build_recognizer` fallback).
  - `name = model_path.file_name()`; store `model_path`, `model_name`.
- `transcribe` (AD-3/AD-4/AD-5):
  ```rust
  let options = WhisperOptions {
      beam_size: params.beam_size.max(1) as usize,  // default 5
      suppress_blank: true,
      repetition_penalty: 1.0,
      ..WhisperOptions::default()
  };
  let language = match params.language.as_deref() { Some("auto") | None => None, Some(l) => Some(l) };
  let segs = self.whisper.generate(audio, language, true, &options)?;
  Ok(segs.into_iter().map(|s| TranscriptionSegment { start: s.start as f64, end: s.end as f64, text: s.text }).collect())
  ```
  - Threads: if `params.threads != self.threads` and a model is loaded, rebuild the `Whisper` with `Config` `cpu_threads = params.threads` (mirrors sherpa Canary rebuild — rare at runtime).
- `model_name`: dir basename (e.g. `turbo`).

**Tests**:
- `test_engine_new_not_loaded` — `is_loaded() == false`.
- `test_rejects_ggml_bin` — `load_model(Path::new("/tmp/ggml-base.bin"), false)` → Err (clear message).
- `test_compute_type_fallback_cpu` — resolution helper: `resolve_compute_type("float16", false) == "int8"`; `resolve_compute_type("float16", true) == "float16"`; `resolve_compute_type("bogus", true) == "float16"` (warn).
- `test_whisper_options_mapping` — pure fn `options_from_params(&params)` asserts beam_size/suppress_blank/repetition_penalty.
- `test_valid_language_none_for_auto` — helper `ct2_language(Some("auto")) == None`.

### M2.3 — Model catalog + download (`ct2/models.rs`)

**New file**: `src-tauri/src/ct2/models.rs` (mirror `sherpa/models.rs`)

```rust
pub struct Ct2ModelFile { pub filename: &'static str, pub url: String }
pub struct Ct2ModelInfo {
    pub name: &'static str,   // "tiny" | "base" | "small" | "medium" | "large-v3" | "turbo"
    pub label: &'static str,
    pub languages: &'static str,
    pub description: &'static str,
    pub size_mb: u64,
    pub files: Vec<Ct2ModelFile>,
}
pub const CT2_REQUIRED_FILES: &[&str] = &[
    "config.json", "model.bin", "tokenizer.json",
    "preprocessor_config.json", "vocabulary.json", "generation_config.json",
];
```

- Catalog: HF base `https://huggingface.co/Systran/faster-whisper-<name>/resolve/main`, 6 entries (tiny/base/small/medium/large-v3/turbo) covering the script's `--modelo` choices; sizes ~75MB/145MB/480MB/1.5GB/2.9GB/1.6GB (approximate, confirmed at download time).
- `model_dir(models_dir, name) = models_dir.join("ct2").join(name)` (AD-6).
- `is_ct2_model(name)`, `list_downloaded(models_dir)` (scans `models/ct2/` dirs), `download(name, models_dir)` (create dir; fetch each file via reqwest; skip existing), `is_downloaded(models_dir, name)` → **all** CT2_REQUIRED_FILES exist (FR-012), `delete(models_dir, name)` (remove dir).

**Tests**: `test_catalog_six_models` (contains turbo/large-v3), `test_catalog_files_have_hf_urls`, `test_model_dir_layout` (`models/ct2/turbo`), `test_is_downloaded_false_when_missing`, `test_list_downloaded_only_dirs` (mirror sherpa tests).

### M2.4 — EngineKind & AsrEngine wiring (`asr/engine.rs`)

**Modified file**: `src-tauri/src/asr/engine.rs`

```rust
pub enum EngineKind { Whisper, Sherpa, Ctranslate2 }        // additive (FR-003)
impl EngineKind {
    pub fn as_str(&self) -> &'static str { /* Whisper=>"whisper", Sherpa=>"sherpa", Ctranslate2=>"ctranslate2" */ }
    pub fn from_str(s: &str) -> Self {
        match s { "ct2" | "ctranslate2" => EngineKind::Ctranslate2, "sherpa" => EngineKind::Sherpa, _ => EngineKind::Whisper }
    }
}
pub enum AsrEngine { Whisper(WhisperEngine), Sherpa(SherpaEngine), Ctranslate2(Ctranslate2Engine) }
```

- Extend every existing arm: `new`, `kind`, `switch_kind`, `load_model`, `transcribe`, `is_loaded`, `model_path`, `model_name` (ct2 `model_name` = dir basename via engine field; matches Sherpa arm shape) (FR-002).
- Add `use crate::ct2::engine::Ctranslate2Engine;`.

**Tests**: `test_engine_kind_from_str` (`"ct2"`/`"ctranslate2"` → Ctranslate2; `"bogus"` → Whisper), `test_engine_kind_as_str`, `test_switch_kind_to_ct2` (switch returns true, kind()==Ctranslate2).

### M2.5 — Params & config fields

**Modified**: `src-tauri/src/whisper/params.rs` — add `pub beam_size: u32` and `pub compute_type: String` to `TranscriptionParams`; `Default`: `beam_size: 5, compute_type: "float16".to_string()`. All existing construction sites (`commands/transcription.rs`, `pipeline/transcriber.rs`, `video_transcription.rs`, `examples/benchmark.rs`) compile unchanged via `..Default::default()`; `video_transcription.rs` is revisited in M3.

**Modified**: `src-tauri/src/settings/config.rs` — `WhisperConfig` gains:

```rust
#[serde(default = "default_whisper_beam_size")]
pub beam_size: u32,        // 5
#[serde(default = "default_whisper_compute_type")]
pub compute_type: String,  // "float16"
```

with default fns + `Default` impl entries (FR-009). Existing TOML without the fields still parses (serde default) — update `test_default_whisper_config` and `test_deserialize_from_toml` to assert new defaults; add `test_whisper_legacy_toml_without_new_fields_parses`.

**Verification**: `cargo test settings whisper`

### M2.6 — Model commands routing (`commands/models.rs`)

**Modified file**: `src-tauri/src/commands/models.rs` — extend each command with a ct2 branch, byte-for-byte mirroring the sherpa branches:

- `list_available_models`: append ct2 catalog entries with `engine: "ctranslate2"`, `downloaded = ct2_models::is_downloaded(&models_dir, name)` (FR-011, FR-015).
- `download_model`: `if ct2_models::is_ct2_model(&model_name) { ct2_models::download(&model_name, &models_dir).await ... return }` before the whisper fallback.
- `delete_model`: ct2 branch removes `models/ct2/<name>` dir.
- `list_downloaded_models`: extend with `ct2_models::list_downloaded(models_dir)`.
- `load_model` / `switch_model`:
  ```rust
  let model_path = if sherpa.is_sherpa_model(&model_name) { sherpa_models::model_dir(...) }
      else if ct2_models::is_ct2_model(&model_name) { ct2_models::model_dir(&models_dir, &model_name) }  // dir
      else { models_dir.join(format!("ggml-{}.bin", model_name)) };
  let desired_kind = sherpa → Sherpa, ct2 → Ctranslate2, else Whisper;
  ```
  Validate download state per branch (FR-013). `switch_model` persists `cfg.whisper.engine = desired_kind.as_str()` as today (writes `"ctranslate2"`).

**Verification**: `cargo test`; manual `download_model({modelName:"turbo"})` → `models/ct2/turbo/` populated; `switch_model({modelName:"turbo"})` → `get_loaded_model` reports it.

### M2.7 — Startup auto-load per engine kind (`lib.rs`)

**Modified file**: `src-tauri/src/lib.rs`

- `pub mod ct2;` + `pub mod transcription;` in the module list.
- In `.setup()` (currently resolves only sherpa vs whisper):
  ```rust
  let model_path = match engine_kind {
      EngineKind::Sherpa => models_dir.join("sherpa").join(&model_name),
      EngineKind::Ctranslate2 => models_dir.join("ct2").join(&model_name),   // dir, FR-014
      EngineKind::Whisper => models_dir.join(format!("ggml-{}.bin", model_name)),
  };
  ```
  `if model_path.exists() … else info!("…skipping auto-load")` (existing). For ct2 with only ggml models on disk → info log + skip (spec edge case).
- Watch: `EngineKind::from_str` now yields `Ctranslate2` for `engine == "ctranslate2"`, so `switch_kind` inside `load_model` flow works; also `AsrEngine::new` at `run()` top handles the new variant.

**Verification**: `cargo check`; manual: set `whisper.engine = "ctranslate2"` + downloaded ct2 model → auto-loads at startup (US1-2).

### M2.8 — Frontend settings & model list

**Modified**: `src/hooks/useSettings.ts` — `whisper` interface gains `beam_size: number; compute_type: string;`.

**Modified**: `src/components/Settings/WhisperSettings.tsx`
- Add **Compute Type** select in the Performance section (always shown — decision #6): options `float16 / int8 / int8_float16`; hint text "Used by the faster-whisper (ctranslate2) engine".
- Add **Beam Size** control (number input / range 1–10, default 5) in the same section.
- Persist both in `handleSave` (`whisper: { ...config.whisper, model, language, threads, gpu, beam_size, compute_type }`).
- Engine remains model-driven via the model cards (the "engine select" the spec anticipated is the existing per-model badge/selection pattern — `list_available_models` now returns ct2 rows with `engine: "ctranslate2"`, so no structural UI change; extend the sherpa accent badge to also highlight `ctranslate2`).
- Note under the Language section for engine `"ctranslate2"`: language preference applies; `auto` = ct2 auto-detect.

**Modified (verify)**: `src/components/ModelManager/ModelList.tsx` — rows render from `list_available_models`, so ct2 entries appear automatically; add a "ctranslate2" badge style + optional section grouping. `StepModelSelection.tsx` consumes the same catalog — verify no engine-specific assumptions break.

**Verification**: `bun run typecheck && bun run lint`; manual: pick `ctranslate2` model in Settings → downloads via Model Manager → shows Active.

**Milestone M2 gate**:
```bash
cargo check && cargo test
bun run typecheck && bun run lint
```

---

## Milestone M3 — Video Transcription Integration (GPU / VAD / Fusion)

**Goal**: Make `transcribe_video` read engine/GPU/threads/beam/compute_type from `AppConfig`, add ct2-specific VAD chunking with timeline offsets + per-chunk progress, and keep whisper/sherpa single-pass. Delivers US2 (FR-016..FR-019).
**Dependencies**: M1 (merge in video path), M2 (engine + config fields).
**Estimated scope**: 1 modified file (plus helpers).

### M3.1 — Config-driven parameters

**Modified file**: `src-tauri/src/commands/video_transcription.rs`

- Add `config_state: State<'_, Arc<Mutex<AppConfig>>>` parameter to `transcribe_video` (mirrors the existing `ai_config_state` pattern).
- Build `TranscriptionParams` from config (FR-016) — remove the hardcoded `threads: 4, gpu: false`:
  ```rust
  let cfg = config_state.lock().unwrap();
  let params = TranscriptionParams {
      language: language.clone(),
      threads: cfg.whisper.threads,
      gpu: cfg.whisper.gpu,
      beam_size: cfg.whisper.beam_size,
      compute_type: cfg.whisper.compute_type.clone(),
      translate: false,
  };
  // decide single-pass vs VAD path by engine kind:
  let engine_kind = engine.kind();
  ```
- Lock scope: clone what's needed (threads/gpu/beam/compute_type/engine/vad_threshold) inside the lock, drop before heavy engine work (convention: no `MutexGuard` across `.await`).

### M3.2 — `transcribe_with_vad` (ct2-only)

New free fn in `video_transcription.rs` (kept batch-specific per FR-010):

```rust
/// Slices full audio into speech windows and transcribes each via the engine,
/// offsetting segment timestamps back into the full-file timeline.
/// ct2-only; whisper/sherpa keep single-pass (FR-017/FR-019).
fn transcribe_with_vad(
    engine: &AsrEngine,
    samples: &[f32],
    params: &TranscriptionParams,
    vad_threshold: f32,
    app_handle: &tauri::AppHandle,
    sample_rate: u32,   // 16000
) -> anyhow::Result<Vec<TranscriptionSegment>>
```

Algorithm (AD-9):
1. `VadDetector::new(vad_threshold.max(1e-4), 16000)` (1s frames) over the samples (reuse `src-tauri/src/vad/detector.rs`; it already exposes `detect()` + `is_speaking` state).
2. Build speech windows: contiguous runs where speech starts; a window ends after `max_silence_frames` of silence (~1.5s dead band → 1–2 silent frames at 1s frames — tune so chunks don't fragment: use `set_max_silence_frames(2)`).
3. Per window: `chunk = samples[win_start .. win_end]` padded `0.3s` (4800 samples) each side, clamped to `[0, len]`.
4. Skip windows whose padded slice has no speech (edge case: chunk with no speech contributes nothing).
5. `engine.transcribe(&chunk, params)` → offset each returned segment by `win_start_sec` (add to `start`/`end`).
6. Emit `app_handle.emit("video-transcription-progress", json!({"step": "transcribing", "progress": (i+1)/total, "message": format!("Transcribing chunk {}/{}", i+1, total)}))` (FR-018) so multi-hour jobs show progress.
7. Whole-file silence → empty `Vec`, no crash (edge case).

Pure helpers extracted for testing: `window_from_vad(windows, i) -> (usize, usize)` offset math and `offset_segments(segs, delta)`.

**Tests**:
- `test_offset_segments` — segments shifted by a window start; two-chunk offsets never reset to 0 (FR-017 acceptance).
- `test_padding_clamped` — window at file start/end keeps bounds.
- `test_window_builder_skips_empty` — all-silence input → zero windows.
- `test_transcribe_with_vad_multi_chunk_timeline` (mock-free; feed synthetic speech blocks + a `FakeEngine`-style local trait or a tiny stub AsrEngine harness if feasible — otherwise unit-test the pure window/offset helpers + integration via manual video).

### M3.3 — Dispatch + fusion in the video path

- In `transcribe_video` Step 4: `let segments = if engine_kind == EngineKind::Ctranslate2 { transcribe_with_vad(&engine, &audio_data, &params, cfg.audio.vad_threshold, &app_handle, 16000) } else { engine.transcribe(&audio_data, &params) };` — both arms return `Vec<TranscriptionSegment>`.
- After transcription, M1.3's `fusionar_segmentos` runs on the (possibly chunk-offset) segments — chunked path merges across chunk boundaries naturally since offsets are absolute (FR-017-4).
- Emit `"transcribing", progress: 1.0` after the final chunk (existing code).
- whisper/sherpa behavior identical to today apart from the M1 merge (FR-019-6).

**Milestone M3 gate**:
```bash
cargo check && cargo test
# Manual: 1h40m video, engine=ctranslate2, gpu=true → single pass, per-chunk progress, no crash;
#   timestamps stay in sync across silence gaps.
```

---

## Milestone M4 — Diarization Status & Speaker Matching Coverage

**Goal**: Better segment→speaker assignment (nearest-turn + gap propagation) and explicit diarization outcome surfaced in result, DB, and UI. Delivers US5 + US6 (FR-027..FR-030).
**Dependencies**: M3 (video path with merged blocks).
**Estimated scope**: 3 modified files (Rust) + 3 frontend (types/page), DB migration.

### M4.1 — Speaker matcher

**Modified file**: `src-tauri/src/commands/video_transcription.rs`

Extract a pure, testable function and use it in place of the inline midpoint lookup (which currently lives in the Step-5 `map`):

```rust
/// Assigns a speaker to each (start,end) block. Rules (FR-027/FR-028):
/// 1. Turn with greatest overlap wins; ties → temporally nearest turn start.
/// 2. No overlap → nearest turn by |start - turn.start| (deterministic; gap
///    between two turns of the SAME speaker → propagate that speaker).
/// 3. Gap between different speakers (≤ ~2.0s) with no speech → most recent speaker.
/// 4. Never invents speakers; single-speaker audio yields only its speaker.
fn assign_speakers(blocks: &[(f64, f64)], turns: &[SpeakerTurn]) -> Vec<Option<String>>;
```

- Overlap amount = `min(a.end, t.end) - max(a.start, t.start)`; require `> 0`.
- Same-speaker propagation: when scanning chronologically, if a block falls in a silent gap whose preceding *and* following turns belong to the same speaker and the gap ≤ 2.0s → that speaker; otherwise most-recent speaker (rule 3).
- If `turns` is empty → `None` for all (FR-028).
- Single-speaker (all turns same speaker) → every labeled block gets that speaker id only.

**Tests**:
- `test_overlap_wins_over_midpoint` — block straddling a turn boundary but overlapping one turn more → that turn.
- `test_nearest_turn_no_overlap` — block between turns → nearest.
- `test_same_speaker_gap_propagates` — 1.2s gap between Speaker_0 turns → block labeled Speaker_0.
- `test_mixed_gap_uses_most_recent` — gap between Speaker_0 and Speaker_1 → most recent deterministic.
- `test_single_speaker_only` — all labels ∈ {Speaker_0}.
- `test_no_turns_no_labels`.

### M4.2 — Diarization status (result + persistence)

**Modified**: `src-tauri/src/commands/video_transcription.rs`, `src-tauri/src/history/db.rs`

- `VideoTranscriptionResult` and `VideoTranscriptionEntry` gain:
  ```rust
  pub diarization_status: String,  // "disabled" | "succeeded" | "failed"
  pub speaker_count: u32,
  ```
- In `transcribe_video`: set status — `disabled` when `!enable_diarization`; `succeeded` when `Some(turns)` (speaker_count = distinct speaker ids, message "Detected N speaker(s)"; keep existing progress message); `failed` when `Err(e)` — keep the `warn!` log (FR-030) but surface "Speaker detection failed" in the final status + done event.
- Final `done` progress event message includes the status, e.g. `"Transcription complete — Speaker detection failed"` vs `"Transcription complete — 2 speakers detected"` vs `"Transcription complete"` (FR-029/FR-030 acceptance; US6).
- `history/db.rs`: additive migration in table init — `ALTER TABLE video_transcriptions ADD COLUMN diarization_status TEXT NOT NULL DEFAULT 'disabled'; ADD COLUMN speaker_count INTEGER NOT NULL DEFAULT 0;` (guarded by existence check; PRAGMA table_info) + write/read in `insert_video_transcription`, `get_video_transcription`, `get_video_transcriptions` (FR-029 "persisted so history views can show it").
- Backward compat: old rows read as `disabled`/0; old serialized results in tests updated.

**Tests**: `test_assigned_status_disabled` (fn that derives status from flag/result), `test_speaker_count_distinct`, DB round-trip test for the new columns (existing history test pattern).

### M4.3 — Frontend surfacing

**Modified**: `src/hooks/useVideoTranscription.ts` — `VideoTranscriptionResult` + `VideoTranscriptionEntry` add `diarization_status: string; speaker_count: number;` (old payloads: tolerate via `||` fallbacks in the page).

**Modified**: `src/components/VideoTranscription/VideoTranscriptionPage.tsx` — render a small status badge next to the video name:
- `succeeded` → `"{speaker_count} speaker(s) detected"` (accent/success styling),
- `disabled` → `"Speaker detection disabled"` (muted),
- `failed` → `"Speaker detection failed"` (danger styling + warning icon).
`ProgressIndicator` shows the final `done` message (already event-driven, no change required); `TranscriptionViewer` already colors by speaker (no change).

**Verification**: `bun run typecheck && bun run lint`; manual 3-run matrix (disabled / enabled-good / forced-fail) → distinct final notification per US6 acceptance.

**Milestone M4 gate**:
```bash
cargo check && cargo test
bun run typecheck && bun run lint
```

---

## Dependency Graph

```
M1 (export + merge)        ← foundation, no engine change
  M1.1 transcription/merge.rs   ── standalone
  M1.2 export.rs                ── standalone
  M1.3 video path merge         ── depends M1.1, M1.2

M2 (ct2 engine + models + settings)   ← can run in parallel with M1
  M2.1 Cargo.toml (ct2rs)       ── standalone
  M2.2 ct2/engine.rs            ── depends M2.1
  M2.3 ct2/models.rs            ── depends M2.1
  M2.4 asr/engine.rs            ── depends M2.2
  M2.5 params.rs + config.rs    ── standalone
  M2.6 commands/models.rs       ── depends M2.3, M2.4
  M2.7 lib.rs auto-load         ── depends M2.4, M2.6
  M2.8 frontend settings/list   ── depends M2.6 (backend commands)

M3 (video integration)       ← depends M1 (merge) + M2 (engine/config)
  M3.1 config-driven params    ── depends M2.5
  M3.2 transcribe_with_vad     ── depends M2.2, M3.1
  M3.3 dispatch + fusion       ── depends M3.1, M3.2, M1.3

M4 (status + matcher)        ← depends M3
  M4.1 assign_speakers         ── depends M3.3
  M4.2 status + db columns     ── depends M4.1 (uses result), history/db.rs
  M4.3 frontend badges         ── depends M4.2
```

### Parallel Opportunities

- **M1 ∥ M2** (fully independent feature halves: export/merge vs new engine).
- Within M2: 2.1/2.5 (Cargo + config) start immediately; 2.2/2.3 parallel after 2.1.
- Frontend (2.8) in parallel with backend 2.6–2.7.
- M3 and M4 sequential on the video path but M4.1 (matcher — pure fn) can start as soon as M1.3 lands.

---

## File Change Summary

| File | Milestone | Change Type | Description |
|------|-----------|-------------|-------------|
| `src-tauri/Cargo.toml` | M2 | MODIFY | Add `ct2rs` (non-optional, `whisper/all-tokenizers/ruy`); extend `cuda` with `ct2rs/cuda+cudnn+cuda-dynamic-loading` |
| `src-tauri/src/transcription/mod.rs` | M1 | NEW | Module declarations |
| `src-tauri/src/transcription/merge.rs` | M1 | NEW | `es_final_de_frase`, `fusionar_segmentos`, `crear_parrafos` (port) + tests |
| `src-tauri/src/commands/export.rs` | M1 | MODIFY | `ExportEntry += speaker/duration` (`#[serde(default)]`); `to_srt/vtt/ass` real ends + `[Speaker X]`; `to_txt` paragraphs; `to_json` speaker; tests |
| `src-tauri/src/commands/video_transcription.rs` | M1/M3/M4 | MODIFY | Apply fusion (M1); config-driven params + `transcribe_with_vad` + dispatch (M3); `assign_speakers` + status (M4); `export_video_transcription` duration/speaker (M1) |
| `src-tauri/src/ct2/mod.rs` | M2 | NEW | Module declarations |
| `src-tauri/src/ct2/engine.rs` | M2 | NEW | `Ctranslate2Engine` wrapping `ct2rs::Whisper`; WhisperOptions mapping; GPU→CPU fallback; tests |
| `src-tauri/src/ct2/models.rs` | M2 | NEW | `Ct2ModelInfo` catalog (`Systran/faster-whisper-*`), download/verify/list/delete; tests |
| `src-tauri/src/asr/engine.rs` | M2 | MODIFY | `EngineKind::Ctranslate2` (`ct2`/`ctranslate2` parse, `as_str "ctranslate2"`); `AsrEngine::Ctranslate2` arms; tests |
| `src-tauri/src/whisper/params.rs` | M2 | MODIFY | `TranscriptionParams += beam_size (5), compute_type ("float16")` + Default |
| `src-tauri/src/settings/config.rs` | M2 | MODIFY | `WhisperConfig += beam_size (serde default 5), compute_type (serde default "float16")` + tests |
| `src-tauri/src/commands/models.rs` | M2 | MODIFY | ct2 routing in `list_available_models`/`download_model`/`delete_model`/`list_downloaded_models`/`load_model`/`switch_model` |
| `src-tauri/src/lib.rs` | M1/M2 | MODIFY | `mod transcription; mod ct2;`; per-kind model path resolution in `.setup()` |
| `src-tauri/src/history/db.rs` | M4 | MODIFY | Additive `diarization_status`/`speaker_count` columns + read/write |
| `src/hooks/useSettings.ts` | M2 | MODIFY | `whisper` interface `+= beam_size, compute_type` |
| `src/hooks/useVideoTranscription.ts` | M4 | MODIFY | result/entry `+= diarization_status, speaker_count` |
| `src/components/Settings/WhisperSettings.tsx` | M2 | MODIFY | Compute Type select + Beam Size control |
| `src/components/ModelManager/ModelList.tsx` | M2 | MODIFY (verify) | ct2 rows (auto via catalog) + `ctranslate2` badge |
| `src/components/VideoTranscription/VideoTranscriptionPage.tsx` | M4 | MODIFY | Diarization status badge |
| `src/components/Onboarding/steps/StepModelSelection.tsx` | M2 | MODIFY (verify) | ct2 models in onboarding list |

---

## Success Criteria Mapping

| Criterion | Milestone | Requirements |
|-----------|-----------|--------------|
| SC-001: ct2 ≥4× faster on GPU (10-min clip) | M3 | FR-007, FR-016 |
| SC-002: 1h40m single-pass, no crash | M3 | FR-017, FR-019 |
| SC-003: VAD skips silence, RTF improves, sync kept | M3 | FR-017, FR-018 |
| SC-004: ≥90% blocks end on sentence-end or 15s cap | M1 | FR-020 |
| SC-005: exports carry real end timestamps (end ≥ start, ≠ +3s) | M1 | FR-023, FR-024, FR-025 |
| SC-006: ≥80% blocks labeled with diarization enabled | M4 | FR-027 |
| SC-007: `export_history` payload still deserializes | M1 | FR-023 (serde default test) |
| SC-008: explicit diarization status visible every run | M4 | FR-029, FR-030 |
| SC-009: cargo build default + `cuda`/`gpu`; tests + typecheck pass | M2/M4 | FR-007, FR-003 |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `ct2rs` needs a C/C++ toolchain (CTranslate2 compiled from source) — build time and CI impact; possible platform hiccup | MEDIUM | AD-1 keeps it non-optional but `default-features=false` (CPU-only builds fast). If the crate blocks a clean build on any platform, fall back to a `ct2` cargo feature gating `pub mod ct2;` so CPU/macOS consumers still build (spec Assumptions). CI: matrix job with and without `--features gpu`; `CUDA_TOOLKIT_ROOT_DIR` documented for GPU runners. |
| `ct2rs` API drift (e.g. `generate` vs `generate_segments`, `Segment` field types) | MEDIUM | Pin `version = "0.9"`; isolate all ct2 calls inside `ct2/engine.rs` so drift is one-file; unit-test the mapping fns without a real model. |
| CUDA absent at runtime with `gpu=true` (FR-008) | HIGH | Engine attempts `Device::Cuda`, falls back to CPU with a logged warning (mirrors sherpa). Never crash at startup; `compute_type` resolution uses int8 on CPU. |
| `ct2rs::Whisper` not `Send` on some build | MEDIUM | Verified Send+Sync upstream (generate takes `&self`); if a platform regresses, keep the engine behind `Arc<Mutex<…>>` (only Send required) — if even that fails, feature-gate the variant per above. |
| Ported heuristics diverge from `transcribir.py` | LOW | Port 1:1 with constant definitions matching the script; unit tests encode each rule (weak conjunction, cap, pause, paragraphs); diff against script output on the same audio in validation. |
| VAD chunking drifts timestamps or splits mid-word | MEDIUM | 0.3s padding + absolute offsetting + fusion across chunk boundaries; pure offset/helper tests; validate SRT sync on a file with ≥20% silence. |
| Diarization DB migration breaks old rows | LOW | Additive `ALTER TABLE` guarded by column-exists check; old rows read `disabled`/0; serde-default result fields. |
| Long single-chunk ct2 audio still OOMs (30-min speech burst) | MEDIUM | `transcribe_with_vad` windows capped by `max_silence_frames` splitting; if a single window is still huge, cap window length (e.g. 60s hard split) — tunable constant, no spec change. |
| Existing `test_entries`/config tests break on new fields | LOW | All new fields serde-defaulted; update fixtures in the same commit as each milestone. |

---

## Notes

- Reference script: `~/faster-whisper/transcribir.py` (lines 103–322 = `es_final_de_frase`/`fusionar_segmentos`/`crear_parrafos`; defaults `--beam-size 5`, `--max-segment-duration 15`, `vad_filter=True`, `compute_type float16` on CUDA) — the constants port directly; values are tunable without spec changes per Assumptions.
- Realtime overlay keeps `AsrEngine::transcribe` for ct2 (FR-010); VAD chunking lives only in the batch video path.
- `TranscriptionParams` construction sites to watch (all compile via Default, M3 replaces the video one): `commands/transcription.rs:32`, `pipeline/transcriber.rs:162`, `commands/video_transcription.rs:144`, `examples/benchmark.rs:88`.
- History exports built by the frontend (`ExportDialog`) don't set `speaker`/`duration` → legacy flat `to_txt` + `+3s` end fallback remain the behavior for realtime history (unchanged UX, SC-007 verified by test).