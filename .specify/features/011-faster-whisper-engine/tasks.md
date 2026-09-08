---

description: "Task list for faster-whisper (CTranslate2) engine, export quality, and diarization status feature"
---

# Tasks: 011-faster-whisper-engine

**Input**: Design documents from `.specify/features/011-faster-whisper-engine/`

**Prerequisites**: plan.md, spec.md

**Organization**: Tasks are grouped by milestone (M1–M4) per `plan.md`, plus a final verification phase. Each task carries a `[P]` marker when it can run in parallel (different files, no dependencies) and a `[US#]` label mapping it to the spec user story. Every task ends with a **Done when** verification line.

## Phase 1: M1 — Export Quality & Segment Merge (no new engine)

**Goal**: Port `fusionar_segmentos`/`crear_parrafos`/`es_final_de_frase` from `~/faster-whisper/transcribir.py`, apply merging in the video path, and fix SRT/VTT/ASS/TXT/JSON exports to carry real end timestamps and speaker labels. Delivers US3 + US4 with the existing whisper/sherpa engines.

- [ ] T001 [P] [US3] Create `src-tauri/src/transcription/mod.rs` (module declarations) and `src-tauri/src/transcription/merge.rs` with `MergedBlock { start, end, text }` and a 1:1 port of `es_final_de_frase` (trims quotes/parens, handles `"..."`, ends in `. ! ? ; :`), `fusionar_segmentos` (constants `MAX_DURATION = 15.0`, `PAUSA_MIN = 1.5`; close at `max_duration` even on a weak conjunction, close before the cap only on a sentence end or ≥1.5s pause, never after weak conjunctions `y, o, pero, que, porque, como, cuando, si, de, del, en, con, para, por, a` before the cap; real first-segment start / last-segment end, no synthetic timestamps) and `crear_parrafos` (`MAX_PARAGRAPH_DURATION = 45.0`, `PARAGRAPH_HARD_DURATION = 90.0`, `PARAGRAPH_HARD_CHARS = 2000`; new paragraph on ≥1.5s pause, or duration ≥ max/2 with closed sentence, or hard caps); register `pub mod transcription;` in `src-tauri/src/lib.rs`
  - **Done when**: `cargo test transcription` passes all ported-behavior tests — `test_es_final_de_frase`, `test_no_cut_after_weak_conjunction_before_cap`, `test_close_at_cap_even_on_weak_conjunction`, `test_close_on_pause`, `test_block_real_boundaries`, `test_paragraph_pause_and_hard_caps`

- [ ] T002 [P] [US4] Modify `src-tauri/src/commands/export.rs`: add `speaker: Option<String>` and `duration: Option<f64>` (real end−start seconds, both `#[serde(default)]`) to `ExportEntry`; `to_srt`/`to_vtt` use `end = start + duration` with the existing `+3s` fallback when `duration` is `None`, and prefix `[Speaker X] ` when `speaker` is `Some`; `to_ass` writes speaker to the Dialogue `Name` field plus `[Speaker X] ` prefix in Text; `to_txt` builds paragraphs via `crear_parrafos` (from T001) when any entry carries `duration`, prefixing each paragraph's first speaker, keeping the legacy flat join for history-origin entries; `to_json` emits `speaker`/`duration` automatically via serde; update `test_entries()` fixture with `speaker: None, duration: None`
  - **Done when**: `cargo test export` passes new tests — `test_srt_end_uses_real_duration`, `test_srt_fallback_plus_3s`, `test_srt_speaker_prefix`, `test_vtt_speaker_prefix`, `test_ass_speaker_name_field`, `test_txt_paragraphs_with_speaker`, `test_txt_legacy_flat`, `test_json_includes_speaker`, `test_export_entry_serde_defaults` (legacy `export_history` payload without new fields still parses, SC-007)

- [ ] T003 [US4] Modify `src-tauri/src/commands/video_transcription.rs`: `export_video_transcription` maps each merged block / `DiarizedSegment` to `ExportEntry { id, timestamp: seconds_to_rfc3339(start), language, original_text: text, translation: None, speaker: seg.speaker.clone(), duration: Some(seg.end - seg.start) }`
  - **Done when**: `cargo check` passes; manual — export SRT/VTT/ASS from a diarized video shows `end ≥ start` with real durations (≠ 3s) and `[Speaker X]` prefixes

