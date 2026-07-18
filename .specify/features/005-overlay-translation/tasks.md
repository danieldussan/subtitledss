# Tasks: Accumulative Overlay + Translation

**Input**: Design documents from `.specify/features/005-overlay-translation/`

**Prerequisites**: plan.md, spec.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US5)

---

## Phase 1: Foundational (Config Expansion)

**Purpose**: Extend config structs with new fields. All other work depends on this.

- [ ] T001 [US1] Extend `OverlayConfig` in `src-tauri/src/settings/config.rs` with `display_duration_ms` (default 10000), `fade_duration_ms` (default 3000), `max_visible_lines` (default 4), `line_gap` (default 4)
- [ ] T002 [US2] Extend `TranslationConfig` in `src-tauri/src/settings/config.rs` with `engine` (default "whisper"), `libretranslate_url` (default "http://localhost:5000"), `show_original` (default true)
- [ ] T003 [US1] Update default values in `impl Default for AppConfig` to include new fields
- [ ] T004 [US1] Update TOML serialization tests in `config.rs` to include new fields
- [ ] T005 [US1] Update `src/hooks/useSettings.ts` TypeScript `AppConfig` interface with new translation and overlay fields
- [ ] T006 [US1] Update `src-tauri/src/lib.rs` overlay_config initialization with new fields

**Checkpoint**: Config compiles, tests pass, frontend type-checks.

---

## Phase 2: Whisper Translate (US2)

**Purpose**: Enable Whisper's built-in translate mode for offline English translation.

- [ ] T007 [US2] In `src-tauri/src/whisper/engine.rs`, add `whisper_params.set_translate(params.translate)` after line 55 (after `set_suppress_blank`)
- [ ] T008 [US2] In `src-tauri/src/pipeline/transcriber.rs`, read `config.translation.enabled` and set `translate: true` in TranscriptionParams when enabled and target_lang is "en"
- [ ] T009 [US2] Test whisper translate by running capture with a non-English audio source

**Checkpoint**: Whisper translate works offline, produces English text from foreign audio.

---

## Phase 3: Accumulative Overlay (US1)

**Purpose**: Rewrite overlay to support multi-line accumulation with gradual fade.

- [ ] T010 [US1] In `src-tauri/tauri.conf.json`, increase overlay window `height` from 80 to 200
- [ ] T011 [US1] Rewrite `public/overlay.html` — remove single `<p>`, add `<div id="lines">` container
- [ ] T012 [US1] In overlay.html, implement `SubtitleLine` class with opacity lifecycle (displayTime → fadeTime → remove)
- [ ] T013 [US1] In overlay.html, implement `requestAnimationFrame` loop to update line opacities and remove expired lines
- [ ] T014 [US1] In overlay.html, listen for `transcription` event and create new SubtitleLine with `id`, `text`, `translation`
- [ ] T015 [US1] In overlay.html, implement line stacking: new lines at bottom, old lines shift up, max 4 visible
- [ ] T016 [US1] In overlay.html, implement auto-hide: when all lines expire, hide the subtitle container
- [ ] T017 [US1] In overlay.html, style each line with: original text white, translation text in accent color below, configurable font size
- [ ] T018 [US1] Pass config values (display_duration_ms, fade_duration_ms, max_visible_lines) to overlay via event payload or data attributes

**Checkpoint**: Overlay accumulates lines, fades gradually, shows translation when present.

---

## Phase 4: Translation Module (US3)

**Purpose**: Create LibreTranslate HTTP client as alternative translation engine.

