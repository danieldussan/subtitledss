# Feature Specification: Faster-Whisper (CTranslate2) Engine & Transcription Export Quality

**Feature Branch**: `011-faster-whisper-engine`

**Created**: 2026-08-09

**Status**: Draft

**Input**: User description: "Add Faster-Whisper (CTranslate2 / ct2rs) as a third ASR engine backend, make video transcription robust on long audio with GPU + VAD preprocessing, merge raw segments into natural subtitle blocks and paragraphs, preserve real segment end timestamps in exports, include speaker labels in all export formats, and surface diarization status to the user."

## User Scenarios & Testing

### User Story 1 - Faster-Whisper (CTranslate2) Engine Backend (Priority: P1)

As a user transcribing long videos, I want to select a faster-whisper (CTranslate2) model in Settings, download it from the Model Manager, and use it for batch transcription, so that transcription is up to 4x faster on my NVIDIA GPU and can handle multi-hour audio that currently crashes the app.

**Why this priority**: This is the core of the feature. The current whisper.cpp batch path is slow on CPU, uses more RAM, and crashed on a 1h40m file. faster-whisper (CTranslate2) with float16/int8 is the direct replacement the user already validated with a working Python script. Without the engine, none of the downstream quality work matters.

**Independent Test**: Can be fully tested by switching the engine kind to `ctranslate2` in Settings, downloading Systran/faster-whisper-turbo from the Model Manager, loading it, and verifying a video transcription completes with correct timestamps. Delivers a working third ASR backend without touching any other feature.

**Acceptance Scenarios**:

1. **Given** a CTranslate2-format model is downloaded under `models/ct2/`, **When** the user selects it in the Model Manager and clicks Load, **Then** the app loads it via `ct2rs::Whisper::new` and reports the model as Active.
2. **Given** the engine setting is `ctranslate2`, **When** the app starts and the ct2 model exists, **Then** the model auto-loads in lib.rs setup without manual action.
3. **Given** a model from ggerganov/whisper.cpp (ggml .bin), **When** the user tries to load it with engine `ctranslate2`, **Then** the app rejects it with a clear message that a CTranslate2-format model directory (e.g. Systran/faster-whisper-turbo) is required.
4. **Given** GPU acceleration is enabled in Settings with a CUDA build, **When** the ct2 engine transcribes, **Then** inference uses the GPU (float16) and reports the device used.
5. **Given** the app is built without the GPU feature, **When** GPU acceleration is requested, **Then** the engine gracefully falls back to CPU (int8) or reports a clear error, and the app does not crash.
6. **Given** the real-time overlay path, **When** the engine is set to `ctranslate2`, **Then** `AsrEngine.transcribe` continues to work for live segments (same unified API).

---

### User Story 2 - Robust Long-Audio Transcription with GPU and VAD Preprocessing (Priority: P1)

As a user transcribing a 1h40m video, I want the transcription to complete in one pass without crashing or degrading, using the configured GPU/threads/engine settings, and skipping silence so I get accurate speech-only segments.

**Why this priority**: A 1h40m file crashed the current app — this is the bug that motivated the whole feature. Long-audio robustness and GPU utilization are the primary user-visible wins and directly justify the engine change.

**Independent Test**: Can be tested by selecting a multi-hour video, enabling GPU, and verifying a single transcription run completes end-to-end with segment timestamps aligned to the full file timeline, without manual chunking.

**Acceptance Scenarios**:

1. **Given** a 1h40m video and the ct2 engine, **When** the user clicks Transcribe, **Then** the job completes in a single run without crashing or running out of memory.
2. **Given** GPU and threads configured in Settings, **When** `transcribe_video` runs, **Then** it reads `gpu`, `threads`, `engine`, `beam_size` and `compute_type` from `AppConfig` instead of hardcoding `gpu=false, threads=4`.
3. **Given** the ct2 engine and an audio file with silence gaps, **When** transcription runs, **Then** VAD-based preprocessing slices the audio into speech chunks, silence is skipped, and each chunk is transcribed with `generate_segments`.
4. **Given** VAD chunking divides the audio, **When** transcription completes, **Then** every segment's start/end timestamps are offset back into the full-file timeline (no chunks restart at 0).
5. **Given** long audio is being transcribed, **When** the user watches the Video page, **Then** progress events are emitted per chunk so the UI shows meaningful progress.
6. **Given** the whisper or sherpa engines, **When** `transcribe_video` runs, **Then** behavior stays identical to today (VAD chunking is ct2-specific).

