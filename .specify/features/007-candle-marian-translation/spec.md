# Feature Specification: In-Process Translation with Candle + Marian MT

**Feature Branch**: `007-candle-marian-translation`

**Created**: 2026-07-13

**Status**: Draft

**Input**: User description: "Replace LibreTranslate and Whisper translate with candle + Marian MT in-process translation. Remove external service dependency, support bidirectional EN<ES, achieve <200ms latency per segment on CPU."

## User Scenarios & Testing

### User Story 1 - Bidirectional Translation Without External Service (Priority: P1)

As a user, I want to translate transcribed text between English and Spanish without running any external service, so that translation works out of the box in a single binary.

**Why this priority**: This is the core replacement. The current system requires LibreTranslate running as a separate process, which is a friction point and a reliability risk. Whisper translate only supports English as target. Marian MT eliminates both problems.

**Independent Test**: Can be fully tested by enabling translation, setting source to English and target to Spanish, speaking English audio, and verifying the overlay shows Spanish translation — all without any external service running.

**Acceptance Scenarios**:

1. **Given** translation is enabled with source English and target Spanish, **When** English audio is transcribed, **Then** the pipeline produces both the original English text and a Spanish translation.
2. **Given** translation is enabled with source Spanish and target English, **When** Spanish audio is transcribed, **When** the pipeline produces both the original Spanish text and an English translation.
3. **Given** translation is enabled, **When** the app starts, **Then** the Marian MT models are loaded in-process within 3 seconds and translation is immediately available.
4. **Given** translation is enabled, **When** source and target language are the same, **Then** the text is shown as-is with no translation step (zero overhead).

---

### User Story 2 - Models Auto-Download on First Launch (Priority: P1)

As a user enabling translation for the first time, I want the required models to automatically download so I don't need to manually manage model files.

**Why this priority**: First-launch experience must be seamless. Users should not need to know about HuggingFace, model files, or manual setup. This matches the existing Whisper model download pattern.

**Independent Test**: Can be tested by deleting model files, enabling translation, restarting the app, and verifying a download progress indicator appears and models are downloaded.

**Acceptance Scenarios**:

1. **Given** no Marian MT models are downloaded, **When** translation is enabled and the app starts, **Then** both EN->ES and ES->EN models are downloaded automatically.
2. **Given** Marian MT models are downloading, **When** the user checks the Model Manager, **Then** download progress is displayed.
3. **Given** Marian MT models are already downloaded, **When** the app starts, **Then** no download occurs and models load directly from `~/.local/share/subtitledss/models/`.
4. **Given** a download fails due to network error, **When** the app retries, **Then** the download resumes or restarts from the beginning, and the user sees an error toast.
5. **Given** models are downloaded, **When** the user deletes them via Model Manager, **Then** they are removed from disk and will be re-downloaded on next use.

---

### User Story 3 - Translation Latency is Imperceptible (Priority: P1)

As a user relying on real-time subtitles, I want translation to add less than 200ms of latency per segment so that the overlay feels instantaneous.

**Why this priority**: Real-time performance is a core principle (Constitution II). If translation adds visible delay, users will disable it or perceive the app as broken. The 200ms target is aggressive but achievable with Marian models on CPU.

**Independent Test**: Can be tested by measuring end-to-end latency from audio chunk to overlay display with translation enabled, and verifying it stays under 200ms on a modern CPU.

**Acceptance Scenarios**:

1. **Given** translation is enabled, **When** a transcription segment is produced, **Then** the translated text appears in the overlay within 200ms.
2. **Given** translation is enabled, **When** the pipeline is running, **Then** CPU usage remains below 40% on a 4-core machine with Tiny/Base whisper model.
3. **Given** translation is enabled, **When** the pipeline is running, **Then** total RAM usage stays within 300MB of the non-translation baseline (models are ~150MB each).

---

### User Story 4 - Single Binary Distribution (Priority: P1)

As a user installing subtitledss, I want a single binary with no sidecar processes, so that installation and maintenance are trivial.

**Why this priority**: The Constitution mandates "single binary distribution" and Linux-native operation. LibreTranslate required a separate Docker container or Python server. Candle MT eliminates that entirely.

**Independent Test**: Can be tested by building the app, distributing it, and verifying translation works with zero additional processes or services.