- [ ] T004 [US3] Modify `src-tauri/src/commands/video_transcription.rs`: in `transcribe_video`, after the engine returns raw segments and before speaker assignment, run `fusionar_segmentos(&segments, MAX_DURATION)` and map each block to `DiarizedSegment { start: block.start, end: block.end, text: block.text, speaker: <midpoint matcher> }`; keep the existing `merging` progress step
  - **Done when**: `cargo test` passes; manual — whisper/sherpa video transcription shows merged blocks ≤ ~15s closed on sentence boundaries instead of raw fragments

**Milestone M1 gate**: `cargo check && cargo test` and `bun run typecheck && bun run lint`

---

## Phase 2: M2 — CTranslate2 Engine, Models & Settings

**Goal**: Third ASR backend via `ct2rs`, model catalog/download, engine-kind wiring, config fields, and UI controls. Delivers US1 (FR-001..FR-015).

- [ ] T005 [P] [US1] Modify `src-tauri/Cargo.toml`: add `ct2rs = { version = "0.9", default-features = false, features = ["whisper", "all-tokenizers", "ruy"] }` NON-optional so the module always compiles; extend the `cuda` feature with `"ct2rs/cuda", "ct2rs/cudnn", "ct2rs/cuda-dynamic-loading"` alongside the existing `whisper-rs/cuda`; `gpu = ["cuda"]` unchanged (default build = CPU ct2, SC-009)
  - **Done when**: `cargo check` (default, CPU ct2) passes; `cargo check --features gpu` passes where a CUDA toolchain is present (else noted as CI job with CUDA runner)

- [ ] T006 [US1] Create `src-tauri/src/ct2/mod.rs`, `src-tauri/src/ct2/engine.rs`, and `src-tauri/src/ct2/models.rs`:
  - `Ctranslate2Engine` wrapping `ct2rs::Whisper`: `load_model(&mut self, model_path: &Path, gpu: bool)` requires a **directory** (`model.bin` + `config.json` + `tokenizer.json`), rejects bare ggml `.bin` with a clear error, resolves compute type (`float16` on GPU, `int8` fallback on CPU, invalid → warn + `"float16"`), tries `Device::Cuda` then falls back to `Device::Cpu` with a logged warning (FR-008); `transcribe(&self, audio, params)` via `whisper.generate` / `generate_segments` with `WhisperOptions { beam_size: params.beam_size.max(1), suppress_blank: true, repetition_penalty: 1.0, ..Default::default() }` and `language = None` for `"auto"`/`None`; maps `Segment { start, end, text }` f32→f64 into `TranscriptionSegment`; rebuild with `cpu_threads = params.threads` when threads change
  - `Ct2ModelInfo` catalog for `Systran/faster-whisper-{tiny,base,small,medium,large-v3,turbo}` (label, languages, size ~75MB–2.9GB, HF `resolve/main` URLs), downloaded to `models/ct2/<name>/`; download/verify requires the FULL file set `model.bin`, `config.json`, `tokenizer.json`, `preprocessor_config.json`, `vocabulary.json`, `generation_config.json`; `list_downloaded` scans `models/ct2/` dirs; `model_dir = models_dir.join("ct2").join(name)`
  - **Done when**: `cargo test ct2` passes — `test_engine_new_not_loaded`, `test_rejects_ggml_bin`, `test_compute_type_fallback_cpu`, `test_whisper_options_mapping`, `test_valid_language_none_for_auto`, `test_catalog_six_models`, `test_catalog_files_have_hf_urls`, `test_model_dir_layout`, `test_is_downloaded_false_when_missing`, `test_list_downloaded_only_dirs`

