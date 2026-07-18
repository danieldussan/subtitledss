# Feature Specification: Translation Integration QA Fixes

**Feature Branch**: `006-translation-qa-fixes`

**Created**: 2026-07-13

**Status**: Draft

**Input**: User description: "Fix all 13 issues found in translation integration QA — 3 critical bugs, 3 moderate bugs, 4 design/code quality issues, 3 minor issues"

**Depends on**: Feature `007-candle-marian-translation` (replaces LibreTranslate/Whisper translate with candle + Marian MT). Issues #3 (SRT/VTT export), #5 (HTTP timeout), #6 (dispatcher architecture), #7 (show_original), #8 (Whisper constraints), #9 (hot-reload) are partially or fully superseded by 007. Issues #1 (delete command), #2 (state sync), #4 (shortcuts), #10 (dead auto logic), #11 (overlay toggle), #12 (deep-merge), #13 (dead model code) remain independent and should be implemented first.

## User Scenarios & Testing

### User Story 1 - History Entry Deletion Works (Priority: P1)

As a user reviewing my transcription history, I want to delete individual entries so that I can manage my saved transcriptions without encountering errors.

**Why this priority**: The trash icon is a primary UI affordance. Clicking it throws a runtime error that crashes the interaction, making history management completely non-functional. This is a critical bug.

**Independent Test**: Can be fully tested by opening the History tab, clicking the trash icon on any entry, and verifying the entry is removed without error.

**Acceptance Scenarios**:

1. **Given** I have at least one history entry, **When** I click the trash icon on that entry, **Then** the entry is removed from the list and the backend database, and no error is displayed.
2. **Given** I have multiple history entries, **When** I delete one entry, **Then** all other entries remain intact and the list re-renders correctly.
3. **Given** I click the trash icon, **When** the deletion succeeds, **Then** a success toast notification appears briefly.
4. **Given** I click the trash icon, **When** the deletion fails (e.g., database error), **Then** an error toast notification appears and the entry remains in the list.

---

### User Story 2 - Translation State Syncs on Startup (Priority: P1)

As a user who has enabled translation, I want the app to correctly reflect my translation preference when I restart it, so that the UI state matches the actual pipeline behavior.

**Why this priority**: State desync between frontend and backend means the UI shows translation as disabled while the pipeline is actually translating. This confuses users and creates a trust issue with the app's visual feedback.

**Independent Test**: Can be fully tested by enabling translation, restarting the app, and verifying the translate toggle matches the config state.

**Acceptance Scenarios**:

1. **Given** I have translation enabled in settings, **When** I restart the application, **Then** the translate toggle button visually shows "enabled" and matches the config state.
2. **Given** I have translation disabled in settings, **When** I restart the application, **Then** the translate toggle button visually shows "disabled".
3. **Given** I toggle translation on, **When** the config is saved, **Then** both the frontend state and backend config agree on `enabled: true`.
4. **Given** I toggle translation off, **When** the config is saved, **Then** both the frontend state and backend config agree on `enabled: false`.

---

### User Story 3 - SRT and VTT Exports Include Translations (Priority: P2)

As a user who has transcriptions with translations, I want exported SRT and VTT files to include the translated text so that I can use the subtitles in video players with the correct language.

**Why this priority**: Export is a core deliverable of the translation feature (Phase 3 roadmap). SRT/VTT are the most common subtitle formats. Missing translations makes the export feature incomplete.

**Independent Test**: Can be fully tested by enabling translation, transcribing audio, exporting as SRT/VTT, and verifying the file contains translated text.

**Acceptance Scenarios**:

1. **Given** I have a transcription with translation enabled and target language set to Spanish, **When** I export as SRT, **Then** the file contains the translated Spanish text (not just the original English text).
2. **Given** I have a transcription with translation enabled, **When** I export as VTT, **Then** the file contains the translated text in the VTT format.
3. **Given** I have a transcription with translation enabled, **When** I export as TXT, **Then** the file contains the translated text (existing behavior preserved).
4. **Given** I have a transcription with translation disabled, **When** I export as any format, **Then** the file contains the original text only (no change in behavior).
5. **Given** `show_original` is configured, **When** I export, **Then** the export format respects the `show_original` preference (show original only, translation only, or both).

---

### User Story 4 - Global Shortcuts Respect User Configuration (Priority: P2)

As a user who has customized my keyboard shortcuts in Settings, I want those shortcuts to take effect after restarting the app, so that I can use my preferred key bindings.

