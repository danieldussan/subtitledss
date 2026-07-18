# Contract: MarianEngine Interface

**Feature**: `007-candle-marian-translation`
**Date**: 2026-07-13

## Overview

This contract defines the public interface for the MarianEngine and related types. All implementations must conform to these specifications.

---

## MarianEngine

### Constructor

```rust
pub fn new(direction: TranslationDirection) -> Self
```

**Preconditions**: None
**Postconditions**: Engine is created in unloaded state. `is_loaded()` returns `false`.
**Side Effects**: None

---

### load

```rust
pub fn load(&mut self, model_dir: &PathBuf) -> anyhow::Result<()>
```

**Preconditions**:
- `model_dir` must exist and be a directory
- Directory must contain: `model.safetensors`, `tokenizer.json`, `config.json`
- `generation_config.json` is optional (uses defaults if missing)

**Postconditions**:
- On success: `is_loaded()` returns `true`
- On failure: `is_loaded()` returns `false`, error returned

**Side Effects**:
- Reads files from disk
- Allocates memory for model weights (~300MB)
- Initializes candle tensors

**Error Cases**:
- `model_dir` not found → `Err("Model directory not found: ...")`
- Missing files → `Err("Missing model file: ...")`
- Invalid weights → `Err("Failed to load model: ...")`
- Tokenizer parse error → `Err("Failed to load tokenizer: ...")`

---

### translate

```rust
pub fn translate(&self, text: &str) -> anyhow::Result<String>
```

**Preconditions**:
- `is_loaded()` must be `true`
- `text` must not be null (empty string is valid)

**Postconditions**:
- Returns translated text in target language
- Empty input returns empty string (zero overhead)
- Input exceeding 512 tokens is truncated (warning logged)

**Side Effects**:
- Logs warning if input truncated
- Logs info with first 50 chars of translation

**Error Cases**:
- Model not loaded → `Err("Model not loaded")`
- Tokenization failure → `Err("Tokenization failed: ...")`
- Inference failure → `Err("Translation failed: ...")`
- Decoding failure → `Err("Failed to decode output: ...")`

**Performance**:
- Latency: <200ms for typical subtitle segments (10-30 words)
- Memory: No additional allocation per call (reuses model buffers)

---

### is_loaded

```rust
pub fn is_loaded(&self) -> bool
```

**Returns**: `true` if both model and tokenizer are loaded, `false` otherwise.

---

### direction

```rust
pub fn direction(&self) -> &TranslationDirection
```

**Returns**: Reference to the engine's translation direction.

---

## TranslationDirection

```rust
pub enum TranslationDirection {
    EnToEs,
    EsToEn,
}
```

**Constraints**: Only these two variants in v1.

---

## MarianModelManager

### available_models

```rust
pub fn available_models() -> Vec<MarianModelInfo>
```

**Returns**: Static list of 2 models (marian-en-es, marian-es-en).

---

### download

```rust
pub async fn download(&self, model_name: &str) -> Result<PathBuf>
```

**Preconditions**:
- `model_name` must be in `available_models()`
- Internet connection available

**Postconditions**:
- Returns path to model directory on success
- All model files downloaded and verified
- Model directory created at `models_dir/<model_name>/`

**Side Effects**:
- Makes HTTP requests to HuggingFace
- Writes files to disk
- Logs download progress

**Error Cases**:
- Unknown model name → `Err("Model '...' not found")`
- Network error → `Err("Download failed: ...")`
- Disk write error → `Err("Failed to write file: ...")`
- Checksum mismatch → `Err("Checksum verification failed")`

---

### is_downloaded

```rust
pub fn is_downloaded(&self, model_name: &str) -> bool
```

**Returns**: `true` if model directory exists and all required files are present.

---

### delete

```rust
pub fn delete(&self, model_name: &str) -> Result<()>
```

**Postconditions**: Model directory and all files removed from disk.

---

## TranslationEngine

### new

```rust
pub fn new() -> Self
```

**Postconditions**: Both MarianEngine instances created in unloaded state.

---

### load_models

```rust
pub fn load_models(&self, models_dir: &Path) -> anyhow::Result<()>
```

**Preconditions**:
- `models_dir` must contain `marian-en-es/` and `marian-es-en/` subdirectories
- Both directories must contain valid model files

**Postconditions**:
- On success: `is_loaded()` returns `true`
- On failure: error returned, partial load state possible

---

### translate

```rust
pub fn translate(&self, text: &str, config: &TranslationConfig) -> anyhow::Result<String>
```

**Preconditions**:
- `is_loaded()` must be `true`
- `config.source_lang` and `config.target_lang` must be "en" or "es"
- `config.source_lang != config.target_lang`

**Postconditions**:
- Returns translated text
- Empty input returns empty string
- Same language returns input unchanged

**Error Cases**:
- Unsupported language pair → returns input unchanged (warning logged)
- Model not loaded → `Err("Translation models not loaded")`
- Translation fails → `Err("Translation failed: ...")`

---

### is_loaded

```rust
pub fn is_loaded(&self) -> bool
```

**Returns**: `true` if both EN→ES and ES→EN models are loaded.

---

## Integration Contract

### Pipeline → TranslationEngine

The `TranscriptionPipeline` calls `TranslationEngine::translate()` after whisper transcription.

**Input**: `text: &str` (transcribed text), `config: &TranslationConfig`
**Output**: `Ok(translated_text)` or `Err(message)`
**Error handling**: Pipeline continues with original text on error (logs warning)

### Frontend → MarianModelManager

The frontend calls Tauri commands to manage Marian models.

**Commands**:
- `download_marian_model(model_name: String) → Result<String, String>`
- `list_downloaded_marian_models() → Result<Vec<String>, String>`
- `delete_marian_model(model_name: String) → Result<String, String>`
- `load_marian_models() → Result<String, String>`

**Error handling**: Returns error string to frontend, displayed as toast notification.
