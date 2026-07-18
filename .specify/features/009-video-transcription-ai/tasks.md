---

description: "Task list for video transcription, AI chat, and speaker diarization feature"
---

# Tasks: 009-video-transcription-ai

**Input**: Design documents from `.specify/features/009-video-transcription-ai/`

**Prerequisites**: plan.md, spec.md

## Phase 1: Video Processing — FFmpeg Audio Extraction

- [ ] T001 [P] Create `src-tauri/src/video/mod.rs` with module declarations
- [ ] T002 [P] Create `src-tauri/src/video/processor.rs` with `VideoProcessor` struct implementing `extract_audio`, `get_duration`, `get_metadata`
- [ ] T003 Add unit tests for `VideoProcessor` in `video/processor.rs`

---

## Phase 2: AI Provider System

- [ ] T004 [P] Create `src-tauri/src/ai/mod.rs` with module declarations
- [ ] T005 [P] Create `src-tauri/src/ai/config.rs` with `AiProviderType`, `AiConfig`, `ChatMessage`, `ChatRole` structs
- [ ] T006 [P] Create `src-tauri/src/ai/provider.rs` with `AiProvider` trait and `OpenAiCompatibleProvider` implementation
- [ ] T007 Create `src-tauri/src/ai/commands.rs` with `ai_summarize`, `ai_chat`, `ai_chat_stream`, `list_ai_providers`, `test_ai_connection` commands
- [ ] T008 Add unit tests for AI config serialization and provider defaults

---

## Phase 3: Speaker Diarization

- [ ] T009 [P] Add `speakrs` dependency to `src-tauri/Cargo.toml`
- [ ] T010 [P] Create `src-tauri/src/diarization/mod.rs` with module declarations
- [ ] T011 [P] Create `src-tauri/src/diarization/engine.rs` with `DiarizationEngine` wrapping speakrs `OfflineDiarizer`
- [ ] T012 Add unit tests for diarization engine initialization

---

## Phase 4: Export — ASS/SSA Format

- [ ] T013 [P] Add `ExportFormat::Ass` variant to `src-tauri/src/commands/export.rs`
- [ ] T014 [P] Implement `to_ass()` function with Script Info, V4+ Styles, and Dialogue sections
- [ ] T015 Add unit tests for ASS format output

---

## Phase 5: Database & Config Changes

- [ ] T016 Add `AiSettingsConfig` struct to `src-tauri/src/settings/config.rs` with defaults for each provider
- [ ] T017 Add `ai` field to `AppConfig` struct and update `Default` impl
- [ ] T018 Add `create_video_transcriptions_table()` method to `src-tauri/src/history/db.rs`
- [ ] T019 Update `src/hooks/useSettings.ts` to include `ai` section in `AppConfig` interface

---

## Phase 6: Tauri Commands — Video Transcription

- [ ] T020 Create `src-tauri/src/commands/video_transcription.rs` with `VideoTranscriptionState` struct
- [ ] T021 Implement `transcribe_video` command (orchestrates video processing → whisper → diarization)
- [ ] T022 Implement `list_video_transcriptions` command
- [ ] T023 Implement `delete_video_transcription` command
- [ ] T024 Implement `export_video_transcription` command (delegates to export.rs with all formats)
- [ ] T025 Wire all new modules in `src-tauri/src/lib.rs` (mod declarations, state management, command registration)

---

## Phase 7: Frontend — Video Transcription Page

- [ ] T026 Add `"video"` to `Section` type in `src/components/Layout/types.ts`
- [ ] T027 Add `Video` icon and nav item to `SECTION_ITEMS` in `src/components/Layout/types.ts`
- [ ] T028 Add `case "video"` to `SectionRouter.tsx`
- [ ] T029 Create `src/hooks/useVideoTranscription.ts` state management hook
- [ ] T030 Create `src/components/VideoTranscription/ProgressIndicator.tsx`
- [ ] T031 Create `src/components/VideoTranscription/VideoPicker.tsx`
- [ ] T032 Create `src/components/VideoTranscription/TranscriptionViewer.tsx`
- [ ] T033 Create `src/components/VideoTranscription/ExportMenu.tsx`
- [ ] T034 Create `src/components/VideoTranscription/VideoTranscriptionPage.tsx`

---

## Phase 8: Frontend — AI Panel

- [ ] T035 Create `src/components/VideoTranscription/AiChatMessage.tsx`
- [ ] T036 Create `src/components/VideoTranscription/AiPanel.tsx` with Summary and Chat tabs
- [ ] T037 Integrate AiPanel into VideoTranscriptionPage
- [ ] T038 Create `src/components/Settings/AiSettings.tsx`
- [ ] T039 Add AiSettings to SettingsLayout.tsx (general tab)

---

## Phase 9: Wiring & Integration

- [ ] T040 Verify `cargo check` passes
- [ ] T041 Verify `cargo test` passes
- [ ] T042 Verify `bun run typecheck` passes
- [ ] T043 Verify `bun run lint` passes

---

## Phase 10: End-to-End Testing

- [ ] T044 Test video transcription with MP4 file
- [ ] T045 Test export in all 5 formats (SRT, VTT, TXT, JSON, ASS)
- [ ] T046 Test speaker diarization with multi-speaker video
- [ ] T047 Test AI summary generation with Ollama
- [ ] T048 Test AI chat streaming with Ollama
- [ ] T049 Test AI provider settings persistence

---

## Dependencies & Execution Order

### Parallel Groups

- **Group A** (no deps): T001-T002, T004-T006, T009-T011, T013-T014, T016-T019
- **Group B** (depends on A): T007-T008, T012, T015, T020-T025
- **Group C** (depends on B): T026-T039
- **Group D** (depends on C): T040-T049

### Critical Path

T001 → T002 → T020 → T021 → T025 → T026 → T034 → T040 → T044

---

## Notes

- [P] tasks can run in parallel (different files, no dependencies)
- FFmpeg must be installed: `pacman -S ffmpeg`
- speakrs models auto-download on first diarization (~30MB)
- All AI providers use OpenAI-compatible API — single implementation covers all three