---

### User Story 3 - Natural Subtitle Blocks and Paragraphs (Priority: P1)

As a user reading a transcription, I want raw whisper segments merged into natural subtitle blocks (up to ~15s, closed on sentence endings, not cut after weak conjunctions) and into readable paragraphs (~45s) for TXT export, matching the quality of the reference Python script.

**Why this priority**: Raw whisper segments produce fragmented subtitles that are hard to read. The reference `transcribir.py` already implements `fusionar_segmentos` and `crear_parrafos` and produces far better output. This is the user-visible export-quality half of the feature title.

**Independent Test**: Can be tested by transcribing a standard video, viewing segments, and verifying blocks are merged to sentence boundaries and TXT export contains cohesive paragraphs rather than raw whisper fragments.

**Acceptance Scenarios**:

1. **Given** raw segments from any engine, **When** post-processing runs, **Then** segments are merged into blocks whose duration is at most ~15 seconds (`max_duration`).
2. **Given** a merge in progress, **When** the accumulated block exceeds `max_duration` or ends on a sentence-ending character (`. ! ? ; :`), **Then** the block is closed at that boundary.
3. **Given** the boundary falls after a weak conjunction (`y, o, pero, que, porque, como, cuando, si, de, del, en, con, para, por, a`), **When** the block has not reached `max_duration`, **Then** merging continues past the conjunction instead of closing there.
4. **Given** merged blocks exist, **When** TXT export runs, **Then** blocks are grouped into paragraphs up to ~45 seconds and exported with a blank line between paragraphs.
5. **Given** merging is applied, **When** each block is built, **Then** the block inherits the start of its first segment and the end of its last segment (no synthetic timestamps).

---

### User Story 4 - Accurate End Timestamps and Speaker Labels in Exports (Priority: P1)

As a user exporting subtitles from a video with multiple speakers, I want real segment end timestamps and speaker labels in every export format, so the subtitles sync correctly in players and dialogues are attributed to the correct speaker in SRT/VTT/ASS/TXT/JSON.

**Why this priority**: Exports are the primary use case for video transcription (feature 009, User Story 2 was P1 for the same reason). Today SRT/VTT/ASS compute `end = start + 3s` and the export path drops the speaker entirely even when diarization succeeded — both are correctness bugs in the feature's stated core deliverable.

**Independent Test**: Can be tested by transcribing a two-speaker interview with diarization enabled and exporting each format, verifying SRT/VTT/ASS end times match the real last-word timestamps and speaker labels appear in all formats.

**Acceptance Scenarios**:

1. **Given** a completed transcription with real segment ends, **When** SRT/VTT/ASS export runs, **Then** the end timestamp is the real segment end, not `start + 3s` (the `+3s` fallback is used only when an entry predates this feature).
2. **Given** a segment with a speaker assigned, **When** SRT/VTT export runs, **Then** the text is prefixed with `[Speaker X]`.
3. **Given** a segment with a speaker assigned, **When** ASS export runs, **Then** the speaker is written to the dialogue `Name` field (and prefixed in `Text` if no style is defined).
4. **Given** a paragraph export, **When** TXT export runs, **Then** each paragraph includes its speaker label.
5. **Given** segments with speakers, **When** JSON export runs, **Then** each object contains `start`, `end`, `text`, and `speaker` fields.
6. **Given** the frontend `export_history` payload (built without the new fields), **When** it is deserialized into `ExportEntry`, **Then** it still parses because new fields have `#[serde(default)]` (`speaker: None`, `end` falls back to start+3s).

---

### User Story 5 - Better Speaker Assignment Coverage (Priority: P2)

As a user depending on speaker labels, I want the segment-to-speaker matcher to label more segments correctly, so that most of the transcript is attributed even when a segment sits between diarization turns.

**Why this priority**: Diarization only labels segments whose midpoint falls inside a speaker turn. Segments that straddle turns or fall in short gaps are left unlabeled, which undermines the speaker features. Improving coverage is a quality enhancement on top of the already-working diarization.