**Why this priority**: Shortcut customization is a Phase 2 roadmap feature. Hardcoded shortcuts make the Settings UI misleading — users see customizable fields but their changes have no effect.

**Independent Test**: Can be fully tested by changing a shortcut in Settings, restarting the app, and verifying the new shortcut works.

**Acceptance Scenarios**:

1. **Given** I change the "Toggle Overlay" shortcut to `Ctrl+Alt+S` in Settings, **When** I restart the application, **Then** pressing `Ctrl+Alt+S` toggles the overlay.
2. **Given** I change the "Toggle Overlay" shortcut to `Ctrl+Alt+S` in Settings, **When** I restart the application, **Then** the old shortcut `Ctrl+Shift+S` no longer works for that action.
3. **Given** I change shortcuts in Settings, **When** the app starts, **Then** all registered shortcuts match the values in the TOML config file.
4. **Given** I provide an invalid shortcut combination in Settings, **When** the app starts, **Then** it falls back to the default shortcut and logs a warning.

---

### User Story 5 - Translation HTTP Requests Have Timeout Protection (Priority: P2)

As a user relying on a LibreTranslate server, I want translation requests to time out gracefully if the server is unresponsive, so that my transcription pipeline is not blocked indefinitely.

**Why this priority**: A hung HTTP request blocks the entire transcription pipeline. Users experience the app as frozen with no feedback. This is a reliability issue.

**Independent Test**: Can be fully tested by pointing at an unresponsive server and verifying the pipeline continues after timeout with an error message.

**Acceptance Scenarios**:

1. **Given** LibreTranslate server is unreachable, **When** a translation request is made, **Then** the request times out after a configurable duration (default 10 seconds) and the pipeline continues with the original text.
2. **Given** LibreTranslate server is slow but responding, **When** a translation request takes longer than the timeout, **Then** the request is cancelled and the pipeline continues.
3. **Given** a translation timeout occurs, **When** the pipeline continues, **Then** a warning is logged and the user sees a toast notification indicating translation was skipped due to timeout.
4. **Given** the user configures a custom timeout in settings, **When** translation requests are made, **Then** the configured timeout is used instead of the default.

---

### User Story 6 - Translation Dispatcher and Config Architecture is Clean (Priority: P3)

As a developer maintaining this codebase, I want the translation module to use a clean dispatcher pattern with consistent config types, so that future changes to translation engines are straightforward and the codebase has no dead code.

**Why this priority**: Dead code and inconsistent types create maintenance burden and confusion for future contributors. This is a code quality improvement that prevents bugs.

**Independent Test**: Can be verified by running `cargo test` and confirming no unused code warnings, and by confirming the dispatcher is called from the pipeline.

**Acceptance Scenarios**:

1. **Given** the translation module, **When** the pipeline needs translation, **Then** it calls the dispatcher function (`translation/mod.rs::translate()`) rather than directly calling a specific engine.
2. **Given** the translation module, **When** the dispatcher is called with an engine identifier, **Then** it routes to the correct engine implementation (LibreTranslate, Whisper, etc.).
3. **Given** the translation module, **When** the code is compiled, **Then** there are no dead code warnings for `TranslationConfig` or the dispatcher function.
4. **Given** the translation module, **When** engine-specific config is needed, **Then** the code uses a consistent config type (`settings::config::TranslationConfig`) throughout.

---

### User Story 7 - Pipeline Respects show_original and Engine Constraints (Priority: P3)

As a user, I want the pipeline to respect my `show_original` configuration and the limitations of my chosen translation engine (e.g., Whisper only translates to English), so that I get the output I expect.

**Why this priority**: Incorrect engine behavior (Whisper translating to wrong language) silently produces wrong results. The `show_original` field being ignored means the frontend displays data the backend didn't filter.

**Independent Test**: Can be tested by configuring Whisper with a non-English target and verifying the pipeline warns/blocks, and by verifying `show_original` affects what data reaches the frontend.

**Acceptance Scenarios**:

1. **Given** I select Whisper as the translation engine with target language set to Spanish, **When** the pipeline starts, **Then** a warning is displayed and the pipeline either translates to English (Whisper's only mode) or falls back to LibreTranslate, and the user is informed of the limitation.
2. **Given** I select Whisper with target language English, **When** the pipeline runs, **Then** translation works correctly (Whisper's native behavior).
3. **Given** `show_original` is set to `false`, **When** the pipeline emits transcription data, **Then** only the translated text is emitted to the frontend (original text is omitted from the emitted event).
4. **Given** `show_original` is set to `true`, **When** the pipeline emits transcription data, **Then** both original and translated text are emitted to the frontend.

---

### User Story 8 - Pipeline Translation Config Updates Without Restart (Priority: P3)

As a user toggling translation on/off via keyboard shortcut, I want the pipeline to pick up the new config without requiring a full application restart, so that I can dynamically control translation during a live session.

**Why this priority**: The pipeline clones config once at start. This means shortcut-triggered toggles require a restart to take effect, which is a poor UX for a real-time application.

**Independent Test**: Can be tested by toggling translation via shortcut and verifying the next transcription chunk reflects the change immediately.

**Acceptance Scenarios**:

1. **Given** translation is initially disabled, **When** I press the toggle shortcut to enable it, **Then** the next transcription chunk includes a translation without restarting the app.
2. **Given** translation is initially enabled, **When** I press the toggle shortcut to disable it, **Then** the next transcription chunk does not include a translation without restarting.
3. **Given** I change the target language in Settings, **When** the config is saved, **Then** the next transcription chunk uses the new target language without restarting.
4. **Given** I change the translation engine in Settings, **When** the config is saved, **Then** the next transcription chunk uses the new engine without restarting.

---

### User Story 9 - Redundant Code and Dead Logic Cleaned Up (Priority: P3)

As a developer, I want redundant code paths and dead logic removed so that the codebase is maintainable and there are no confusing code paths that do nothing.

**Why this priority**: Minor cleanup. Does not affect user-facing behavior but reduces cognitive load for future development.

**Independent Test**: Verified by code review and confirming no behavioral change after removal.

**Acceptance Scenarios**:

1. **Given** the `source_lang == "auto"` check in `libretranslate.rs`, **When** the code is reviewed, **Then** the redundant if/else is replaced with a single assignment of `"auto"`.
2. **Given** the `translation/mod.rs` dispatcher, **When** it is confirmed to be called, **Then** it is no longer flagged as dead code.

---

### User Story 10 - Overlay Toggle Stays in Sync (Priority: P1)

As a user toggling the overlay via the system tray or global shortcut, I want the frontend overlay button to accurately reflect the actual overlay visibility state, so that I never see an inverted or stale toggle.

**Why this priority**: The overlay toggle discards the backend return value and uses optimistic state inversion. If the overlay is toggled from the system tray while the main window is open, the frontend state becomes inverted — clicking "Show" actually hides it and vice versa. This is a critical UI desync.

**Independent Test**: Can be tested by toggling overlay from system tray, then checking the button state in the main window.

**Acceptance Scenarios**:

1. **Given** the overlay is visible, **When** I toggle it off from the system tray, **Then** the main window's overlay button updates to show "hidden" state.
2. **Given** the overlay is hidden, **When** I toggle it on from the system tray, **Then** the main window's overlay button updates to show "visible" state.
3. **Given** I click the overlay toggle button in the main window, **When** the backend returns the new visibility state, **Then** the frontend uses the returned boolean instead of inverting the previous state.

---

### User Story 11 - Settings Config Updates Are Deep-Merged (Priority: P2)

As a developer using the `useSettings.updateConfig` helper, I want partial config updates to merge deeply with the existing config rather than replacing entire nested objects, so that calling `updateConfig({ translation: { enabled: true } })` doesn't wipe out `source_lang`, `target_lang`, and other translation fields.

**Why this priority**: The current shallow merge (`{ ...config, ...updates }`) means any partial update to a nested object (like `translation`) replaces all fields in that object. While currently safe because all callers construct full sub-objects, this is a latent bug that will cause data loss the moment any caller passes a partial update.

**Independent Test**: Can be verified by code review confirming all callers construct full sub-objects, and by adding a deep-merge utility that prevents future breakage.

**Acceptance Scenarios**:

1. **Given** the current config has `translation: { enabled: false, source_lang: "en", target_lang: "es" }`, **When** `updateConfig` is called with `{ translation: { enabled: true } }`, **Then** the resulting config preserves `source_lang: "en"` and `target_lang: "es"`.
2. **Given** the `updateConfig` function, **When** called with partial nested updates, **Then** it performs a recursive deep merge instead of a shallow spread.

---

### User Story 12 - Dead Model Manager Code Removed (Priority: P3)

As a developer maintaining this codebase, I want duplicate and dead `ModelManager`/`ModelInfo` structs removed, so that there is only one canonical implementation and no confusion about which to use.

**Why this priority**: Two separate `ModelManager` implementations exist (`whisper::model::ModelManager` which is used, and `models::manager::ModelManager` + `models::downloader::{ModelDownloader, ModelInfo}` which are dead code). This creates confusion and maintenance burden.

**Independent Test**: Verified by confirming the project compiles without the dead modules and `cargo test` passes.

**Acceptance Scenarios**:

1. **Given** the `models::manager` and `models::downloader` modules, **When** the dead code is removed, **Then** the project compiles and all tests pass.
2. **Given** the `whisper::model::ModelManager`, **When** the dead code is removed, **Then** it remains the sole implementation with no behavioral changes.

---

### Edge Cases

- What happens if the user deletes all history entries? The history list shows an empty state message.
- What happens if the translation server returns an empty string? The pipeline uses the original text as fallback.
- What happens if an invalid shortcut is registered? The app falls back to defaults and logs a warning.
- What happens if `show_original` is toggled mid-transcription? The next emitted chunk respects the new value.
- What happens if the pipeline config changes while a transcription is in-flight? The in-flight chunk completes with the old config; the next chunk picks up the new config.
- What happens if the user sets a timeout of 0 seconds? The system clamps it to a minimum of 1 second.
- What happens if the overlay is toggled from the system tray while the main window is focused? The main window's toggle button updates to reflect the actual state.
- What happens if `updateConfig` is called with an empty object? No change is made to the config.
- What happens if dead model modules are removed but some other module still references them? The build fails, indicating a missing dependency that must be resolved.

## Requirements

### Functional Requirements

**History Management (Issue #1)**

- **FR-001**: The system MUST implement a `delete_history_entry` Tauri command that accepts an `id` parameter and removes the corresponding entry from the SQLite database.
- **FR-002**: The `delete_history_entry` command MUST be registered in the `invoke_handler` in `src-tauri/src/lib.rs`.
- **FR-003**: The `delete_history_entry` command MUST return a success/failure result to the frontend.
- **FR-004**: The frontend `HistoryList.tsx` trash icon handler MUST handle both success and error responses from the `delete_history_entry` invocation.

**Translation State Sync (Issue #2)**

- **FR-005**: On application startup, the frontend MUST read the `translation.enabled` value from the TOML config and initialize the `translationEnabled` state accordingly.
- **FR-006**: The `useSettings` hook (or equivalent) MUST load config values before the first render to prevent state desync.

**Export with Translations (Issue #3)**

- **FR-007**: The `to_srt()` function in `src-tauri/src/commands/export.rs` MUST include translated text when available, following the same logic as `to_txt()`.
- **FR-008**: The `to_vtt()` function in `src-tauri/src/commands/export.rs` MUST include translated text when available, following the same logic as `to_txt()`.
- **FR-009**: The export functions MUST respect the `show_original` configuration when determining what text to include.

**Global Shortcuts (Issue #4)**

- **FR-010**: On application startup, global shortcuts MUST be registered using values from the `ShortcutsConfig` in the TOML config file, not hardcoded values.
- **FR-011**: If a configured shortcut is invalid or conflicts with a system shortcut, the system MUST fall back to the default shortcut and log a warning.
- **FR-012**: Changing shortcuts in Settings and restarting the app MUST result in the new shortcuts being active.

**HTTP Timeout (Issue #5)**

- **FR-013**: The LibreTranslate HTTP client MUST use a configurable timeout (default: 10 seconds) on all requests.
- **FR-014**: The timeout value MUST be configurable via the TOML settings file under the translation section.
- **FR-015**: On timeout, the translation request MUST be cancelled and the pipeline MUST continue with the original text.
- **FR-016**: On timeout, the system MUST log a warning and optionally display a toast notification.

**Translation Architecture (Issue #6)**

- **FR-017**: The pipeline MUST call the dispatcher function (`translation/mod.rs::translate()`) instead of directly calling engine-specific functions.
- **FR-018**: The dispatcher MUST route to the correct engine based on the configured engine identifier.
- **FR-019**: The `TranslationConfig` enum in `translation/mod.rs` MUST either be removed (if unused) or properly integrated, eliminating dead code.
- **FR-020**: The pipeline MUST use `settings::config::TranslationConfig` consistently for all config access.

**Engine Constraints and show_original (Issues #7, #8)**

- **FR-021**: When Whisper is selected as the translation engine with a non-English target language, the system MUST display a warning that Whisper translate mode only supports English output, and either refuse to start translation or fall back to an appropriate engine.
- **FR-022**: The `show_original` config field MUST be respected by the pipeline when emitting transcription data to the frontend.
- **FR-023**: When `show_original` is `false`, the pipeline MUST only emit translated text (not original text) in the transcription event payload.

**Pipeline Config Hot-Reload (Issue #9)**

- **FR-024**: The pipeline MUST read translation config dynamically (via shared state or config reload) rather than cloning it once at startup.
- **FR-025**: Changes to translation config (enabled, target language, engine) MUST take effect for the next transcription chunk without requiring an application restart.

**Code Cleanup (Issue #10)**

- **FR-026**: The redundant `if/else` on `source_lang == "auto"` in `libretranslate.rs` MUST be replaced with a direct assignment of `"auto"`.
- **FR-027**: All dead code paths identified in the QA MUST be removed or integrated.

**Overlay Toggle Desync (Issue #11)**

- **FR-028**: The `toggle_overlay` Tauri command MUST return the new visibility state as a boolean.
- **FR-029**: The frontend `toggleOverlay` function MUST use the returned boolean from `invoke("toggle_overlay")` to set the correct state, instead of optimistically inverting the previous state.
- **FR-030**: The overlay toggle button MUST accurately reflect the actual overlay visibility at all times, regardless of whether the toggle was triggered from the main window, system tray, or global shortcut.

**Settings Config Deep-Merge (Issue #12)**

- **FR-031**: The `useSettings.updateConfig` function MUST perform a recursive deep merge when applying partial config updates, preventing data loss on nested objects.
- **FR-032**: Calling `updateConfig` with a partial nested object (e.g., `{ translation: { enabled: true } }`) MUST preserve all other fields in that nested object.

**Dead Model Manager Code (Issue #13)**

- **FR-033**: The `models::manager` and `models::downloader` modules MUST be removed from the codebase if they contain unused code.
- **FR-034**: After removal, the `whisper::model::ModelManager` MUST remain the sole model management implementation.
- **FR-035**: The project MUST compile and all existing tests MUST pass after dead code removal.

### Key Entities

- **HistoryEntry**: A transcription record with `id`, `original_text`, `translated_text`, `timestamp`, `language`, `source` fields. Stored in SQLite.
- **TranslationConfig**: Settings for translation including `enabled` (bool), `engine` (string: "libretranslate"|"whisper"), `source_lang` (string), `target_lang` (string), `show_original` (bool), `timeout_seconds` (u64).
- **ShortcutsConfig**: User-configurable keyboard shortcuts for overlay toggle, transcription toggle, etc. Stored in TOML.
- **TranslationResult**: Output of the translation pipeline containing `text` (translated) and `source_lang` (detected or configured).

## Success Criteria

### Measurable Outcomes

- **SC-001**: Clicking the trash icon on any history entry successfully deletes it 100% of the time (0 runtime errors).
- **SC-002**: After restarting the app, the translate toggle visually matches the config state in 100% of cases.
- **SC-003**: Exported SRT/VTT files contain translated text when translation is enabled (verified by file content inspection).
- **SC-004**: Custom shortcuts configured in Settings are active after app restart (verified by pressing new key combo).
- **SC-005**: Translation requests to an unresponsive server time out within 10-15 seconds and the pipeline continues.
- **SC-006**: The pipeline respects `show_original` setting — emitted data matches the configured preference.
- **SC-007**: Whisper + non-English target produces a clear warning message to the user.
- **SC-008**: Toggling translation via shortcut takes effect on the next transcription chunk without restart.
- **SC-009**: `cargo test` passes with no new warnings related to translation code.
- **SC-010**: `bun run typecheck` passes with no new TypeScript errors.
- **SC-011**: Overlay toggle button always reflects actual overlay state, including after system tray toggles.
- **SC-012**: `updateConfig` with partial nested objects preserves all non-specified fields.
- **SC-013**: Dead `models::manager` and `models::downloader` modules are removed, project compiles, tests pass.

## Assumptions

- The `delete_history_entry` command should follow the same pattern as existing history commands (e.g., `get_history`, `clear_history`) using `sqlx` and `anyhow`.
- The translation timeout default of 10 seconds is reasonable for local LibreTranslate instances; network-hosted instances may need longer (user-configurable).
- The `show_original` behavior in the pipeline should match the existing frontend display logic in `HistoryList.tsx:299`.
- Whisper's translate mode limitation (English-only) is a whisper.cpp constraint, not something we can work around without switching engines.
- The pipeline config hot-reload can use `Arc<Mutex<TranslationConfig>>` or equivalent shared state pattern per project conventions.
- Shortcut validation should check for conflicts with common system shortcuts (e.g., `Ctrl+C`, `Alt+Tab`) but not all possible conflicts.
- The dispatcher pattern should use a simple match statement routing on engine name strings, consistent with the existing architecture.