- [ ] T007 [US1] Modify `src-tauri/src/asr/engine.rs`: add `EngineKind::Ctranslate2` (additive — `Whisper`/`Sherpa` untouched), `from_str` maps `"ct2" | "ctranslate2"` → `Ctranslate2` (anything else → `Whisper`), `as_str` → `"ctranslate2"`; add `AsrEngine::Ctranslate2(Ctranslate2Engine)` variant and extend every arm — `new`, `kind`, `switch_kind`, `load_model`, `transcribe`, `is_loaded`, `model_path`, `model_name` (ct2 `model_name` = dir basename); add `use crate::ct2::engine::Ctranslate2Engine;`
  - **Done when**: `cargo test engine` passes new tests — `test_engine_kind_from_str` (`"ct2"`/`"ctranslate2"` → Ctranslate2, `"bogus"` → Whisper), `test_engine_kind_as_str`, `test_switch_kind_to_ct2`; `cargo check` passes

- [ ] T008 [P] [US1] Modify `src-tauri/src/whisper/params.rs` and `src-tauri/src/settings/config.rs`: add `pub beam_size: u32` (default `5`) and `pub compute_type: String` (default `"float16"`) to `TranscriptionParams` (Default impl) and to `WhisperConfig` with `#[serde(default = "default_whisper_beam_size")]` / `#[serde(default = "default_whisper_compute_type")]` so existing TOML still parses; `default_whisper_engine` stays `"whisper"`; existing construction sites compile unchanged via `..Default::default()`
  - **Done when**: `cargo test settings whisper` passes — updated `test_default_whisper_config`, `test_deserialize_from_toml` assert new defaults, plus `test_whisper_legacy_toml_without_new_fields_parses`

- [ ] T009 [US1] Modify `src-tauri/src/commands/models.rs` with a ct2 branch mirroring the sherpa branches byte-for-byte: `list_available_models` appends ct2 catalog entries with `engine: "ctranslate2"` and `downloaded = is_downloaded`; `download_model` routes ct2 names to `ct2_models::download` before the whisper fallback; `delete_model` removes `models/ct2/<name>`; `list_downloaded_models` includes `ct2_models::list_downloaded`; `load_model`/`switch_model` resolve `model_path = ct2_models::model_dir(&models_dir, &model_name)` (a dir) for ct2, `desired_kind = EngineKind::Ctranslate2`, and persist `cfg.whisper.engine = "ctranslate2"`; ggml + sherpa paths keep working
  - **Done when**: `cargo test` passes; manual — `download_model({modelName:"turbo"})` populates `models/ct2/turbo/`, `switch_model({modelName:"turbo"})` → `get_loaded_model` reports it

- [ ] T010 [US1] Modify `src-tauri/src/lib.rs`: register `pub mod transcription;` and `pub mod ct2;`; in `.setup()`, resolve the startup model path per kind — `EngineKind::Ctranslate2 => models_dir.join("ct2").join(&model_name)` (a directory), `Sherpa => models_dir/ sherpa/<name>`, `Whisper => models_dir/ggml-<name>.bin`; skip auto-load with an info log when the ct2 dir is missing (edge case: only ggml models on disk)
  - **Done when**: `cargo check` passes; manual — set `whisper.engine = "ctranslate2"` with a downloaded ct2 model → auto-loads at startup (US1-2)

- [ ] T011 [US1] Frontend: modify `src/hooks/useSettings.ts` — `whisper` interface gains `beam_size: number; compute_type: string;`; modify `src/components/Settings/WhisperSettings.tsx` — engine select (`whisper` / `sherpa` / `ctranslate2`), Compute Type select (`float16` / `int8` / `int8_float16`, hint "Used by the faster-whisper (ctranslate2) engine"), Beam Size number field (1–10, default 5), persisted in `handleSave` via `save_config`; extend the sherpa accent badge to also highlight `ctranslate2`; verify `src/components/ModelManager/ModelList.tsx` shows ct2 catalog rows + downloaded `models/ct2/` dirs (auto via `list_available_models`) and `src/components/Onboarding/steps/StepModelSelection.tsx` has no engine-specific assumptions
  - **Done when**: `bun run typecheck && bun run lint` pass; manual — select a `ctranslate2` model in Settings → downloads via Model Manager → shows Active

**Milestone M2 gate**: `cargo check && cargo test` and `bun run typecheck && bun run lint`

---

## Phase 3: M3 — Video Transcription Integration (GPU / VAD / Fusion)

**Goal**: Make `transcribe_video` read engine/GPU/threads/beam/compute_type from `AppConfig`, add ct2-specific VAD chunking with timeline offsets + per-chunk progress, keep whisper/sherpa single-pass. Delivers US2 (FR-016..FR-019).