**Acceptance Scenarios**:

1. **Given** the app is built, **When** the binary is distributed, **Then** no external processes, Docker containers, or sidecars are required for translation.
2. **Given** translation is enabled, **When** the app runs, **Then** the only processes visible are the Tauri app itself.
3. **Given** the user has no internet connection, **When** models are already downloaded, **Then** translation works fully offline.

---

### User Story 5 - Clean Removal of Legacy Translation Code (Priority: P2)

As a developer maintaining this codebase, I want all LibreTranslate and Whisper translate code removed, so that the codebase has no dead code, no unused dependencies, and a single translation engine path.

**Why this priority**: Dead code creates confusion and maintenance burden. The translation module should have one clear path: Marian MT. No dispatcher needed, no engine enum, no URL fields.

**Independent Test**: Can be verified by confirming `cargo build` compiles with no warnings, `cargo test` passes, and grep finds zero references to "libretranslate" or "whisper translate" in the Rust source.

**Acceptance Scenarios**:

1. **Given** the refactored codebase, **When** `cargo build` runs, **Then** there are no dead code warnings for translation-related modules.
2. **Given** the refactored codebase, **When** `cargo test` runs, **Then** all existing tests pass with no regressions.
3. **Given** the refactored codebase, **When** code is reviewed, **Then** the `TranslationEngine` enum, `libretranslate_url` config field, and `whisper translate` flag are gone.
4. **Given** the TOML config from a previous version with `engine = "whisper"` or `engine = "libretranslate"`, **When** the new version loads it, **Then** it migrates to `engine = "marian"` gracefully (or falls back to default).

---

### User Story 6 - Settings UI Reflects Marian MT (Priority: P2)

As a user configuring translation settings, I want the Settings UI to show Marian MT as the engine with no reference to LibreTranslate or Whisper translate, so the interface is accurate and uncluttered.

**Why this priority**: Users see the Settings UI. If it mentions removed engines, it's confusing. The UI must match the actual capabilities.

**Independent Test**: Can be tested by opening the Translation Settings tab and verifying: no engine selector (Marian is the only engine), no URL field, language selector shows EN<ES only.

**Acceptance Scenarios**:

1. **Given** the new UI, **When** I open Translation Settings, **Then** there is no engine selection (Marian MT is the only engine).
2. **Given** the new UI, **When** I open Translation Settings, **Then** the LibreTranslate URL field is gone.
3. **Given** the new UI, **When** I open Translation Settings, **Then** the source/target language selector is limited to English and Spanish.
4. **Given** the new UI, **When** translation is enabled, **Then** model status shows download/load state for Marian models.

---

### User Story 7 - Model Manager Shows Marian Models (Priority: P2)

As a user managing my models, I want the Model Manager to list Marian MT models alongside Whisper models, with download/delete/load actions.

**Why this priority**: Model Manager is the central place for model lifecycle. Marian models need the same treatment as Whisper models.

**Independent Test**: Can be tested by opening the Model Manager and verifying Marian models appear with correct names, sizes, and actions.

**Acceptance Scenarios**:

1. **Given** the Model Manager, **When** I view the model list, **Then** Marian MT models (EN->ES, ES->EN) appear in a separate section.
2. **Given** a Marian model is not downloaded, **When** I click Download, **Then** it downloads to `~/.local/share/subtitledss/models/` with progress indication.
3. **Given** a Marian model is downloaded, **When** I click Load, **Then** the model loads into the candle runtime and shows "Active" badge.
4. **Given** a Marian model is loaded, **When** I click Delete, **Then** a confirmation dialog appears and the model is removed from disk.

---

### User Story 8 - History and Export Preserve Translation (Priority: P3)

As a user reviewing history or exporting transcriptions, I want translated text to be stored and exported correctly, matching existing behavior.

**Why this priority**: History DB and export are already implemented (006-translation-qa-fixes). This story confirms they continue to work with the new engine.

**Independent Test**: Can be tested by transcribing with translation enabled, checking the History tab for translated text, and exporting as SRT/VTT/TXT with translations.

**Acceptance Scenarios**:

1. **Given** translation is enabled, **When** a transcription completes, **Then** the History entry stores both original and translated text.
2. **Given** history entries with translations, **When** I export as SRT, **Then** the file contains translated text.
3. **Given** `show_original` is false, **When** I export, **Then** only translated text is included.

