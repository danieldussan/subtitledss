# Tasks: In-Process Translation with Candle + Marian MT

**Feature**: `007-candle-marian-translation`
**Generated**: 2026-07-13
**Spec**: `spec.md` | **Plan**: `plan.md`

---

## Phase 1: Core Candle Engine

> Goal: Implement `MarianEngine` and model loading infrastructure.
> Dependencies: None — foundation layer.

### 1A. Add Candle Dependencies to Cargo.toml

- [ ] [T-1A.1] Add `candle-core`, `candle-transformers`, `candle-nn`, and `tokenizers` to `[dependencies]` in `src-tauri/Cargo.toml`
  - **File**: `src-tauri/Cargo.toml`
  - **Spec Ref**: FR-027
  - **Verification**: `cargo check` compiles with new deps (no link errors)

### 1B. Create MarianEngine (`translation/marian.rs`)

- [ ] [T-1B.1] Create `src-tauri/src/translation/marian.rs` with `MarianEngine` struct and `TranslationDirection` enum
  - **File**: `src-tauri/src/translation/marian.rs`
  - **Spec Ref**: FR-001, FR-002, FR-007
  - **Details**: Struct holds `model: Option<marian::Model>`, `tokenizer: Option<Tokenizer>`, `device: Device`, `direction: TranslationDirection`. Enum variants: `EnToEs`, `EsToEn`.
  - **Verification**: `cargo check` passes; `MarianEngine::new()` compiles

- [ ] [T-1B.2] Implement `MarianEngine::load(model_dir)` — load tokenizer.json + model.safetensors from a directory
  - **File**: `src-tauri/src/translation/marian.rs`
  - **Spec Ref**: FR-003, FR-004
  - **Details**: Use `tokenizers::Tokenizer::from_file()` for tokenizer, `VarBuilder::from_safe_tensors()` for weights, initialize `marian::Model` from config.json. Store loaded state.
  - **Verification**: `cargo check` passes; unit test `load()` returns `Ok` with valid model dir

- [ ] [T-1B.3] Implement `MarianEngine::translate(text)` — synchronous translation with tokenization, truncation, and decoding
  - **File**: `src-tauri/src/translation/marian.rs`
  - **Spec Ref**: FR-005, FR-006
  - **Details**: Return empty string for empty input (zero overhead). Truncate to 512 tokens with warning log. Run `model.forward()`, decode output tokens. Return translated text.
  - **Verification**: `cargo check` passes; unit test `translate("")` returns `Ok("")`

- [ ] [T-1B.4] Add `is_loaded()` and `direction()` accessor methods
  - **File**: `src-tauri/src/translation/marian.rs`
  - **Spec Ref**: FR-001
  - **Verification**: `cargo check` passes

### 1C. Create Model Loader (`translation/model.rs`)

- [ ] [T-1C.1] Create `src-tauri/src/translation/model.rs` with `MarianModelInfo`, `MarianModelFile`, and `MarianModelManager` structs
  - **File**: `src-tauri/src/translation/model.rs`
  - **Spec Ref**: FR-008, FR-009, FR-010
  - **Details**: `MarianModelManager` holds `models_dir: PathBuf`. `available_models()` returns EN→ES and ES→EN models with repo IDs `Helsinki-NLP/opus-mt-en-es` and `Helsinki-NLP/opus-mt-es-en`. Files: `model.safetensors`, `tokenizer.json`, `config.json`, `generation_config.json`.
  - **Verification**: `cargo check` passes; unit test `available_models().len() == 2`

- [ ] [T-1C.2] Implement `MarianModelManager::download(model_name)` — async download from HuggingFace
  - **File**: `src-tauri/src/translation/model.rs`
  - **Spec Ref**: FR-008, FR-011
  - **Details**: Follow existing `ModelDownloader` pattern. Create subdirectory `models_dir/<model_name>/`. For each file: skip if exists, download from `https://huggingface.co/<repo_id>/resolve/main/<filename>`, verify size. Return model dir path.
  - **Verification**: `cargo check` passes; integration test `download()` + `is_downloaded()` (mark `#[ignore]` for network)