**Independent Test**: Can be tested by transcribing an interview with rapid back-and-forth dialogue and verifying the percentage of labeled segments is measurably higher than the current midpoint-only matcher.

**Acceptance Scenarios**:

1. **Given** a segment whose midpoint falls outside any speaker turn but which overlaps a turn, **When** matching runs, **Then** the segment is assigned to the nearest speaker turn (overlap-based assignment).
2. **Given** a short gap between two turns of the same speaker, **When** matching runs, **Then** the speaker label propagates across the pause instead of leaving the segment unlabeled.
3. **Given** a gap between two different speakers with no speech, **When** matching runs, **Then** the segment is assigned to the most recent speaker (deterministic rule, no hallucinated speakers).
4. **Given** a single-speaker video, **When** matching runs, **Then** all segments are labeled `Speaker_0` without inventing extra speakers.

---

### User Story 6 - Clear Diarization Status Feedback (Priority: P2)

As a user enabling speaker detection, I want the completion notification and result to clearly state whether speaker detection was disabled, succeeded, or failed, so I know whether the missing labels are intentional or an error.

**Why this priority**: Currently a diarization failure is silently swallowed — the user only ever sees "Diarization failed, continuing without speaker labels" inside the progress stream, and the final result looks identical to a disabled run. Surfacing the outcome builds trust and makes failures diagnosable.

**Independent Test**: Can be tested by running three transcriptions — diarization disabled, diarization enabled with a good model, and diarization forced to fail — and verifying the final notification/result differs clearly in each case.

**Acceptance Scenarios**:

1. **Given** the user runs transcription with diarization disabled, **When** it completes, **Then** the result/notification states "Speaker detection disabled" (no error styling).
2. **Given** the user runs transcription with diarization enabled, **When** it succeeds, **Then** the result/notification states the number of speakers detected.
3. **Given** the user runs transcription with diarization enabled, **When** diarization fails, **Then** the result/notification states "Speaker detection failed" with a visible warning, and the job still completes without speaker labels.
4. **Given** a completed run, **When** the result is persisted, **Then** the diarization status is stored with the transcription so history views can show it later.

---

### Edge Cases

- What happens when the selected ct2 model directory is missing or corrupt? → The engine reports a clear error, the app keeps running, and the previously loaded engine (if any) stays usable.
- What happens when `gpu=true` but the build has no CUDA support or the driver is absent? → The ct2 engine attempts CUDA and falls back to CPU with a logged warning (mirrors the sherpa engine provider fallback).
- What happens during VAD preprocessing when an entire chunk contains no speech? → The chunk is skipped and contributes no segments; timestamps of later chunks remain correct.
- What happens when the audio contains a 30-minute silent gap? → VAD skips the silence; exported timestamps jump correctly without phantom segments.
- What happens when the whole file is silent? → Transcription returns zero segments and no crash; the UI shows an empty result.
- What happens when a subtitle block ends exactly at `max_duration` on a weak conjunction? → The block closes at `max_duration` (the conjunction rule only applies before the cap is reached).
- What happens when a sentence contains none of the closing punctuation by `max_duration`? → The block is cut at `max_duration` on the last word boundary available.
- What happens when diarization fails? → Status is "failed" (not silent) and exports still work, just without speaker labels.
- What happens when exporting an old history entry created before this feature (no `end`, no `speaker`)? → `#[serde(default)]` keeps it deserializable; end falls back to `start+3s`; speaker is `None`.
- What happens when the user selects `ctranslate2` but only ggml models are on disk? → Auto-load is skipped with an info log; the user is guided to download a CTranslate2 model.
- What happens when `beam_size` or `compute_type` are set to unsupported values? → Values are clamped/validated against ct2rs `WhisperOptions` and config bounds; invalid values log a warning and use defaults.
- What happens when the machine has no NVIDIA GPU and no CUDA build? → ct2 still runs on CPU (int8), just slower; the app must never crash at startup because of the GPU flag.

## Requirements

### Functional Requirements

**Engine Kind & Enums**