---

### Edge Cases

- What happens if the Marian model file is corrupted on disk? The model loader detects checksum mismatch, deletes the file, and re-downloads on next use.
- What happens if translation is enabled but only one direction's model is available (e.g., EN->ES downloaded but ES->EN missing)? The system translates in the available direction and shows a warning for the missing direction.
- What happens if the text to translate is empty? The pipeline returns an empty string without invoking the model (zero overhead).
- What happens if the text exceeds the model's max input length (512 tokens for Marian)? The text is truncated to 512 tokens and a warning is logged.
- What happens if candle fails to initialize (e.g., incompatible CPU)? The app falls back to showing original text only and displays an error toast explaining the issue.
- What happens if the user switches source/target language mid-session? The pipeline picks up the new direction on the next chunk (no restart required).
- What happens if both whisper model and marian model are being loaded simultaneously? Loading is sequential (whisper first, then marian) to avoid peak RAM spikes.
- What happens if the user has an existing config with `engine = "libretranslate"`? The migration logic defaults to `engine = "marian"` and logs an info message.

## Requirements

### Functional Requirements

**Candle Marian MT Engine**

- **FR-001**: The system MUST implement a `MarianEngine` struct in `src-tauri/src/translation/marian.rs` that wraps candle-transformers Marian MT inference.
- **FR-002**: The `MarianEngine` MUST support bidirectional translation: EN->ES and ES->EN.
- **FR-003**: The `MarianEngine` MUST load models from `~/.local/share/subtitledss/models/` using the HuggingFace safetensors format.
- **FR-004**: The `MarianEngine` MUST tokenize input using the model's associated tokenizer (tokenizer.json bundled with the model).
- **FR-005**: The `MarianEngine.translate()` method MUST accept a text string and return the translated text synchronously (no async needed — candle inference is CPU-bound).
- **FR-006**: The `MarianEngine` MUST handle token sequences up to 512 tokens, truncating longer input and logging a warning.
- **FR-007**: The `MarianEngine` MUST be thread-safe (wrapped in `Arc<Mutex<>>` or `Arc<RwLock<>>`) for use from the async pipeline.

**Model Loading and Download**

- **FR-008**: The system MUST implement model download in `src-tauri/src/translation/model.rs` following the existing `ModelDownloader` pattern from `src-tauri/src/models/downloader.rs`.
- **FR-009**: Models MUST be downloaded from HuggingFace: `Helsinki-NLP/opus-mt-en-es` and `Helsinki-NLP/opus-mt-es-en`.
- **FR-010**: The download MUST include `model.safetensors`, `config.json`, `tokenizer.json`, and `generation_config.json`.
- **FR-011**: Downloaded models MUST be verified against expected file sizes (~300MB each for safetensors).
- **FR-012**: The Model Manager UI (`src/components/ModelManager/ModelList.tsx`) MUST display Marian models in a separate "Translation" section.
- **FR-013**: Marian models MUST support download, load, and delete actions identical to Whisper models.

**Pipeline Integration**

- **FR-014**: The pipeline (`src-tauri/src/pipeline/transcriber.rs`) MUST replace the LibreTranslate HTTP call with a synchronous call to `MarianEngine.translate()`.
- **FR-015**: The pipeline MUST remove the `translation_engine == "whisper"` branch entirely (no more `set_translate(true)` on whisper params).
- **FR-016**: The pipeline MUST use `source_lang` and `target_lang` from config to select the correct Marian model direction (EN->ES or ES->EN).
- **FR-017**: The pipeline MUST skip translation when `source_lang == target_lang`.
- **FR-018**: The pipeline MUST continue with original text if Marian translation fails, logging a warning.

**Configuration**

- **FR-019**: The `TranslationConfig` in `src-tauri/src/settings/config.rs` MUST remove the `libretranslate_url` field.
- **FR-020**: The `engine` field MUST be removed (Marian is the only engine) or retained as `"marian"` for forward compatibility.
- **FR-021**: The default `target_lang` MUST remain `"es"` (Spanish).
- **FR-022**: The `show_original` field MUST be preserved.
- **FR-023**: The system MUST handle legacy configs with `engine = "whisper"` or `engine = "libretranslate"` by migrating to `"marian"` and logging an info message.

