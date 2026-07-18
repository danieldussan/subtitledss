# Tasks: Translation Integration QA Fixes

**Feature**: 006-translation-qa-fixes
**Spec**: `.specify/features/006-translation-qa-fixes/spec.md`
**Plan**: `.specify/features/006-translation-qa-fixes/plan.md`
**Total tasks**: 22
**Estimated effort**: 2-3 days

---

## Dependency Graph

```
Phase 1 (Critical) ─── No dependencies, foundation layer
  ├─ 1A: Delete History Entry    ─── standalone
  ├─ 1B: Translation State Sync  ─── standalone
  └─ 1C: Overlay Toggle Sync     ─── standalone

Phase 2 (Moderate) ─── Depends on Phase 1
  ├─ 2A: SRT/VTT Export          ─── standalone
  ├─ 2B: Shortcuts from Config   ─── standalone
  ├─ 2C: HTTP Timeout            ─── standalone (adds timeout_seconds to config)
  └─ 2D: Deep Merge              ─── standalone

Phase 3 (Architecture) ─── Depends on Phase 2
  ├─ 3A: Dispatcher Cleanup      ─── depends on 2C (timeout_seconds field)
  ├─ 3B: show_original           ─── depends on 3A (dispatcher uses correct config)
  ├─ 3C: Whisper Constraints     ─── standalone
  └─ 3D: Hot-Reload              ─── depends on 3A (dispatcher uses Arc<Config>)

Phase 4 (Cleanup) ─── Depends on Phase 3
  ├─ 4A: Dead auto Logic         ─── standalone
  └─ 4B: Dead Model Manager      ─── standalone
```

---

## Phase 1: Critical Fixes (P1)

> **Goal**: Fix 3 critical bugs that cause runtime errors or UI desync.
> **Dependencies**: None — this is the foundation.
> **Files modified**: 4 files

### 1A. Delete History Entry Command (Issue #1)

- [ ] **1A-1** [P1] [US1] Add `delete_history_entry` Tauri command function in `src-tauri/src/commands/history.rs`
  - Function signature: `pub async fn delete_history_entry(id: i64, state: State<'_, Arc<Mutex<HistoryDb>>>) -> Result<(), String>`
  - Uses existing `db.delete(id)` method (already tested in unit tests)
  - Return `Ok(())` on success, `Err(e.to_string())` on failure
  - Place after existing `clear_history` command (~line 34)
  - **FR-001, FR-002, FR-003**

- [ ] **1A-2** [P1] [US1] Register command in Tauri invoke handler in `src-tauri/src/lib.rs`
  - Add `commands::history::delete_history_entry` to the `invoke_handler` list
  - Place after existing history commands (~line 103)
  - **FR-002**

- [ ] **1A-3** [P1] [US1] Add toast notifications to delete handler in `src/components/History/HistoryList.tsx`
  - Import `useToast` hook (already available at `src/hooks/useToast.ts`)
  - Wrap `handleDeleteEntry` (lines 127-135) in try/catch
  - Show `toast.success("Entry deleted")` on success
  - Show `toast.error(msg)` on failure
  - **FR-003, FR-004**

### 1B. Translation State Sync on Startup (Issue #2)

- [ ] **1B-1** [P1] [US2] Add `loadTranslationState` function in `src/App.tsx`
  - Create async function that calls `invoke<AppConfig>("get_config")`
  - Set `setTranslationEnabled(config.translation.enabled)` from loaded config
  - Handle errors with `console.error`
  - **FR-005**

- [ ] **1B-2** [P1] [US2] Call `loadTranslationState` on mount in `src/App.tsx`
  - Add call inside the mount `useEffect` (lines 89-108) alongside existing `loadModelState()` and `loadAudioDevice()`
  - **FR-006**

### 1C. Overlay Toggle Sync (Issue #11)

- [ ] **1C-1** [P1] [US10] Use backend return value for overlay toggle in `src/App.tsx`
  - Modify `toggleOverlay` (lines 52-59) to use `const visible = await invoke<boolean>("toggle_overlay")`
  - Set `setOverlayVisible(visible)` using the returned boolean
  - Remove optimistic state inversion (`setOverlayVisible((prev) => !prev)`)
  - **FR-028, FR-029**