- **FR-001**: `EngineKind` in `src-tauri/src/asr/engine.rs` MUST gain a `Ctranslate2` variant with `as_str() -> "ctranslate2"` and `from_str()` mapping `"ctranslate2"` to it; any unrecognized value MUST continue to default to `Whisper`.
- **FR-002**: `AsrEngine` MUST gain a `Ctranslate2(Ctranslate2Engine)` variant wired through `new`, `kind`, `switch_kind`, `load_model`, `transcribe`, `is_loaded`, `model_path`, and `model_name` so every existing caller (pipeline, commands, video) works unchanged.
- **FR-003**: The system MUST NOT remove or rename `Whisper`/`Sherpa` variants — the new variant is additive to keep macOS/Windows future compatibility and existing configs valid.

**CTranslate2 Engine (`ct2rs`)**

- **FR-004**: A new `Ctranslate2Engine` (new module, e.g. `src-tauri/src/ct2/engine.rs`) MUST wrap `ct2rs` with the API `ct2rs::Whisper::new(model_dir, config)` and `whisper.generate_segments(&samples, language, &WhisperOptions) -> Vec<Segment{id,text,start,end,words}>`.
- **FR-005**: The engine MUST load CTranslate2-format model directories (e.g. Systran/faster-whisper-turbo: `model.bin`, `config.json`, `tokenizer.json`, `vocabulary.txt`) from `models/ct2/<name>/`; it MUST NOT accept ggml `.bin` files (clear error instead).
- **FR-006**: The engine MUST map config into `WhisperOptions`: `beam_size`, `patience`, `length_penalty`, `repetition_penalty`, `max_length`, `sampling_temperature`, `suppress_blank`, with safe defaults when unset.
- **FR-007**: `src-tauri/Cargo.toml` MUST add `ct2rs` with GPU support via its `cuda`/`cudnn` features, married to the existing `cuda`/`gpu` feature flags; the default (no-features) build MUST still compile and run ct2 on CPU.
- **FR-008**: The engine MUST honor `config.whisper.gpu`; when CUDA is requested but unavailable it MUST fall back to CPU (int8) with a logged warning, mirroring the sherpa provider fallback — the app MUST never crash from an unavailable device.
- **FR-009**: `WhisperConfig` MUST add `beam_size: u32` and `compute_type: String` (e.g. `"auto"`, `"int8"`, `"float16"`) with `#[serde(default)]` so existing TOML configs still parse; `TranscriptionParams` MUST carry the ct2-relevant options for engine construction (defaults per FR-031).
- **FR-010**: The real-time pipeline MUST keep working when the selected engine is `ctranslate2` (same `transcribe(&[f32], params)` API); VAD chunking is batch/video-specific and MUST NOT be applied to realtime chunks in this feature.

**Model Management**

- **FR-011**: A `models/ct2/` directory MUST be created alongside the existing whisper (`ggml-*.bin`) and `models/sherpa/` layouts, and the ModelManager/view MUST include CTranslate2 models in `list_available_models` with `engine: "ctranslate2"`.
- **FR-012**: The ct2 model catalog MUST include at minimum `Systran/faster-whisper-turbo` (label, languages, size, HF download URLs for the CTranslate2 file set); the download path MUST verify presence of required files before marking the model downloaded.
- **FR-013**: `download_model`, `delete_model`, `load_model`, and `switch_model` in `src-tauri/src/commands/models.rs` MUST handle ct2 models (directory under `models/ct2/`, `desired_kind = EngineKind::Ctranslate2`, `switch_kind` + `load_model`).
- **FR-014**: `lib.rs` startup auto-load MUST resolve the model path per engine kind — `models/ct2/<name>/` for `ctranslate2`, `models/sherpa/<name>/` for `sherpa`, `ggml-<name>.bin` for `whisper` — and skip with an info log when the ct2 model is absent.
- **FR-015**: Settings UI (`WhisperSettings.tsx`) and Model Manager (`ModelList.tsx`) MUST surface the engine per model (`whisper` / `sherpa` / `ctranslate2`), and the engine selector must offer `whisper`, `sherpa`, and `ctranslate2`.

**Long-Audio Video Transcription**