**Whisper Params Cleanup**

- **FR-024**: The `translate` field in `TranscriptionParams` (`src-tauri/src/whisper/params.rs`) MUST be removed.
- **FR-025**: The `whisper_params.set_translate()` call in `WhisperEngine::transcribe()` (`src-tauri/src/whisper/engine.rs`) MUST be removed.
- **FR-026**: The `transcribe_audio` Tauri command (`src-tauri/src/commands/transcription.rs`) MUST remove the `translate: false` initialization.

**Cargo Dependencies**

- **FR-027**: `src-tauri/Cargo.toml` MUST add: `candle-core`, `candle-transformers`, `candle-nn`, `tokenizers`.
- **FR-028**: The existing `reqwest` dependency for LibreTranslate MUST be kept only if used elsewhere (e.g., model download), otherwise its features can be reduced.
- **FR-029**: The `sha2` and `hex` crates MUST remain for model checksum verification.

**Frontend**

- **FR-030**: `TranslationSettings.tsx` MUST remove the engine selector and LibreTranslate URL field.
- **FR-031**: `TranslationSettings.tsx` MUST limit language options to English and Spanish (source/target).
- **FR-032**: `TranslationSettings.tsx` MUST show Marian model status (downloaded, loaded, missing).
- **FR-033**: `useSettings.ts` hook MUST remove `libretranslate_url` from the `AppConfig` type.

**Files to Delete**

- **FR-034**: `src-tauri/src/translation/libretranslate.rs` MUST be deleted entirely.
- **FR-035**: The `pub mod libretranslate;` declaration in `src-tauri/src/translation/mod.rs` MUST be removed.

### Key Entities

- **MarianEngine**: In-process translation engine wrapping candle-transformers Marian MT. Holds loaded model state, tokenizer, and vocab. One instance per direction (EN->ES and ES->EN).
- **MarianModelInfo**: Metadata for a Marian model — name, direction (src->tgt), filename, download URL, expected size, download status.
- **TranslationConfig** (updated): Settings struct — `enabled` (bool), `source_lang` (String), `target_lang` (String), `show_original` (bool). No engine field, no URL field.
- **TranscriptionParams** (updated): Whisper params — `language` (Option<String>), `threads` (u32), `gpu` (bool). No `translate` field.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Translation latency (audio chunk to overlay display) is under 200ms for 90% of segments on a 4-core CPU with Base whisper model.
- **SC-002**: Marian model download completes in under 60 seconds on a 50 Mbps connection.
- **SC-003**: App cold start with translation enabled (models already downloaded) is under 3 seconds.
- **SC-004**: Total RAM with translation enabled is under 700MB (Whisper Base ~600MB + Marian ~100MB runtime overhead).
- **SC-005**: Zero references to "libretranslate" or "whisper translate" in Rust source after cleanup.
- **SC-006**: `cargo build` produces zero warnings related to translation code.
- **SC-007**: `cargo test` passes with no regressions.
- **SC-008**: `bun run typecheck` passes with no TypeScript errors.
- **SC-009**: EN->ES and ES->EN translation both work correctly (verified with sample audio in both languages).
- **SC-010**: Legacy config migration works — app loads existing TOML configs without error and defaults to Marian engine.

## Assumptions

- Helsinki-NLP/opus-mt-en-es and opus-mt-es-en models are compatible with candle-transformers (they are standard Marian encoder-decoder models with safetensors weights available on HuggingFace).
- Each Marian model is approximately 300MB in safetensors format. Two models (EN->ES, ES->EN) = ~600MB total download, ~100MB runtime memory each.
- The `tokenizers` crate from HuggingFace is compatible with Marian tokenizers (tokenizer.json format).
- CPU inference on a modern 4-core machine can achieve <200ms for typical subtitle segments (10-30 words).
- The existing `ModelDownloader` pattern (HTTP GET + write to disk) can be extended to download Marian model files.
- The `source_lang` and `target_lang` config values will be limited to `"en"` and `"es"` in v1 — additional language pairs can be added later by downloading more Marian models.
- The pipeline's existing architecture (lock engine, transcribe, lock for translation, emit) supports synchronous Marian inference without restructuring.
- candle-core supports the CPU backend required for Marian MT without GPU dependencies.