- [ ] **1C-2** [P1] [US10] Emit `toggle-overlay` event from system tray handler in `src-tauri/src/lib.rs`
  - Modify the `"show_overlay"` tray handler (lines 171-177) to emit `app.emit("toggle-overlay", ())`
  - This makes tray action go through the same event path as global shortcuts
  - Frontend will call `toggle_overlay` which returns the correct state
  - **FR-030**

### Phase 1 Verification Gate

```bash
cargo check && cargo test && bun run typecheck && bun run lint
```

---

## Phase 2: Moderate Fixes (P2)

> **Goal**: Fix 4 moderate bugs affecting export, shortcuts, HTTP reliability, and config merging.
> **Dependencies**: Phase 1 complete.
> **Files modified**: 5 files

### 2A. SRT/VTT Export Include Translations (Issue #3)

- [ ] **2A-1** [P2] [US3] Add `display_text()` helper function in `src-tauri/src/commands/export.rs`
  - Function determines display text based on translation availability
  - When translation is present: returns `"{translation} ({original_text})"`
  - When no translation: returns `original_text.clone()`
  - Place after existing helpers (~line 22)
  - **FR-007, FR-008**

- [ ] **2A-2** [P2] [US3] Update `to_srt()` to use `display_text()` in `src-tauri/src/commands/export.rs`
  - Replace `entry.original_text` reference (line 69) with `display_text(&entry)`
  - **FR-007, FR-009**

- [ ] **2A-3** [P2] [US3] Update `to_vtt()` to use `display_text()` in `src-tauri/src/commands/export.rs`
  - Replace `entry.original_text` reference (line 86) with `display_text(&entry)`
  - **FR-008, FR-009**

- [ ] **2A-4** [P2] [US3] Update and add export tests in `src-tauri/src/commands/export.rs`
  - Update `test_srt_format` to assert SRT output contains translation text
  - Update `test_vtt_format` to assert VTT output contains translation text
  - Add new `test_srt_translation_only` test for translation-only behavior
  - **FR-007..FR-009**

### 2B. Shortcuts from Config (Issue #4)

- [ ] **2B-1** [P2] [US4] Add `parse_shortcut` helper function in `src-tauri/src/lib.rs`
  - Parse shortcut strings like `"Ctrl+Shift+S"` into `Shortcut` structs
  - Support modifiers: Ctrl, Shift, Alt, Super/Meta/Win
  - Support keys: S, O, T, H
  - Return `None` for invalid shortcuts and log warning
  - **FR-011**

- [ ] **2B-2** [P2] [US4] Replace hardcoded shortcut registration in `src-tauri/src/lib.rs`
  - Read `config.shortcuts` from `Arc<Mutex<AppConfig>>` state
  - Use `parse_shortcut` for each shortcut (`toggle_capture`, `toggle_overlay`, `toggle_translation`)
  - Fall back to default shortcuts if parsing fails
  - Remove the three hardcoded shortcut blocks (lines 205-227)
  - **FR-010, FR-012**

### 2C. HTTP Timeout for LibreTranslate (Issue #5)

- [ ] **2C-1** [P2] [US5] Add `timeout_seconds` field to `TranslationConfig` in `src-tauri/src/settings/config.rs`
  - Add `pub timeout_seconds: u64` to the struct (~line 51)
  - Default value: `10` in the `Default` impl (~line 101)
  - **FR-013, FR-014**

- [ ] **2C-2** [P2] [US5] Add timeout parameter to `translate()` in `src-tauri/src/translation/libretranslate.rs`
  - Add `timeout_secs: u64` parameter to the function signature
  - Use `reqwest::Client::builder().timeout(Duration::from_secs(timeout_secs.max(1)))`
  - Clamp minimum to 1 second (edge case: timeout of 0)
  - **FR-013, FR-015**

- [ ] **2C-3** [P2] [US5] Pass timeout config to translation call in `src-tauri/src/pipeline/transcriber.rs`
  - Capture `timeout_seconds` from config before async block (~line 56)
  - Pass to `libretranslate::translate()` call (~line 180)
  - **FR-014**

- [ ] **2C-4** [P2] [US5] Add `timeout_seconds` to TypeScript `AppConfig` interface in `src/hooks/useSettings.ts`
  - Add `timeout_seconds: number` to the `translation` section (~line 35)
  - **FR-014**