- [ ] [T-1C.3] Implement `MarianModelManager::is_downloaded(model_name)` — check all files present
  - **File**: `src-tauri/src/translation/model.rs`
  - **Spec Ref**: FR-011
  - **Verification**: Unit test `is_downloaded()` returns false for missing model, true after mock download

- [ ] [T-1C.4] Implement `MarianModelManager::delete(model_name)` — remove model directory from disk
  - **File**: `src-tauri/src/translation/model.rs`
  - **Spec Ref**: FR-013
  - **Verification**: Unit test `delete()` removes directory

- [ ] [T-1C.5] Implement `MarianModelManager::verify(model_name)` — checksum validation for downloaded files
  - **File**: `src-tauri/src/translation/model.rs`
  - **Spec Ref**: FR-011
  - **Verification**: Unit test `verify()` returns false for corrupted/missing model

- [ ] [T-1C.6] Implement `MarianModelManager::model_dir(model_name)` — return path to model directory
  - **File**: `src-tauri/src/translation/model.rs`
  - **Spec Ref**: FR-003
  - **Verification**: Unit test returns `models_dir.join(model_name)`

### Phase 1 Verification Gate

```bash
cargo check
cargo test
```

---

## Phase 2: Pipeline Integration

> Goal: Wire MarianEngine into the transcription pipeline and clean up whisper/LibreTranslate code.
> Dependencies: Phase 1 complete.

### 2A. Rewrite `translation/mod.rs`

- [ ] [T-2A.1] Remove `pub mod libretranslate;`, `TranslationEngine` enum, old `TranslationConfig` struct, and old `translate()` async function from `src-tauri/src/translation/mod.rs`
  - **File**: `src-tauri/src/translation/mod.rs`
  - **Spec Ref**: FR-035
  - **Verification**: `cargo check` compiles (will have errors until 2A.2)

- [ ] [T-2A.2] Rewrite `src-tauri/src/translation/mod.rs` with new `TranslationEngine` struct that holds two `Arc<Mutex<MarianEngine>>` (EN→ES and ES→EN), plus `load_models()`, `translate()`, and `is_loaded()` methods
  - **File**: `src-tauri/src/translation/mod.rs`
  - **Spec Ref**: FR-001, FR-002, FR-007, FR-016, FR-017
  - **Details**: Import from `marian` and `model` submodules. Route translation by `(source_lang, target_lang)` pair. Skip when `source_lang == target_lang`. Log warning for unsupported pairs and return original text.
  - **Verification**: `cargo check` passes; `TranslationEngine::new()` compiles

### 2B. Update Settings Config (`settings/config.rs`)

- [ ] [T-2B.1] Remove `engine` and `libretranslate_url` fields from `TranslationConfig` in `src-tauri/src/settings/config.rs`
  - **File**: `src-tauri/src/settings/config.rs`
  - **Spec Ref**: FR-019, FR-020, FR-022
  - **Verification**: `cargo check` passes; `cargo test` — existing config tests updated

- [ ] [T-2B.2] Update `Default` impl for `TranslationConfig` — remove `engine` and `libretranslate_url` defaults
  - **File**: `src-tauri/src/settings/config.rs`
  - **Spec Ref**: FR-021
  - **Verification**: `cargo test` — `test_default_translation_config` passes with new defaults

- [ ] [T-2B.3] Add config migration logic in `AppConfig::load()` to handle legacy TOML with `engine` and `libretranslate_url` fields
  - **File**: `src-tauri/src/settings/config.rs`
  - **Spec Ref**: FR-023
  - **Details**: Use intermediate `LegacyTranslationConfig` struct with `Option<>` fields. Try new format first, fall back to legacy parse and migrate. Log info message on migration.
  - **Verification**: `cargo test` — new test `test_legacy_config_migration` passes

- [ ] [T-2B.4] Update existing config tests: remove assertions on `engine` and `libretranslate_url`, update `test_deserialize_from_toml` to remove legacy fields
  - **File**: `src-tauri/src/settings/config.rs`
  - **Spec Ref**: FR-035
  - **Verification**: `cargo test` — all config tests pass

### 2C. Update Pipeline (`pipeline/transcriber.rs`)

- [ ] [T-2C.1] Add `translation_engine: Arc<Mutex<crate::translation::TranslationEngine>>` parameter to `TranscriptionPipeline::start()`
  - **File**: `src-tauri/src/pipeline/transcriber.rs`
  - **Spec Ref**: FR-014
  - **Verification**: `cargo check` passes (will have errors until 2C.2-2C.4)

