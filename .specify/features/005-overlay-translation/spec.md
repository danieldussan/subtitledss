# Feature Specification: Accumulative Overlay + Translation

**Feature Branch**: `feature/005-overlay-translation`

**Target Version**: `0.2.0`

**Base Branch**: `main`

**Created**: 2026-07-11

**Status**: Draft

**Input**: User description: "Improve overlay to accumulate transcriptions with gradual fade, and add translation support using Whisper translate (default) and LibreTranslate (alternative)"

## Branch Strategy

- All work happens on branch `feature/005-overlay-translation` (forked from `main`)
- No direct commits to `main` during development
- Feature is complete when:
  - All 39 tasks pass (`cargo test`, `bun run typecheck`)
  - Overlay accumulation + translation work end-to-end
  - Config migration from 0.1.x works correctly
- On completion: PR → `main`, bump version to `0.2.0` in `Cargo.toml` and `tauri.conf.json`
- Release tag: `v0.2.0`

## User Scenarios & Testing

### User Story 1 - Accumulative Subtitle Overlay (Priority: P1)

As a user watching live transcriptions, I want the overlay to accumulate multiple lines of text instead of replacing them, so that I can follow the conversation flow without losing context.

**Why this priority**: This is the core UX improvement. Without it, transcriptions disappear too fast and the user loses the thread of conversation. This is independently testable.

**Independent Test**: Start capture, speak several sentences. Verify that lines accumulate in the overlay, old lines gradually fade out over ~10s, and new lines appear at the bottom. Max 4 lines visible simultaneously.

**Acceptance Scenarios**:

1. **Given** capture is running, **When** a transcription chunk arrives, **Then** a new line appears at the bottom of the overlay with full opacity (1.0)
2. **Given** multiple lines are visible, **When** a new line arrives, **Then** older lines shift upward and a new line appears at the bottom
3. **Given** a line has been visible for 10 seconds, **When** 3 more seconds pass, **Then** the line fades from opacity 1.0 to 0.0 and is removed
4. **Given** more than 4 lines exist, **When** a new line arrives, **Then** the oldest line is immediately removed
5. **Given** the overlay is empty and no transcription arrives for 10s, **When** time passes, **Then** the overlay becomes fully transparent (auto-hide)

### User Story 2 - Whisper Translate (Default Engine) (Priority: P2)

As a user, I want to translate transcribed text to English automatically using Whisper's built-in translate mode, so that I can understand foreign language content without additional setup.

**Why this priority**: This is the simplest translation path. Whisper translate is offline, fast, and requires zero external dependencies. It only translates to English, which covers the most common use case.

**Independent Test**: Enable translation with engine=whisper, start capture in a foreign language. Verify the overlay shows translated text below the original.

**Acceptance Scenarios**:

1. **Given** translation is enabled with engine=whisper, **When** transcription produces non-English text, **Then** the overlay shows original text on top and English translation below
2. **Given** translation is enabled with engine=whisper, **When** transcription produces English text, **Then** only the original English text is shown (no redundant translation)
3. **Given** translation is enabled, **When** the user toggles it off via Ctrl+Shift+T, **Then** only original text is shown

### User Story 3 - LibreTranslate (Alternative Engine) (Priority: P3)

As a user, I want to translate transcribed text to any language (not just English) using LibreTranslate, so that I can get translations in my preferred target language.

**Why this priority**: LibreTranslate requires running a separate service, making it a secondary option. It adds multi-language support but adds complexity.

**Independent Test**: Set up LibreTranslate locally, configure engine=libretranslate with target_lang=es, start capture. Verify translations appear in Spanish.

**Acceptance Scenarios**:

1. **Given** engine=libretranslate is configured with target_lang=es, **When** transcription produces English text, **Then** the overlay shows original English on top and Spanish translation below
2. **Given** engine=libretranslate but the service is not running, **When** transcription arrives, **Then** original text is shown without translation and a warning appears in logs
3. **Given** engine=libretranslate, **When** the user changes target_lang in settings, **Then** subsequent translations use the new language