- [ ] T019 [US3] Create `src-tauri/src/translation/mod.rs` — define `TranslationEngine` enum (`Whisper`, `LibreTranslate`) and `async fn translate(text, source, target, engine, config) -> Result<String>`
- [ ] T020 [US3] Create `src-tauri/src/translation/libretranslate.rs` — implement POST to `{url}/translate` with reqwest, payload `{ q, source, target, format: "text" }`
- [ ] T021 [US3] In `libretranslate.rs`, handle connection errors gracefully (return error, don't crash)
- [ ] T022 [US3] Add `pub mod translation;` to `src-tauri/src/lib.rs`

**Checkpoint**: Translation module compiles, LibreTranslate client can be called.

---

## Phase 5: Pipeline Integration (US1+US2+US3)

**Purpose**: Connect translation to the transcription pipeline and emit enriched events.

- [ ] T023 [US1] In `src-tauri/src/pipeline/transcriber.rs`, change event payload to include `id` (timestamp millis) and `translation: Option<String>`
- [ ] T024 [US2] In `transcriber.rs`, when `translation.engine == "whisper"` and `translation.enabled`, re-run whisper with `translate: true` and add result to event
- [ ] T025 [US3] In `transcriber.rs`, when `translation.engine == "libretranslate"` and `translation.enabled`, call `translation::libretranslate::translate()` async
- [ ] T026 [US3] In `transcriber.rs`, handle translation errors gracefully — emit event with `translation: null` and log warning
- [ ] T027 [US1] In `src-tauri/src/pipeline/transcriber.rs`, store translation in history DB when inserting transcription entries

**Checkpoint**: Pipeline emits events with translation, overlay shows translated text.

---

## Phase 6: Translation Settings UI (US4)

**Purpose**: Create Translation tab in Settings panel.

- [ ] T028 [US4] Create `src/components/Settings/TranslationSettings.tsx` — enable toggle, engine selector (Whisper/LibreTranslate), source language select, target language select, LibreTranslate URL input (conditional on engine), show-original toggle, save button
- [ ] T029 [US4] In `src/components/Settings/SettingsPanel.tsx`, add `{ id: "translation", label: "Translation", icon: Languages }` to tabs array and render `<TranslationSettings>` when active
- [ ] T030 [US4] Add `Languages` icon import from lucide-react in SettingsPanel.tsx

**Checkpoint**: Translation settings UI works, saves to config, applies to pipeline.

---

## Phase 7: Toggle Shortcut (US5)

**Purpose**: Register Ctrl+Shift+T global shortcut for translation toggle.

- [ ] T031 [US5] In `src-tauri/src/lib.rs`, register `Ctrl+Shift+T` shortcut (similar to existing Ctrl+Shift+S and Ctrl+Shift+O)
- [ ] T032 [US5] In `lib.rs`, emit `toggle-translation` event on shortcut press
- [ ] T033 [US5] In `src/App.tsx`, listen for `toggle-translation` event, toggle `config.translation.enabled`, save config

**Checkpoint**: Ctrl+Shift+T toggles translation from any window.

---

## Phase 8: Polish & Cross-Cutting

**Purpose**: Final testing, cleanup, edge cases.

- [ ] T034 [P] Verify overlay transparency on Hyprland/Wayland
- [ ] T035 [P] Test LibreTranslate fallback when service is down
- [ ] T036 [P] Verify config migration (existing TOML files load correctly with new defaults)
- [ ] T037 [P] Test whisper translate with different audio qualities
- [ ] T038 [P] Run `cargo test` and `bun run typecheck` — all pass
- [ ] T039 [P] Update AGENTS.md with new translation module conventions

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Config)**: No dependencies — START HERE
- **Phase 2 (Whisper)**: Depends on Phase 1 (uses new config fields)
- **Phase 3 (Overlay)**: Depends on Phase 1 (uses new overlay config), independent of Phase 2
- **Phase 4 (Translation module)**: Depends on Phase 1, can run parallel with Phase 3
- **Phase 5 (Pipeline)**: Depends on Phase 2 + Phase 4 (needs both engines ready)
- **Phase 6 (UI)**: Depends on Phase 1 (uses config interface)
- **Phase 7 (Shortcut)**: Depends on Phase 6 (toggles same setting)
- **Phase 8 (Polish)**: Depends on all previous phases

### Parallel Opportunities

- Phase 2 (Whisper) || Phase 3 (Overlay) || Phase 4 (Translation module) — all can run in parallel after Phase 1
- Phase 6 (UI) can run in parallel with Phase 4 + Phase 5
- Phase 8 tasks marked [P] can all run in parallel

### Within Each Phase

- Config structs before tests
- Backend before frontend integration
- Core implementation before error handling
- Phase complete before moving to next

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Config
2. Complete Phase 3: Overlay
3. **STOP and VALIDATE**: Test overlay accumulation independently
4. Deploy/demo if ready

### Incremental Delivery

1. Config → Overlay accumulation (MVP!)
2. + Whisper translate → Basic translation works
3. + LibreTranslate module → Multi-language support
4. + Pipeline integration → Full feature complete
5. + Settings UI → User-friendly configuration
6. + Shortcut → Quick toggle
7. + Polish → Production-ready

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each phase should be completable and testable independently
- Commit after each task or logical group
- Stop at any checkpoint to validate independently
- The overlay rewrite (Phase 3) is the highest-risk item — test early on target platform