- [ ] [T-2C.2] Remove variable captures for `translation_engine` (string) and `libretranslate_url` from `start()` (lines 53-56)
  - **File**: `src-tauri/src/pipeline/transcriber.rs`
  - **Spec Ref**: FR-019
  - **Verification**: `cargo check` passes

- [ ] [T-2C.3] Remove `use_translate` logic and `translate: use_translate` from `TranscriptionParams` (lines 142, 149)
  - **File**: `src-tauri/src/pipeline/transcriber.rs`
  - **Spec Ref**: FR-015, FR-024
  - **Verification**: `cargo check` passes

- [ ] [T-2C.4] Replace LibreTranslate HTTP call (lines 179-197) with synchronous MarianEngine `translate()` call via `translation_engine` lock
  - **File**: `src-tauri/src/pipeline/transcriber.rs`
  - **Spec Ref**: FR-014, FR-016, FR-017, FR-018
  - **Details**: Lock `translation_engine`, call `translate(&text, &config.translation)`, handle `Ok`/`Err`. On error, log warning and continue with original text (no translation).
  - **Verification**: `cargo check` passes; `cargo test` passes

### 2D. Clean Whisper Params

- [ ] [T-2D.1] Remove `translate` field from `TranscriptionParams` struct and its `Default` impl in `src-tauri/src/whisper/params.rs`
  - **File**: `src-tauri/src/whisper/params.rs`
  - **Spec Ref**: FR-024
  - **Verification**: `cargo check` passes; `grep -r "translate" src-tauri/src/whisper/params.rs` returns no matches

- [ ] [T-2D.2] Remove `whisper_params.set_translate(params.translate)` call in `WhisperEngine::transcribe()` in `src-tauri/src/whisper/engine.rs`
  - **File**: `src-tauri/src/whisper/engine.rs`
  - **Spec Ref**: FR-025
  - **Verification**: `cargo check` passes; `grep -r "set_translate" src-tauri/src/whisper/engine.rs` returns no matches

- [ ] [T-2D.3] Remove `translate: false` from `TranscriptionParams` in `transcribe_audio` command in `src-tauri/src/commands/transcription.rs`
  - **File**: `src-tauri/src/commands/transcription.rs`
  - **Spec Ref**: FR-026
  - **Verification**: `cargo check` passes

### 2E. Wire TranslationEngine in `lib.rs`

- [ ] [T-2E.1] Create `Arc<Mutex<TranslationEngine>>` in `lib.rs::run()`, load Marian models if translation is enabled, register as Tauri managed state
  - **File**: `src-tauri/src/lib.rs`
  - **Spec Ref**: FR-003, FR-007
  - **Details**: After creating other states, create `translation_engine`. If `config.translation.enabled`, load models from `~/.local/share/subtitledss/models/`. Register with `app.manage()`.
  - **Verification**: `cargo check` passes

- [ ] [T-2E.2] Update `commands/capture.rs` to accept `translation_engine` state and pass it to `pipeline.start()`
  - **File**: `src-tauri/src/commands/capture.rs`
  - **Spec Ref**: FR-014
  - **Verification**: `cargo check` passes; `cargo test` passes

- [ ] [T-2E.3] Register new Marian model commands in `lib.rs` invoke handler (see Phase 4C for command impl)
  - **File**: `src-tauri/src/lib.rs`
  - **Spec Ref**: FR-012, FR-013
  - **Verification**: `cargo check` passes (commands added in Phase 4C)

### Phase 2 Verification Gate

```bash
cargo check
cargo test
bun run typecheck
bun run lint
```

---

## Phase 3: Frontend

> Goal: Update Settings UI, Model Manager, and hooks to reflect Marian MT.
> Dependencies: Phase 2 complete.

### 3A. Update `useSettings` Hook

- [ ] [T-3A.1] Remove `engine` and `libretranslate_url` from `AppConfig.translation` type in `src/hooks/useSettings.ts`
  - **File**: `src/hooks/useSettings.ts`
  - **Spec Ref**: FR-033
  - **Verification**: `bun run typecheck` passes