- [ ] T012 [US2] Modify `src-tauri/src/commands/video_transcription.rs`: add `config_state: State<'_, Arc<Mutex<AppConfig>>>` parameter to `transcribe_video` (mirrors the `ai_config_state` pattern); build `TranscriptionParams` from config — `threads: cfg.whisper.threads`, `gpu: cfg.whisper.gpu`, `beam_size: cfg.whisper.beam_size`, `compute_type: cfg.whisper.compute_type.clone()` — removing the hardcoded `gpu: false, threads: 4`; clone needed values inside the lock and drop the guard before any heavy/await work
  - **Done when**: `cargo check` passes; grep confirms no `threads: 4, gpu: false` hardcode remains in `transcribe_video`

- [ ] T013 [US2] Add `transcribe_with_vad` helper (new free fn in `src-tauri/src/commands/video_transcription.rs`; reuses `src-tauri/src/vad/detector.rs` `VadDetector`): energy VAD over ~1s frames finds speech windows (end after ~2 silent frames dead band), per window take `samples[win_start..win_end]` padded 0.3s (4800 samples) each side clamped to bounds, skip empty/no-speech windows, `engine.transcribe(&chunk, params)`, offset every returned segment by the window start into the full-file absolute timeline, accumulate results, and emit `"video-transcription-progress"` (`step: "transcribing"`, progress `(i+1)/total`, `"Transcribing chunk i/total"`) per chunk; whole-file silence → empty `Vec`, no crash; extract pure helpers `offset_segments(segs, delta)` and window math for testing; wire into `transcribe_video` for `EngineKind::Ctranslate2` **only** (whisper/sherpa keep single-pass), running `fusionar_segmentos` (T004) across chunk boundaries afterward
  - **Done when**: `cargo test` passes new tests — `test_offset_segments`, `test_padding_clamped`, `test_window_builder_skips_empty`; manual — 1h40m video with `engine=ctranslate2, gpu=true` completes one pass with per-chunk progress and timestamps in sync across silence gaps (SC-002/SC-003)

**Milestone M3 gate**: `cargo check && cargo test` + the 1h40m manual run above

---

## Phase 4: M4 — Diarization Status & Speaker Matching Coverage

**Goal**: Better segment→speaker assignment (nearest-turn + gap propagation) and explicit diarization outcome surfaced in result, DB, and UI. Delivers US5 + US6 (FR-027..FR-030).

- [ ] T014 [US5] Modify `src-tauri/src/commands/video_transcription.rs`: replace the inline midpoint lookup with a pure `assign_speakers(blocks: &[(f64, f64)], turns: &[SpeakerTurn]) -> Vec<Option<String>>` — (1) greatest-overlap turn wins, ties → temporally nearest turn start; (2) no overlap → nearest turn by `|start - turn.start|`; (3) gap ≤ ~2.0s between two turns of the SAME speaker → propagate that speaker, gap between different speakers → most-recent speaker (deterministic); (4) never invents speakers — empty `turns` → all `None`, single-speaker audio → only its speaker; parse speakrs `"SPEAKER_00"` ids to numbers for colors
  - **Done when**: `cargo test` passes new tests — `test_overlap_wins_over_midpoint`, `test_nearest_turn_no_overlap`, `test_same_speaker_gap_propagates`, `test_mixed_gap_uses_most_recent`, `test_single_speaker_only`, `test_no_turns_no_labels`

- [ ] T015 [US6] Modify `src-tauri/src/commands/video_transcription.rs` and `src-tauri/src/history/db.rs`: add `diarization_status: String` (`"disabled" | "succeeded" | "failed"`) and `speaker_count: u32` to `VideoTranscriptionResult` and `VideoTranscriptionEntry`; in `transcribe_video` set status from the diarization outcome (`disabled` when `!enable_diarization`, `succeeded` with distinct speaker count, `failed` keeps the existing `warn!` log but surfaces "Speaker detection failed"); final `done` progress message differs per status — `"Transcription complete"` / `"Transcription complete — 2 speakers detected"` / `"Transcription complete — Speaker detection failed"`; `history/db.rs` additive migration in table init — `ALTER TABLE video_transcriptions ADD COLUMN diarization_status TEXT NOT NULL DEFAULT 'disabled'; ADD COLUMN speaker_count INTEGER NOT NULL DEFAULT 0;` guarded by a `PRAGMA table_info` existence check — plus read/write in `insert_video_transcription`, `get_video_transcription`, `get_video_transcriptions` (old rows read `disabled`/0)
  - **Done when**: `cargo test` passes new tests — `test_assigned_status_disabled`, `test_speaker_count_distinct`, DB round-trip for the new columns; `cargo check` passes