- **FR-016**: `transcribe_video` (`src-tauri/src/commands/video_transcription.rs`) MUST stop hardcoding `threads: 4, gpu: false` and instead build `TranscriptionParams` from `AppConfig` (`whisper.gpu`, `whisper.threads`, `whisper.engine`, `whisper.beam_size`, `whisper.compute_type`).
- **FR-017**: For the ct2 engine, `transcribe_video` MUST run VAD-based preprocessing (reusing/augmenting `src-tauri/src/vad/detector.rs`) to slice the 16kHz mono samples into speech chunks, transcribe each chunk with `generate_segments`, and offset each segment's timestamps into the full-file timeline.
- **FR-018**: The chunked transcription MUST emit per-chunk progress events (`video-transcription-progress`, step `transcribing`) so multi-hour jobs show meaningful progress.
- **FR-019**: Long-audio jobs (≥ 1h40m, single pass, ct2 engine) MUST complete without crashing and without manual chunking; whisper/sherpa paths MUST keep their current single-pass behavior.

**Segment Post-Processing**

- **FR-020**: The system MUST implement `fusionar_segmentos` (ported from `~/faster-whisper/transcribir.py`): merge raw engine segments into blocks up to `max_duration` (~15s), closing a block on a sentence-ending character (`. ! ? ; :`) and NOT closing after weak conjunctions (`y, o, pero, que, porque, como, cuando, si, de, del, en, con, para, por, a`) until the duration cap is reached.
- **FR-021**: The system MUST implement `crear_parrafos`: group merged blocks into paragraphs up to ~45s for TXT export, preserving order and real boundaries.
- **FR-022**: Post-processed blocks MUST keep real boundaries — first segment start and last segment end — and MUST be stored as `DiarizedSegment`s (start/end/text/speaker) in SQLite so all downstream consumers are unchanged.

**Exports: Real End Timestamps & Speakers**

- **FR-023**: `ExportEntry` in `src-tauri/src/commands/export.rs` MUST add `end: String` (RFC3339) and `speaker: Option<String>`, BOTH with `#[serde(default)]`, so the existing frontend `export_history` payload and old stored segments still deserialize.
- **FR-024**: `export_video_transcription` MUST map each `DiarizedSegment` to an `ExportEntry` carrying the real `end` timestamp and `speaker`.
- **FR-025**: `to_srt`, `to_vtt`, and `to_ass` MUST use the real end timestamp when present, falling back to `start + 3s` only for entries without one (backward compatibility).
- **FR-026**: `to_srt`/`to_vtt` MUST prefix speaker-bearing text with `[Speaker X]`; `to_ass` MUST place the speaker in the `Name` field (with `[Speaker X]` fallback in Text); `to_txt` MUST include the speaker for paragraphs; `to_json` MUST emit `start`, `end`, `text`, `translation`, and `speaker`.

**Speaker Matching**

- **FR-027**: The segment-to-turn matcher in `transcribe_video` MUST replace the midpoint-only lookup with nearest-turn assignment (assign to the turn with the most/greatest overlap, or the temporally nearest turn) and MUST propagate the speaker label across short pauses between turns of the same speaker.
- **FR-028**: The matcher MUST NOT invent speakers — unlabeled output is allowed only when no turn exists near the segment, and single-speaker audio MUST produce only `Speaker_0`.

**Diarization Status**

- **FR-029**: `transcribe_video` MUST return and persist a diarization status (`disabled` | `succeeded` (with speaker count) | `failed`) in the result (`VideoTranscriptionResult`) and in the stored transcription, and the frontend MUST surface it in the completion notification/result view.
- **FR-030**: The current silent `warn!` path for diarization failure MUST remain logged but the user-facing outcome MUST now say "Speaker detection failed" instead of only the in-progress message.

**Defaults & Configuration**

- **FR-031**: Default `beam_size` MUST be `5` (faster-whisper default) and default `compute_type` MUST be `"auto"` — resolved as `float16` on CUDA and `int8` on CPU unless the user overrides.
- **FR-032**: [NEEDS CLARIFICATION: The user description lists "compute type" but does not state the exact set of user-exposed values or where the setting lives in the UI — assume `int8` / `float16` / `auto` selectable in the Whisper Settings performance section, consistent with the current GPU toggle.]

### Key Entities