### 3B. Rewrite TranslationSettings Component

- [ ] [T-3B.1] Remove engine selector section (lines 97-127) from `src/components/Settings/TranslationSettings.tsx`
  - **File**: `src/components/Settings/TranslationSettings.tsx`
  - **Spec Ref**: FR-030
  - **Verification**: `bun run typecheck` passes

- [ ] [T-3B.2] Remove LibreTranslate URL section (lines 176-192) from `src/components/Settings/TranslationSettings.tsx`
  - **File**: `src/components/Settings/TranslationSettings.tsx`
  - **Spec Ref**: FR-030
  - **Verification**: `bun run typecheck` passes

- [ ] [T-3B.3] Remove Whisper translate warning (lines 166-171) from `src/components/Settings/TranslationSettings.tsx`
  - **File**: `src/components/Settings/TranslationSettings.tsx`
  - **Spec Ref**: FR-030
  - **Verification**: `bun run typecheck` passes

- [ ] [T-3B.4] Limit language options to English and Spanish only (remove Auto-detect and other languages)
  - **File**: `src/components/Settings/TranslationSettings.tsx`
  - **Spec Ref**: FR-031
  - **Details**: Change `languages` array to only `{ value: "en", label: "English" }` and `{ value: "es", label: "Español" }`. Remove auto-detect from source selector.
  - **Verification**: `bun run typecheck` passes

- [ ] [T-3B.5] Update component state and `handleSave` to remove `engine` and `libretranslateUrl` state variables
  - **File**: `src/components/Settings/TranslationSettings.tsx`
  - **Spec Ref**: FR-030, FR-033
  - **Verification**: `bun run typecheck` passes

- [ ] [T-3B.6] Add Marian model status display section when translation is enabled — show download/load state for EN→ES and ES→EN models
  - **File**: `src/components/Settings/TranslationSettings.tsx`
  - **Spec Ref**: FR-032
  - **Details**: Add `ModelStatusCard` sub-component showing model name, direction, download status, and download/delete buttons. Invoke `list_downloaded_marian_models` to check status.
  - **Verification**: `bun run typecheck` passes; manual: Open Translation Settings → no engine selector, no URL field, languages limited to EN/ES

### 3C. Update Model Manager (`ModelList.tsx`)

- [ ] [T-3C.1] Add Marian model entries to `ModelList.tsx` — `marian-en-es` and `marian-es-en` in a new "Translation Models" section
  - **File**: `src/components/ModelManager/ModelList.tsx`
  - **Spec Ref**: FR-012
  - **Details**: Add `marianModels` array with name, direction, size (~300MB each). Render a separate section header "Translation Models" after Whisper models.
  - **Verification**: `bun run typecheck` passes

- [ ] [T-3C.2] Update `checkDownloadedModels` to also query `list_downloaded_marian_models` and merge results
  - **File**: `src/components/ModelManager/ModelList.tsx`
  - **Spec Ref**: FR-013
  - **Verification**: `bun run typecheck` passes

- [ ] [T-3C.3] Update `handleDownload` to route Marian model names (`marian-*`) to `download_marian_model` command
  - **File**: `src/components/ModelManager/ModelList.tsx`
  - **Spec Ref**: FR-013
  - **Verification**: `bun run typecheck` passes

- [ ] [T-3C.4] Update `handleLoad` to route Marian model names to `load_marian_models` command
  - **File**: `src/components/ModelManager/ModelList.tsx`
  - **Spec Ref**: FR-013
  - **Verification**: `bun run typecheck` passes

- [ ] [T-3C.5] Update `handleDelete` to route Marian model names to `delete_marian_model` command
  - **File**: `src/components/ModelManager/ModelList.tsx`
  - **Spec Ref**: FR-013
  - **Verification**: `bun run typecheck` passes

### Phase 3 Verification Gate

```bash
cargo check
bun run typecheck
bun run lint
```

---

## Phase 4: Legacy Removal & Polish

> Goal: Delete dead code, add Marian model commands, add unit tests, verify clean build.
> Dependencies: Phase 3 complete.

### 4A. Delete LibreTranslate Module