- [ ] T016 [US6] Frontend: modify `src/hooks/useVideoTranscription.ts` — `VideoTranscriptionResult`/`VideoTranscriptionEntry` add `diarizationStatus: string; speakerCount: number;` (tolerate old payloads via `||` fallbacks); modify `src/components/VideoTranscription/VideoTranscriptionPage.tsx` — render a small status badge next to the video name: `succeeded` → `{speakerCount} speaker(s) detected` (accent/success), `disabled` → `"Speaker detection disabled"` (muted), `failed` → `"Speaker detection failed"` (danger styling + warning icon); `ProgressIndicator` shows the final `done` message (already event-driven) and `TranscriptionViewer` stays unchanged
  - **Done when**: `bun run typecheck && bun run lint` pass; manual 3-run matrix (disabled / enabled-good / forced-fail) yields distinct final notification and badge per run (US6 acceptance)

**Milestone M4 gate**: `cargo check && cargo test` and `bun run typecheck && bun run lint`

---

## Phase 5: Verification

- [ ] T017 Run full verification: `cargo test` (all new merge/export/`assign_speakers`/ct2/config/VAD tests + existing 76), `cargo check` (default and `--features gpu` where toolchain allows), `bun run typecheck`, `bun run lint`, `bun run fmt:check`
  - **Done when**: all green — `cargo test`, `cargo check`, `bun run typecheck`, `bun run lint`, `bun run fmt:check` exit 0 with no regressions (SC-009)

---

## Dependencies & Execution Order

### Parallel Groups

- **Group A** (no deps): T001 (merge module), T005 (Cargo.toml), T008 (params/config fields)
- **Group B** (depends A): T002 (export formats; needs T001's `crear_parrafos` for the txt-paragraph path), T006 (ct2 engine+models; needs T005)
- **Group C** (depends B): T003 (needs T002), T004 (needs T001), T007 (needs T006), T012 (needs T008)
- **Group D** (depends C): T009 (needs T006+T007), T013 (needs T007+T012), T014 (needs T004 — matcher is a pure fn, can start as soon as M1.3 lands)
- **Group E** (depends D): T010 (needs T007+T009), T011 (needs T009), T015 (needs T013+T014)
- **Group F** (depends E): T016 (needs T015)
- **Group G** (depends F): T017 (needs all)

### Critical Path

T005 → T006 → T007 → T013 → T015 → T016 → T017

(The M1 chain T001 → T004 → T014 runs in parallel with M2 and merges into T015; T008 → T012 joins the path at T013; frontend T011 completes M2 in parallel with M3.)

---

## Notes

- [P] tasks run in parallel (different files, no dependencies)
- Reference port: `~/faster-whisper/transcribir.py` lines 103–322 define `es_final_de_frase`/`fusionar_segmentos`/`crear_parrafos` — constants port directly (`--beam-size 5`, `--max-segment-duration 15`, `compute_type float16` on CUDA); values are tunable without spec changes
- `ct2rs` 0.9 is NON-optional (`default-features=false` → CPU builds fast) but compiles CTranslate2 from source on first build — C/C++ toolchain required; GPU rides the existing `cuda`/`gpu` features; CI should add a matrix job with and without `--features gpu`
- Realtime overlay keeps the unified `AsrEngine::transcribe` API for `ctranslate2` (FR-010); VAD chunking is applied ONLY in the batch video path
- `cd src-tauri && cargo build` once before running tests (first ct2 compile is slow)
- Milestone gates: run `cargo check && cargo test` + `bun run typecheck && bun run lint` at the end of each milestone (M1/M2/M4 have frontend changes, M3 is backend-only plus manual long-video validation)