### User Story 4 - Translation Settings UI (Priority: P3)

As a user, I want a dedicated Translation tab in Settings to configure translation options, so that I can easily enable/disable and switch engines.

**Why this priority**: UI configuration makes the feature accessible to non-technical users.

**Independent Test**: Open Settings, navigate to Translation tab. Verify all controls work: engine toggle, language selectors, LibreTranslate URL input, show-original toggle.

**Acceptance Scenarios**:

1. **Given** user opens Settings, **When** they click the Translation tab, **Then** they see: enable/disable toggle, engine selector (Whisper/LibreTranslate), source language, target language, LibreTranslate URL (conditional), show-original toggle
2. **Given** translation settings are changed, **When** user clicks Save, **Then** settings are persisted to TOML config and applied to the running pipeline

### User Story 5 - Toggle Translation Shortcut (Priority: P3)

As a user, I want to toggle translation on/off with Ctrl+Shift+T from any window, so that I can quickly switch between original and translated view.

**Independent Test**: Press Ctrl+Shift+T while capture is running. Verify translation toggles without opening Settings.

**Acceptance Scenarios**:

1. **Given** capture is running with translation enabled, **When** user presses Ctrl+Shift+T, **Then** translation is disabled and overlay shows only original text
2. **Given** capture is running with translation disabled, **When** user presses Ctrl+Shift+T, **Then** translation is enabled and overlay shows translated text

### Edge Cases

- What happens when LibreTranslate service goes down mid-session? → Fall back to original text only, log warning
- What happens when Whisper translate fails on a chunk? → Show original text, skip translation for that chunk
- What happens when source and target language are the same? → Skip translation, show original only
- What happens when overlay text is very long (e.g., a full paragraph)? → Truncate with ellipsis at max-width, full text available in history

## Requirements

### Functional Requirements

- **FR-001**: System MUST accumulate transcription lines in the overlay instead of replacing them
- **FR-002**: System MUST display each line for a configurable duration (default 10s) before fading out over a configurable fade period (default 3s)
- **FR-003**: System MUST limit visible lines to a configurable maximum (default 4)
- **FR-004**: System MUST support Whisper translate engine (offline, translates to English only)
- **FR-005**: System MUST support LibreTranslate engine (multi-language, requires running service)
- **FR-006**: System MUST show original text and translation simultaneously in the overlay when translation is enabled
- **FR-007**: System MUST persist translation settings in TOML config file
- **FR-008**: System MUST support Ctrl+Shift+T shortcut to toggle translation on/off
- **FR-009**: System MUST gracefully handle LibreTranslate service unavailability
- **FR-010**: System MUST store transcription + translation in history database

### Key Entities

- **OverlayLine**: Represents a single subtitle line with text, translation, timestamp, opacity, and display state
- **TranslationConfig**: Settings for translation engine, languages, and overlay display preferences
- **OverlayDisplayConfig**: Settings for line duration, fade time, max lines, and gap

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can see at least 4 lines of accumulated transcription before oldest fades out
- **SC-002**: Lines remain visible for 8-12 seconds before starting to fade (configurable)
- **SC-003**: Translation adds less than 1 second of latency to the pipeline
- **SC-004**: Whisper translate works fully offline with no additional downloads
- **SC-005**: LibreTranslate integration handles service unavailability without crashing

## Assumptions

- Users have whisper-rs 0.16 which supports `set_translate()` method
- The overlay window height needs to increase from 80px to ~200px to accommodate multiple lines
- LibreTranslate runs as a separate service (Docker or binary) and is not bundled
- The existing `TranslationConfig` in `settings/config.rs` can be extended with additional fields
- The existing `reqwest` 0.12 dependency is sufficient for LibreTranslate HTTP calls
- The overlay HTML (`public/overlay.html`) needs to be rewritten from vanilla JS to support multi-line accumulation
- The `TranscriptionPipeline` needs to be modified to pass translation config and perform translation