- [ ] [T-4A.1] Delete `src-tauri/src/translation/libretranslate.rs`
  - **File**: `src-tauri/src/translation/libretranslate.rs`
  - **Spec Ref**: FR-034
  - **Verification**: `cargo check` passes; `grep -r "libretranslate" src-tauri/src/` returns no matches (except config migration comments)

- [ ] [T-4A.2] Remove `pub mod libretranslate;` declaration from `src-tauri/src/translation/mod.rs` (if not already removed in 2A.1)
  - **File**: `src-tauri/src/translation/mod.rs`
  - **Spec Ref**: FR-035
  - **Verification**: `cargo check` passes

### 4B. Verify Dead Code Removal

- [ ] [T-4B.1] Verify `libretranslate_url` is gone from all Rust and TypeScript source files
  - **Files**: `src-tauri/src/settings/config.rs`, `src/hooks/useSettings.ts`, `src/components/Settings/TranslationSettings.tsx`, `src-tauri/src/pipeline/transcriber.rs`
  - **Spec Ref**: FR-019, FR-033
  - **Verification**: `grep -r "libretranslate_url" src-tauri/src/ src/` returns no matches

- [ ] [T-4B.2] Verify `engine` field is gone from `TranslationConfig` and related code
  - **Files**: `src-tauri/src/settings/config.rs`, `src/hooks/useSettings.ts`, `src/components/Settings/TranslationSettings.tsx`
  - **Spec Ref**: FR-020
  - **Verification**: `grep -r '"engine"' src-tauri/src/settings/config.rs` returns no matches (except migration)

- [ ] [T-4B.3] Verify `translate` field is gone from whisper params and related code
  - **Files**: `src-tauri/src/whisper/params.rs`, `src-tauri/src/whisper/engine.rs`, `src-tauri/src/commands/transcription.rs`, `src-tauri/src/pipeline/transcriber.rs`
  - **Spec Ref**: FR-024, FR-025, FR-026
  - **Verification**: `grep -r "\.translate" src-tauri/src/whisper/` returns no matches

- [ ] [T-4B.4] Verify `TranslationEngine` enum and old `TranslationConfig` struct are gone from `translation/mod.rs`
  - **File**: `src-tauri/src/translation/mod.rs`
  - **Spec Ref**: FR-035
  - **Verification**: `grep -r "TranslationEngine" src-tauri/src/translation/mod.rs` shows only the new struct

### 4C. Add Marian Model Tauri Commands

- [ ] [T-4C.1] Add `download_marian_model` command to `src-tauri/src/commands/models.rs`
  - **File**: `src-tauri/src/commands/models.rs`
  - **Spec Ref**: FR-013
  - **Details**: Accept `model_name: String`, lock `MarianModelManager`, call `download()`, return success message.
  - **Verification**: `cargo check` passes

- [ ] [T-4C.2] Add `list_downloaded_marian_models` command to `src-tauri/src/commands/models.rs`
  - **File**: `src-tauri/src/commands/models.rs`
  - **Spec Ref**: FR-013
  - **Details**: Lock `MarianModelManager`, iterate `models_dir`, return list of downloaded Marian model names.
  - **Verification**: `cargo check` passes

- [ ] [T-4C.3] Add `delete_marian_model` command to `src-tauri/src/commands/models.rs`
  - **File**: `src-tauri/src/commands/models.rs`
  - **Spec Ref**: FR-013
  - **Details**: Accept `model_name: String`, lock `MarianModelManager`, call `delete()`, return success message.
  - **Verification**: `cargo check` passes

- [ ] [T-4C.4] Add `load_marian_models` command to `src-tauri/src/commands/models.rs`
  - **File**: `src-tauri/src/commands/models.rs`
  - **Spec Ref**: FR-013
  - **Details**: Accept state for `TranslationEngine` and `MarianModelManager`, get models dir, lock engine, call `load_models()`.
  - **Verification**: `cargo check` passes

- [ ] [T-4C.5] Register all four Marian commands in `lib.rs` invoke handler
  - **File**: `src-tauri/src/lib.rs`
  - **Spec Ref**: FR-013
  - **Verification**: `cargo check` passes

- [ ] [T-4C.6] Create `MarianModelManager` state in `lib.rs::run()` and register as Tauri managed state
  - **File**: `src-tauri/src/lib.rs`
  - **Spec Ref**: FR-013
  - **Details**: Create `Arc<Mutex<MarianModelManager>>` with models dir, register with `app.manage()`.
  - **Verification**: `cargo check` passes

