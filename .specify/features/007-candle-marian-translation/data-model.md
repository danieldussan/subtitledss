# Data Model: Candle + Marian MT Translation

**Feature**: `007-candle-marian-translation`
**Date**: 2026-07-13

## Entities

### MarianEngine

**Purpose**: In-process translation engine wrapping candle-transformers Marian MT inference.

```rust
pub struct MarianEngine {
    model: Option<marian::Model>,        // candle Marian model instance
    tokenizer: Option<Tokenizer>,         // HuggingFace tokenizer
    device: Device,                       // candle device (CPU)
    direction: TranslationDirection,      // EnToEs or EsToEn
}
```

**Relationships**:
- One instance per translation direction (EN→ES, ES→EN)
- Wrapped in `Arc<Mutex<MarianEngine>>` for thread-safe shared access
- Owned by `TranslationEngine` (which holds both directions)

**State Transitions**:
```
Created → Loaded → Ready
  │         │        │
  │         │        └─ translate() called
  │         └─ load() called with model files
  └─ new(direction)
```

---

### TranslationDirection

**Purpose**: Enum for translation direction.

```rust
pub enum TranslationDirection {
    EnToEs,
    EsToEn,
}
```

**Constraints**: Only these two directions supported in v1. Additional languages require downloading more Marian models.

---

### MarianModelInfo

**Purpose**: Metadata for a Marian model — name, direction, download URL, expected files.

```rust
pub struct MarianModelInfo {
    pub name: String,              // "marian-en-es"
    pub direction: String,         // "en-es" or "es-en"
    pub repo_id: String,           // "Helsinki-NLP/opus-mt-en-es"
    pub files: Vec<MarianModelFile>,
}

pub struct MarianModelFile {
    pub filename: String,          // "model.safetensors"
    pub expected_size_mb: u64,     // 300
    pub sha256: String,            // checksum (empty if not verified)
}
```

**Relationships**:
- Referenced by `MarianModelManager` for download/verify operations
- Maps to HuggingFace repository structure

---

### MarianModelManager

**Purpose**: Manages Marian model lifecycle — download, verify, delete.

```rust
pub struct MarianModelManager {
    models_dir: PathBuf,           // ~/.local/share/subtitledss/models/
}
```

**Relationships**:
- Uses `MarianModelInfo` for model metadata
- Shared via `Arc<Mutex<MarianModelManager>>` in Tauri state
- Referenced by `TranslationEngine::load_models()`

**State**: Models stored in subdirectories:
```
~/.local/share/subtitledss/models/
├── marian-en-es/
│   ├── model.safetensors
│   ├── tokenizer.json
│   ├── config.json
│   └── generation_config.json
├── marian-es-en/
│   ├── model.safetensors
│   ├── tokenizer.json
│   ├── config.json
│   └── generation_config.json
└── ggml-base.bin  (Whisper model, existing)
```

---

### TranslationEngine (composite)

**Purpose**: Holds both MarianEngine instances and provides unified translation API.

```rust
pub struct TranslationEngine {
    en_to_es: Arc<Mutex<MarianEngine>>,
    es_to_en: Arc<Mutex<MarianEngine>>,
}
```

**Relationships**:
- Owned by `lib.rs` as Tauri managed state
- Passed to `TranscriptionPipeline::start()`
- Uses `TranslationConfig` to select direction

**API**:
- `new()` — create with unloaded engines
- `load_models(models_dir)` — load both models from disk
- `translate(text, config)` — translate using appropriate direction
- `is_loaded()` — check if both models are loaded

---

### TranslationConfig (updated)

**Purpose**: User-facing translation settings.

```rust
pub struct TranslationConfig {
    pub enabled: bool,
    pub source_lang: String,       // "en" or "es"
    pub target_lang: String,       // "en" or "es"
    pub show_original: bool,
}
```

**Removed fields**:
- `engine: String` — no longer needed (Marian is the only engine)
- `libretranslate_url: String` — no external service

**Constraints**:
- `source_lang != target_lang` for translation to occur
- Only "en" and "es" supported in v1

---

### TranscriptionParams (updated)

**Purpose**: Whisper transcription parameters.

```rust
pub struct TranscriptionParams {
    pub language: Option<String>,
    pub threads: u32,
    pub gpu: bool,
}
```

**Removed field**:
- `translate: bool` — Whisper translate mode no longer used

---

## Data Flow

```
Audio Chunk
    │
    ▼
WhisperEngine::transcribe()
    │
    ▼
TranscriptionSegments
    │
    ▼
TranslationEngine::translate()
    │
    ├─ source_lang="en", target_lang="es" → MarianEngine (EnToEs)
    │
    └─ source_lang="es", target_lang="en" → MarianEngine (EsToEn)
    │
    ▼
Translated Text
    │
    ▼
HistoryDb::insert() + emit("transcription")
```

---

## Storage Schema

### Model Directory Structure

```text
~/.local/share/subtitledss/models/
├── marian-en-es/
│   ├── model.safetensors      # ~300MB
│   ├── tokenizer.json         # ~2MB
│   ├── config.json            # <1KB
│   └── generation_config.json # <1KB
├── marian-es-en/
│   ├── model.safetensors      # ~300MB
│   ├── tokenizer.json         # ~2MB
│   ├── config.json            # <1KB
│   └── generation_config.json # <1KB
├── ggml-tiny.bin              # 39MB (Whisper)
├── ggml-base.bin              # 142MB (Whisper)
└── ...
```

### Config File (TOML)

```toml
[translation]
enabled = true
source_lang = "en"
target_lang = "es"
show_original = true
# REMOVED: engine = "marian"
# REMOVED: libretranslate_url = "http://localhost:5000"
```