- **Ctranslate2Engine**: New ASR backend wrapping `ct2rs::Whisper` (CTranslate2). Holds model config, device (CUDA/CPU), `WhisperOptions` (beam_size, patience, length_penalty, repetition_penalty, max_length, sampling_temperature, suppress_blank). Swappable behind the existing `AsrEngine` enum.
- **EngineKind** (updated): `Whisper | Sherpa | Ctranslate2` — used by config (`whisper.engine`), `AsrEngine::new/switch_kind`, and model-loading commands.
- **Ct2ModelInfo**: Catalog entry for a CTranslate2 model — name, label, HF repo, required files (`model.bin`, `config.json`, `tokenizer.json`, `vocabulary.txt`), size, languages, `downloaded` state.
- **WhisperConfig** (updated): Gains `beam_size: u32` and `compute_type: String` (serde-defaulted); retains `model`, `language`, `threads`, `gpu`, `engine`.
- **ExportEntry** (updated): Adds `end: String` (RFC3339, real segment end) and `speaker: Option<String>` — both `#[serde(default)]` for backward compat with `export_history`.
- **DiarizedSegment** (unchanged, now post-processed): `start`, `end`, `text`, `speaker: Option<String>`; stored in SQLite `segments` JSON.
- **DiarizationStatus**: New discriminated result (`disabled` / `succeeded { speaker_count }` / `failed`) returned and persisted with each video transcription.

## Success Criteria

### Measurable Outcomes

- **SC-001**: On an NVIDIA GPU (CUDA build), the ct2 engine transcribes a 10-minute clip at least 4x faster than the whisper.cpp path at comparable model size (float16/int8), measured wall-clock end-to-end.
- **SC-002**: A 1h40m video transcribes successfully in a single run with the ct2 engine — no crash, no manual chunking → this was a hard failure before the feature.
- **SC-003**: VAD preprocessing skips silence: for a file with ≥20% silence, total transcribed audio is reduced proportionally and RTF improves, while exported timestamps stay in sync with the source timeline.
- **SC-004**: At least 90% of merged subtitle blocks end on a sentence-ending character or the ~15s cap (no mid-sentence cuts inside the cap, no cuts after weak conjunctions).
- **SC-005**: SRT/VTT/ASS exports from new video transcriptions contain real end timestamps; a validator confirms end ≥ start and end - start matches the stored segment duration (not `+3s`).
- **SC-006**: With diarization enabled, at least 80% of exported blocks carry a speaker label (up from the midpoint-only matcher baseline).
- **SC-007**: The existing frontend `export_history` payload continues to deserialize after the `ExportEntry` additions (serde-default backward compatibility verified by unit test).
- **SC-008**: Every video transcription run produces an explicit diarization status (`disabled` / `succeeded` / `failed`) visible in the UI — no silent failures.
- **SC-009**: `cargo test` (existing 76+ tests) and `bun run typecheck` pass with no regressions; `cargo build` succeeds both with the default feature set (CPU-only ct2) and with the `cuda`/`gpu` feature set.

## Assumptions

- The `ct2rs` crate (jkawamoto/ctranslate2-rs, bindings to CTranslate2) is compatible with the project's Rust edition / Tauri 2 toolchain and its `cuda`/`cudnn` features can be gated behind the existing Cargo `cuda`/`gpu` flags; if the crate's maintenance status blocks a clean build on some platform, the engine is feature-gated so CPU consumers still build.
- CTranslate2-format models (at minimum Systran/faster-whisper-turbo) can be downloaded from HuggingFace to `models/ct2/<name>/` and then used 100% offline, in line with the Offline-First constitution principle.
- The reference Python heuristics in `fusionar_segmentos`/`crear_parrafos` port 1:1 (sentence-end set `. ! ? ; :`, weak-conjunction list, `max_duration` ≈ 15s, paragraph target ≈ 45s); these constants are tunable in code without spec changes.
- The ct2 engine serves both the realtime overlay pipeline and the batch video path through the unified `AsrEngine`; VAD chunking is applied only to the batch video path in this feature.
- `beam_size` defaults to 5 and `compute_type` to `"auto"` (float16 on CUDA, int8 on CPU); GPU-acceleration ergonomics match the existing sherpa provider-fallback pattern.
- When `gpu=true` on a machine without a CUDA-capable build/driver, the ct2 engine degrades to CPU rather than failing the whole app — graceful degradation is required by the constraints.
- Diarization keeps using speakrs; only the segment-to-turn assignment logic changes (midpoint → nearest-turn + pause propagation), so the diarization engine API is untouched.