### 2D. Settings Config Deep-Merge (Issue #12)

- [ ] **2D-1** [P2] [US11] Add `deepMerge` utility function in `src/hooks/useSettings.ts`
  - Recursive deep merge for nested objects
  - Preserve all non-specified fields when merging partial updates
  - Handle edge cases: null, undefined, arrays (don't deep-merge arrays)
  - **FR-031**

- [ ] **2D-2** [P2] [US11] Update `updateConfig` to use `deepMerge` in `src/hooks/useSettings.ts`
  - Replace `{ ...config, ...updates }` (line 86) with `deepMerge(config, updates)`
  - **FR-031, FR-032**

### Phase 2 Verification Gate

```bash
cargo check && cargo test && bun run typecheck && bun run lint
```

---

## Phase 3: Architecture Cleanup (P3)

> **Goal**: Clean up translation dispatcher, engine constraints, show_original, and config hot-reload.
> **Dependencies**: Phase 2 complete (timeout_seconds field needed).
> **Files modified**: 3 files

### 3A. Translation Dispatcher Cleanup (Issue #6)

- [ ] **3A-1** [P3] [US6] Remove dead `TranslationConfig` struct and `TranslationEngine` enum in `src-tauri/src/translation/mod.rs`
  - Delete the `TranslationConfig` struct (lines 20-27)
  - Delete the `TranslationEngine` enum (lines 5-18)
  - Import `crate::settings::config::TranslationConfig` instead
  - **FR-019, FR-020**

- [ ] **3A-2** [P3] [US6] Update dispatcher `translate()` signature in `src-tauri/src/translation/mod.rs`
  - Accept `&settings::config::TranslationConfig` instead of individual params
  - Route to correct engine based on `config.engine` string
  - Call `libretranslate::translate()` with `config.timeout_seconds`
  - **FR-017, FR-018**

- [ ] **3A-3** [P3] [US6] Switch pipeline to use dispatcher in `src-tauri/src/pipeline/transcriber.rs`
  - Replace direct `crate::translation::libretranslate::translate()` call (~line 180)
  - Call `crate::translation::translate(&text, &config.translation)` instead
  - Clone config before spawn block to make it available inside async
  - **FR-017**

### 3B. Pipeline Respects show_original (Issue #7)

- [ ] **3B-1** [P3] [US7] Conditionally emit original text based on `show_original` in `src-tauri/src/pipeline/transcriber.rs`
  - Read `show_original` from config inside the spawn loop
  - When `show_original` is `false` and translation exists: set `display_text` to translation, set `translation` field to `None`
  - When `show_original` is `true`: keep existing behavior (emit both)
  - Update the `app_handle.emit("transcription", ...)` payload (~lines 214-221)
  - **FR-022, FR-023**

- [ ] **3B-2** [P3] [US7] Update history DB insert to respect `show_original` in `src-tauri/src/pipeline/transcriber.rs`
  - When `show_original` is `false`: insert translation as `original_text`, set `translation` column to `None`
  - When `show_original` is `true`: keep existing behavior
  - Update the `db.insert()` call (~lines 570-580)
  - **FR-022, FR-023**

### 3C. Whisper Engine Constraints (Issue #7)

- [ ] **3C-1** [P3] [US7] Add Whisper engine constraint warning in `src-tauri/src/pipeline/transcriber.rs`
  - Check if `config.translation.engine == "whisper"` AND `config.translation.target_lang != "en"`
  - Log warning via `tracing::warn!`
  - Emit `pipeline-warning` event to frontend with type `translation_engine_constraint`
  - Place check in `start()` method before the spawn block
  - **FR-021**

### 3D. Pipeline Config Hot-Reload (Issue #8)

- [ ] **3D-1** [P3] [US8] Change pipeline `start()` signature in `src-tauri/src/pipeline/transcriber.rs`
  - Accept `config: Arc<Mutex<AppConfig>>` instead of `config: AppConfig`
  - Read translation config dynamically inside the spawn loop (cheap clone of small struct)
  - **FR-024**

- [ ] **3D-2** [P3] [US8] Update pipeline call sites to pass `Arc<Mutex<AppConfig>>`
  - Update `src-tauri/src/commands/capture.rs` to pass `config_arc` to `pipeline.start()`
  - Update `src-tauri/src/lib.rs` if pipeline is initialized there
  - **FR-024, FR-025**

### Phase 3 Verification Gate

```bash
cargo check && cargo test && bun run typecheck
```

---

## Phase 4: Code Cleanup (P3)

> **Goal**: Remove dead code and redundant logic.
> **Dependencies**: Phase 3 complete.
> **Files modified**: 2 files (1 deleted)

### 4A. Remove Dead auto Logic in libretranslate.rs (Issue #10)

- [ ] **4A-1** [P3] [US9] Simplify redundant `source_lang == "auto"` check in `src-tauri/src/translation/libretranslate.rs`
  - Replace the if/else block (lines 38-42) with direct `source: source_lang.to_string()`
  - The conditional always produces the same result
  - **FR-026**

### 4B. Remove Dead Model Manager Code (Issue #13)

- [ ] **4B-1** [P3] [US12] Remove `manager` module from `src-tauri/src/models/mod.rs`
  - Remove `pub mod manager` declaration
  - Remove `pub use manager::ModelManager` re-export
  - Keep `pub mod downloader` and `pub use downloader::ModelDownloader`
  - **FR-033**

- [ ] **4B-2** [P3] [US12] Delete `src-tauri/src/models/manager.rs`
  - Remove the entire file (dead code)
  - Verify `whisper::model::ModelManager` remains the sole implementation
  - **FR-033, FR-034, FR-035**

### Phase 4 Verification Gate

```bash
cargo check && cargo test && cargo test --lib && bun run typecheck && bun run lint
```

---

## Final Verification (All Phases)

- [ ] **FV-1** [P3] Run full Rust test suite
  ```bash
  cargo test
  ```
  - All existing tests pass
  - No new warnings related to translation code

- [ ] **FV-2** [P3] Run TypeScript type checking
  ```bash
  bun run typecheck
  ```
  - No new TypeScript errors

- [ ] **FV-3** [P3] Run linting
  ```bash
  bun run lint
  ```
  - No new lint errors

- [ ] **FV-4** [P3] Run formatting check
  ```bash
  bun run fmt:check
  ```
  - All files properly formatted

---

## Task Summary

| Phase | Tasks | User Stories | FRs Covered |
|-------|-------|-------------|-------------|
| Phase 1: Critical Fixes | 7 tasks | US1, US2, US10 | FR-001..FR-006, FR-028..FR-030 |
| Phase 2: Moderate Fixes | 9 tasks | US3, US4, US5, US11 | FR-007..FR-016, FR-031..FR-032 |
| Phase 3: Architecture | 7 tasks | US6, US7, US8 | FR-017..FR-025 |
| Phase 4: Cleanup | 3 tasks | US9, US12 | FR-026, FR-033..FR-035 |
| **Total** | **26 tasks** | **8 user stories** | **35 FRs** |

---

## File Change Summary

| File | Phase | Tasks |
|------|-------|-------|
| `src-tauri/src/commands/history.rs` | 1 | 1A-1 |
| `src-tauri/src/lib.rs` | 1, 2 | 1A-2, 1C-2, 2B-1, 2B-2, 3D-2 |
| `src/App.tsx` | 1 | 1B-1, 1B-2, 1C-1 |
| `src/components/History/HistoryList.tsx` | 1 | 1A-3 |
| `src-tauri/src/commands/export.rs` | 2 | 2A-1, 2A-2, 2A-3, 2A-4 |
| `src-tauri/src/settings/config.rs` | 2 | 2C-1 |
| `src-tauri/src/translation/libretranslate.rs` | 2, 4 | 2C-2, 4A-1 |
| `src/hooks/useSettings.ts` | 2 | 2C-4, 2D-1, 2D-2 |
| `src-tauri/src/pipeline/transcriber.rs` | 2, 3 | 2C-3, 3A-3, 3B-1, 3B-2, 3C-1, 3D-1 |
| `src-tauri/src/translation/mod.rs` | 3 | 3A-1, 3A-2 |
| `src-tauri/src/commands/capture.rs` | 3 | 3D-2 |
| `src-tauri/src/models/mod.rs` | 4 | 4B-1 |
| `src-tauri/src/models/manager.rs` | 4 | 4B-2 (DELETE) |