### 4D. Add Unit Tests

- [ ] [T-4D.1] Add unit tests to `src-tauri/src/translation/marian.rs` — `test_marian_engine_new`, `test_translate_empty_string`, `test_translation_direction`
  - **File**: `src-tauri/src/translation/marian.rs`
  - **Spec Ref**: V-001
  - **Verification**: `cargo test --lib translation` passes

- [ ] [T-4D.2] Add unit tests to `src-tauri/src/translation/model.rs` — `test_available_models`, `test_is_downloaded_false`, `test_model_dir`
  - **File**: `src-tauri/src/translation/model.rs`
  - **Spec Ref**: V-001
  - **Verification**: `cargo test --lib translation` passes

- [ ] [T-4D.3] Add unit tests to `src-tauri/src/translation/mod.rs` — `test_translation_engine_new`, `test_translate_empty`, `test_translate_same_lang`
  - **File**: `src-tauri/src/translation/mod.rs`
  - **Spec Ref**: V-001
  - **Verification**: `cargo test --lib translation` passes

### 4E. Config Migration Tests

- [ ] [T-4E.1] Add test `test_legacy_config_migration` in `src-tauri/src/settings/config.rs` — parse TOML with `engine="whisper"` and `libretranslate_url`, verify migration to new format
  - **File**: `src-tauri/src/settings/config.rs`
  - **Spec Ref**: FR-023, SC-010
  - **Verification**: `cargo test` passes

- [ ] [T-4E.2] Add test `test_legacy_config_libretranslate_migration` — parse TOML with `engine="libretranslate"`, verify migration
  - **File**: `src-tauri/src/settings/config.rs`
  - **Spec Ref**: FR-023
  - **Verification**: `cargo test` passes

### Phase 4 Verification Gate

```bash
cargo check
cargo test
bun run typecheck
bun run lint
grep -r "libretranslate" src-tauri/src/ src/   # should return only migration comments
grep -r "whisper.*translate" src-tauri/src/     # should return nothing
```

---

## Dependency Graph

```
Phase 1 (Core Engine)
  ├─ T-1A.1: Cargo.toml deps          ─── standalone
  ├─ T-1B.1: marian.rs struct          ─── depends on T-1A.1
  ├─ T-1B.2: marian.rs load()          ─── depends on T-1B.1
  ├─ T-1B.3: marian.rs translate()     ─── depends on T-1B.2
  ├─ T-1B.4: marian.rs accessors       ─── depends on T-1B.1
  ├─ T-1C.1: model.rs structs          ─── depends on T-1A.1
  ├─ T-1C.2: model.rs download()       ─── depends on T-1C.1
  ├─ T-1C.3: model.rs is_downloaded()  ─── depends on T-1C.1
  ├─ T-1C.4: model.rs delete()         ─── depends on T-1C.1
  ├─ T-1C.5: model.rs verify()         ─── depends on T-1C.1
  └─ T-1C.6: model.rs model_dir()      ─── depends on T-1C.1

Phase 2 (Pipeline Integration)
  ├─ T-2A.1: mod.rs remove old code    ─── depends on T-1B.1, T-1C.1
  ├─ T-2A.2: mod.rs rewrite            ─── depends on T-2A.1, T-1B.2, T-1C.1
  ├─ T-2B.1: config.rs remove fields   ─── standalone
  ├─ T-2B.2: config.rs update defaults ─── depends on T-2B.1
  ├─ T-2B.3: config.rs migration       ─── depends on T-2B.1
  ├─ T-2B.4: config.rs update tests    ─── depends on T-2B.1, T-2B.2, T-2B.3
  ├─ T-2C.1: transcriber.rs add param  ─── depends on T-2A.2
  ├─ T-2C.2: transcriber.rs remove captures ─── depends on T-2C.1
  ├─ T-2C.3: transcriber.rs remove translate ─── depends on T-2C.1
  ├─ T-2C.4: transcriber.rs Marian call ─── depends on T-2C.1, T-2C.2, T-2C.3
  ├─ T-2D.1: params.rs remove field    ─── depends on T-2C.3
  ├─ T-2D.2: engine.rs remove set_translate ─── depends on T-2D.1
  ├─ T-2D.3: transcription.rs remove field ─── depends on T-2D.1
  ├─ T-2E.1: lib.rs wire engine        ─── depends on T-2A.2
  ├─ T-2E.2: capture.rs update         ─── depends on T-2E.1, T-2C.1
  └─ T-2E.3: lib.rs register commands  ─── depends on T-4C.5

Phase 3 (Frontend)
  ├─ T-3A.1: useSettings.ts type       ─── depends on T-2B.1
  ├─ T-3B.1: TranslationSettings remove engine ─── depends on T-3A.1
  ├─ T-3B.2: TranslationSettings remove URL    ─── depends on T-3A.1
  ├─ T-3B.3: TranslationSettings remove warning ─── depends on T-3A.1
  ├─ T-3B.4: TranslationSettings limit langs   ─── depends on T-3A.1
  ├─ T-3B.5: TranslationSettings update state   ─── depends on T-3B.1, T-3B.2
  ├─ T-3B.6: TranslationSettings model status   ─── depends on T-4C.2
  ├─ T-3C.1: ModelList add marian entries       ─── standalone
  ├─ T-3C.2: ModelList update checkDownloaded   ─── depends on T-4C.2
  ├─ T-3C.3: ModelList route download           ─── depends on T-4C.1
  ├─ T-3C.4: ModelList route load               ─── depends on T-4C.4
  └─ T-3C.5: ModelList route delete             ─── depends on T-4C.3

Phase 4 (Legacy Removal & Polish)
  ├─ T-4A.1: delete libretranslate.rs  ─── depends on T-2A.2
  ├─ T-4A.2: remove mod declaration    ─── depends on T-4A.1
  ├─ T-4B.1: verify libretranslate_url gone ─── depends on all above
  ├─ T-4B.2: verify engine field gone       ─── depends on all above
  ├─ T-4B.3: verify translate field gone    ─── depends on all above
  ├─ T-4B.4: verify old enum gone           ─── depends on all above
  ├─ T-4C.1: download_marian_model command  ─── depends on T-1C.2
  ├─ T-4C.2: list_downloaded_marian_models  ─── depends on T-1C.3
  ├─ T-4C.3: delete_marian_model command    ─── depends on T-1C.4
  ├─ T-4C.4: load_marian_models command     ─── depends on T-2A.2
  ├─ T-4C.5: register commands in lib.rs    ─── depends on T-4C.1-4C.4
  ├─ T-4C.6: MarianModelManager state       ─── depends on T-1C.1
  ├─ T-4D.1: marian.rs unit tests           ─── depends on T-1B.1-1B.4
  ├─ T-4D.2: model.rs unit tests            ─── depends on T-1C.1-1C.6
  ├─ T-4D.3: mod.rs unit tests              ─── depends on T-2A.2
  ├─ T-4E.1: config migration test          ─── depends on T-2B.3
  └─ T-4E.2: config libretranslate migration test ─── depends on T-2B.3
```

---

## Task Summary

| Phase | Tasks | Key Files |
|-------|-------|-----------|
| **Phase 1** | 11 tasks | `Cargo.toml`, `translation/marian.rs`, `translation/model.rs` |
| **Phase 2** | 16 tasks | `translation/mod.rs`, `settings/config.rs`, `pipeline/transcriber.rs`, `whisper/params.rs`, `whisper/engine.rs`, `commands/transcription.rs`, `commands/capture.rs`, `lib.rs` |
| **Phase 3** | 12 tasks | `hooks/useSettings.ts`, `components/Settings/TranslationSettings.tsx`, `components/ModelManager/ModelList.tsx` |
| **Phase 4** | 17 tasks | `translation/libretranslate.rs` (delete), `commands/models.rs`, `lib.rs`, test files |
| **Total** | **56 tasks** | |

---

## Verification Commands (Final)

```bash
# Rust
cargo check
cargo test
cargo clippy

# Frontend
bun run typecheck
bun run lint
bun run build

# Dead code verification
grep -r "libretranslate" src-tauri/src/ src/    # only migration comments
grep -r "whisper.*translate" src-tauri/src/     # nothing
grep -r "TranslationEngine" src-tauri/src/translation/  # only new struct
```
