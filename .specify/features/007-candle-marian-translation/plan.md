# Implementation Plan: In-Process Translation with Candle + Marian MT

**Feature Branch**: `007-candle-marian-translation`
**Created**: 2026-07-13
**Spec**: `.specify/features/007-candle-marian-translation/spec.md`
**Status**: Draft

## Summary

Replace LibreTranslate (external HTTP service) and Whisper translate (English-only) with candle + Marian MT in-process translation. The new engine runs entirely in Rust via `candle-transformers`, supports bidirectional EN<ES, requires no external processes, and targets <200ms latency per segment on CPU. Two Marian models (~300MB each) are downloaded from HuggingFace on first use.

---

## Technical Context

**Language/Version**: Rust 2021 edition, TypeScript 5.x
**Primary Dependencies**: candle-core, candle-transformers, candle-nn, tokenizers (new); whisper-rs, tauri 2, reqwest (existing)
**Storage**: SQLite + FTS5 (existing), TOML config (existing)
**Testing**: `cargo test`, `bun run typecheck`
**Target Platform**: Linux (Arch, Wayland, Hyprland)
**Project Type**: Desktop app (Tauri 2)
**Performance Goals**: <200ms translation latency per segment, <40% CPU with Base whisper model
**Constraints**: Single binary, no external services, offline after model download
**Scale/Scope**: 2 translation directions (EN→ES, ES→EN), ~600MB total model download

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Offline-First | ✅ PASS | Marian MT runs in-process. No network calls except model download (with consent). |
| II. Real-Time Performance | ✅ PASS | <200ms target is achievable with Marian on CPU. Synchronous inference avoids async overhead. |
| III. Modular Architecture | ✅ PASS | New `translation/marian.rs` and `translation/model.rs` are independent modules. Clear boundary. |
| IV. Linux-Native | ✅ PASS | candle-core supports CPU backend required. No GPU dependency. |
| V. Test-First | ⚠️ ACTION NEEDED | Must add unit tests for MarianEngine, model loading, and translation pipeline integration. |

---

## Project Structure

### Documentation (this feature)

```text
specs/007-candle-marian-translation/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── marian-engine.md
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src-tauri/src/
├── translation/
│   ├── mod.rs              # REWRITE — remove libretranslate dispatch, add Marian dispatch
│   ├── marian.rs           # NEW — MarianEngine struct, candle inference
│   └── model.rs            # NEW — Marian model loading, download, verification
├── settings/
│   └── config.rs           # MODIFY — remove libretranslate_url, clean engine field
├── pipeline/
│   └── transcriber.rs      # MODIFY — replace LibreTranslate call with MarianEngine
├── whisper/
│   ├── params.rs           # MODIFY — remove translate field
│   └── engine.rs           # MODIFY — remove set_translate call
├── commands/
│   ├── transcription.rs    # MODIFY — remove translate field from params
│   └── models.rs           # MODIFY — add Marian model commands
├── models/
│   ├── mod.rs              # MODIFY — export MarianModelManager
│   ├── downloader.rs       # MODIFY — add Marian model entries to available_models
│   └── manager.rs          # MODIFY — extend list_downloaded for Marian models
└── lib.rs                  # MODIFY — wire MarianEngine state

src/
├── hooks/
│   └── useSettings.ts      # MODIFY — remove libretranslate_url from AppConfig
├── components/
│   ├── Settings/
│   │   └── TranslationSettings.tsx  # MODIFY — remove engine selector, URL field, limit languages
│   └── ModelManager/
│       └── ModelList.tsx            # MODIFY — add Marian model section
└── App.tsx                          # MODIFY — no change expected (reads config via invoke)

src-tauri/
└── Cargo.toml              # MODIFY — add candle deps

DELETED:
└── src-tauri/src/translation/libretranslate.rs
```

**Structure Decision**: Single-project Tauri app. Translation is a Rust module (`src-tauri/src/translation/`). Frontend components are self-contained. No structural changes needed — only file additions and modifications.

---

## Complexity Tracking

No constitution violations. This feature simplifies the architecture by removing an external service dependency and consolidating to a single in-process engine.

---

## Phase 1: Core Candle Engine

**Goal**: Implement `MarianEngine` and model loading infrastructure.
**Dependencies**: None — foundation layer.
**Estimated scope**: 2 new files, 1 modified file (Cargo.toml).

### 1A. Add Candle Dependencies to Cargo.toml

**File**: `src-tauri/Cargo.toml`

Add to `[dependencies]`:
```toml
candle-core = "0.8"
candle-transformers = "0.8"
candle-nn = "0.8"
tokenizers = "0.21"
```

Keep existing `reqwest` (used for model download), `sha2`, `hex`.

**Verification**: `cargo check` passes with new deps.

---

### 1B. Create MarianEngine (translation/marian.rs)

**New file**: `src-tauri/src/translation/marian.rs`

Core struct:
```rust
use std::path::PathBuf;
use candle_core::{Device, Tensor};
use candle_transformers::models::marian;
use tokenizers::Tokenizer;
use tracing::{info, warn};

pub struct MarianEngine {
    model: Option<marian::Model>,
    tokenizer: Option<Tokenizer>,
    device: Device,
    direction: TranslationDirection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TranslationDirection {
    EnToEs,
    EsToEn,
}

impl MarianEngine {
    pub fn new(direction: TranslationDirection) -> Self {
        Self {
            model: None,
            tokenizer: None,
            device: Device::Cpu,
            direction,
        }
    }

    /// Load model from a directory containing model.safetensors, tokenizer.json, config.json
    pub fn load(&mut self, model_dir: &PathBuf) -> anyhow::Result<()> {
        // 1. Load tokenizer from model_dir/tokenizer.json
        // 2. Load model weights from model_dir/model.safetensors
        // 3. Initialize marian::Model with config
        // 4. Store in self
        todo!("Implement in Phase 1B")
    }

    /// Translate text synchronously. Returns translated text.
    pub fn translate(&self, text: &str) -> anyhow::Result<String> {
        if text.is_empty() {
            return Ok(String::new());
        }
        // 1. Tokenize input
        // 2. Truncate to 512 tokens, log warning if truncated
        // 3. Run model.forward()
        // 4. Decode output tokens
        // 5. Return translated text
        todo!("Implement in Phase 1B")
    }

    pub fn is_loaded(&self) -> bool {
        self.model.is_some() && self.tokenizer.is_some()
    }

    pub fn direction(&self) -> &TranslationDirection {
        &self.direction
    }
}
```

Key implementation details:
- Use `candle_transformers::models::marian` for the encoder-decoder architecture
- Tokenizer loaded from `tokenizer.json` via `tokenizers::Tokenizer::from_file()`
- Model weights from `model.safetensors` via `candle_core::VarBuilder::from_safe_tensors()`
- CPU-only device (`Device::Cpu`)
- Thread-safe: wrap in `Arc<Mutex<MarianEngine>>` at usage sites
- Max input: 512 tokens, truncate + warn
- Empty text: return empty string (zero overhead)

**Verification**:
- `cargo check` passes
- Unit test: `MarianEngine::new()` creates engine
- Unit test: `is_loaded()` returns false before load
- Unit test: `translate("")` returns empty string

---

### 1C. Create Model Loader (translation/model.rs)

**New file**: `src-tauri/src/translation/model.rs`

```rust
use std::path::PathBuf;
use anyhow::Result;
use reqwest;
use sha2::{Sha256, Digest};
use tracing::info;

pub struct MarianModelInfo {
    pub name: String,
    pub direction: String,  // "en-es" or "es-en"
    pub repo_id: String,    // "Helsinki-NLP/opus-mt-en-es"
    pub files: Vec<MarianModelFile>,
}

pub struct MarianModelFile {
    pub filename: String,
    pub expected_size_mb: u64,
    pub sha256: String,
}

pub struct MarianModelManager {
    models_dir: PathBuf,
}

impl MarianModelManager {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    /// List available Marian models (EN→ES, ES→EN)
    pub fn available_models() -> Vec<MarianModelInfo> {
        vec![
            MarianModelInfo {
                name: "marian-en-es".to_string(),
                direction: "en-es".to_string(),
                repo_id: "Helsinki-NLP/opus-mt-en-es".to_string(),
                files: vec![
                    MarianModelFile { filename: "model.safetensors".to_string(), expected_size_mb: 300, sha256: String::new() },
                    MarianModelFile { filename: "tokenizer.json".to_string(), expected_size_mb: 2, sha256: String::new() },
                    MarianModelFile { filename: "config.json".to_string(), expected_size_mb: 0, sha256: String::new() },
                    MarianModelFile { filename: "generation_config.json".to_string(), expected_size_mb: 0, sha256: String::new() },
                ],
            },
            MarianModelInfo {
                name: "marian-es-en".to_string(),
                direction: "es-en".to_string(),
                repo_id: "Helsinki-NLP/opus-mt-es-en".to_string(),
                files: vec![
                    MarianModelFile { filename: "model.safetensors".to_string(), expected_size_mb: 300, sha256: String::new() },
                    MarianModelFile { filename: "tokenizer.json".to_string(), expected_size_mb: 2, sha256: String::new() },
                    MarianModelFile { filename: "config.json".to_string(), expected_size_mb: 0, sha256: String::new() },
                    MarianModelFile { filename: "generation_config.json".to_string(), expected_size_mb: 0, sha256: String::new() },
                ],
            },
        ]
    }

    /// Download all files for a Marian model from HuggingFace
    pub async fn download(&self, model_name: &str) -> Result<PathBuf> {
        // 1. Find model by name in available_models()
        // 2. Create subdirectory: models_dir/<model_name>/
        // 3. For each file in model.files:
        //    a. Check if already downloaded (skip if exists)
        //    b. Download from https://huggingface.co/<repo_id>/resolve/main/<filename>
        //    c. Verify size
        //    d. Verify SHA256 if provided
        // 4. Return model_dir path
        todo!("Implement in Phase 1C")
    }

    /// Check if model is downloaded (all files present)
    pub fn is_downloaded(&self, model_name: &str) -> bool {
        // Check if model directory exists and all expected files are present
        todo!("Implement in Phase 1C")
    }

    /// Get path to model directory
    pub fn model_dir(&self, model_name: &str) -> PathBuf {
        self.models_dir.join(model_name)
    }

    /// Delete model from disk
    pub fn delete(&self, model_name: &str) -> Result<()> {
        let dir = self.model_dir(model_name);
        if dir.exists() {
            std::fs::remove_dir_all(dir)?;
            info!("Marian model '{}' deleted", model_name);
        }
        Ok(())
    }

    /// Verify model integrity via checksum
    pub fn verify(&self, model_name: &str) -> Result<bool> {
        // Check all files exist and checksums match
        todo!("Implement in Phase 1C")
    }
}
```

Download URL pattern:
- `https://huggingface.co/Helsinki-NLP/opus-mt-en-es/resolve/main/model.safetensors`
- `https://huggingface.co/Helsinki-NLP/opus-mt-en-es/resolve/main/tokenizer.json`
- etc.

**Verification**:
- `cargo check` passes
- Unit test: `available_models()` returns 2 models
- Unit test: `is_downloaded()` returns false for missing model
- Unit test: `model_dir()` returns correct path
- Integration test: download + verify + delete lifecycle (requires network, mark `#[ignore]`)

---

## Phase 2: Pipeline Integration

**Goal**: Wire MarianEngine into the transcription pipeline and clean up whisper/LibreTranslate code.
**Dependencies**: Phase 1 complete.
**Estimated scope**: ~8 files modified.

### 2A. Rewrite translation/mod.rs

**File**: `src-tauri/src/translation/mod.rs`

Remove:
- `pub mod libretranslate;`
- `TranslationEngine` enum
- Old `TranslationConfig` struct (use `settings::config::TranslationConfig`)
- Old `translate()` function

Replace with:
```rust
pub mod marian;
pub mod model;

use std::sync::{Arc, Mutex};
use crate::settings::config::TranslationConfig;
use marian::{MarianEngine, TranslationDirection};

pub struct TranslationEngine {
    en_to_es: Arc<Mutex<MarianEngine>>,
    es_to_en: Arc<Mutex<MarianEngine>>,
}

impl TranslationEngine {
    pub fn new() -> Self {
        Self {
            en_to_es: Arc::new(Mutex::new(MarianEngine::new(TranslationDirection::EnToEs))),
            es_to_en: Arc::new(Mutex::new(MarianEngine::new(TranslationDirection::EsToEn))),
        }
    }

    /// Load models from disk
    pub fn load_models(&self, models_dir: &std::path::Path) -> anyhow::Result<()> {
        let en_es_dir = models_dir.join("marian-en-es");
        let es_en_dir = models_dir.join("marian-es-en");

        self.en_to_es.lock().map_err(|e| e.to_string())?.load(&en_es_dir)?;
        self.es_to_en.lock().map_err(|e| e.to_string())?.load(&es_en_dir)?;

        info!("Marian MT models loaded");
        Ok(())
    }

    /// Translate text using the appropriate direction
    pub fn translate(&self, text: &str, config: &TranslationConfig) -> anyhow::Result<String> {
        if !config.enabled || text.is_empty() {
            return Ok(text.to_string());
        }

        if config.source_lang == config.target_lang {
            return Ok(text.to_string());
        }

        let engine = match (config.source_lang.as_str(), config.target_lang.as_str()) {
            ("en", "es") => &self.en_to_es,
            ("es", "en") => &self.es_to_en,
            _ => {
                warn!("Unsupported language pair: {} -> {}", config.source_lang, config.target_lang);
                return Ok(text.to_string());
            }
        };

        engine.lock().map_err(|e| e.to_string())?.translate(text)
    }

    pub fn is_loaded(&self) -> bool {
        self.en_to_es.lock().map(|e| e.is_loaded()).unwrap_or(false)
            && self.es_to_en.lock().map(|e| e.is_loaded()).unwrap_or(false)
    }
}
```

**Verification**:
- `cargo check` passes
- `TranslationEngine::new()` creates engine
- `translate()` with empty text returns empty string

---

### 2B. Update Settings Config (settings/config.rs)

**File**: `src-tauri/src/settings/config.rs`

Modify `TranslationConfig`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationConfig {
    pub enabled: bool,
    pub source_lang: String,
    pub target_lang: String,
    pub show_original: bool,
    // REMOVED: engine, libretranslate_url
}
```

Update `Default` impl:
```rust
translation: TranslationConfig {
    enabled: false,
    source_lang: "en".to_string(),
    target_lang: "es".to_string(),
    show_original: true,
},
```

Add config migration in `AppConfig::load()`:
```rust
pub fn load() -> Self {
    // ... existing load logic ...
    // After successful parse, migrate legacy fields
    if let Ok(mut config) = toml::from_str::<AppConfig>(&content) {
        // Remove legacy fields that may exist in old configs
        // The TOML deserializer will ignore unknown fields by default
        // with serde(default), so this is handled automatically
        return config;
    }
    // ... fallback to default ...
}
```

Since `engine` and `libretranslate_url` are not in the new struct, old TOML files with these fields will fail to deserialize. Add `#[serde(deny_unknown_fields)]` removal or use `serde(default)` with a custom deserializer. Simpler approach: wrap in an intermediate struct for migration:

```rust
fn migrate_config(content: &str) -> anyhow::Result<Self> {
    // Try new format first
    if let Ok(config) = toml::from_str::<AppConfig>(content) {
        return Ok(config);
    }
    // Try legacy format with engine/libretranslate_url
    #[derive(Deserialize)]
    struct LegacyTranslationConfig {
        enabled: Option<bool>,
        source_lang: Option<String>,
        target_lang: Option<String>,
        engine: Option<String>,
        libretranslate_url: Option<String>,
        show_original: Option<bool>,
    }
    // Parse and migrate...
    todo!("Implement migration logic")
}
```

**Verification**:
- `cargo check` passes
- `cargo test` — existing config tests pass
- Unit test: new config deserializes correctly
- Unit test: legacy config with `engine="whisper"` migrates gracefully

---

### 2C. Update Pipeline (pipeline/transcriber.rs)

**File**: `src-tauri/src/pipeline/transcriber.rs`

Changes:
1. Remove `translation_engine` and `libretranslate_url` variable captures (lines 53-56)
2. Add `Arc<Mutex<TranslationEngine>>` parameter to `start()`
3. Remove `use_translate` logic (line 142)
4. Remove `translate: use_translate` from `TranscriptionParams` (line 149)
5. Replace LibreTranslate call (lines 179-197) with MarianEngine call

Updated `start()` signature:
```rust
pub fn start(
    &mut self,
    buffer: Arc<Mutex<RingBuffer>>,
    engine: Arc<Mutex<WhisperEngine>>,
    history_db: Arc<Mutex<HistoryDb>>,
    app_handle: tauri::AppHandle,
    config: AppConfig,
    translation_engine: Arc<Mutex<crate::translation::TranslationEngine>>,  // NEW
)
```

Updated translation block (replacing lines 179-197):
```rust
// Handle Marian MT translation
let translation = if translation_enabled {
    let text_clone = text.clone();
    let config_clone = config.clone();
    match translation_engine.lock() {
        Ok(eng) => match eng.translate(&text_clone, &config_clone.translation) {
            Ok(t) => {
                info!("Translation: {}", t);
                Some(t)
            }
            Err(e) => {
                warn!("Translation failed: {}", e);
                None
            }
        },
        Err(e) => {
            warn!("Translation engine lock failed: {}", e);
            None
        }
    }
} else {
    None
};
```

Remove lines 53-56 (variable captures no longer needed):
- `let translation_engine = config.translation.engine.clone();`
- `let libretranslate_url = config.translation.libretranslate_url.clone();`

Remove line 142:
- `let use_translate = translation_enabled && translation_engine == "whisper";`

Remove line 149:
- `translate: use_translate,`

**Verification**:
- `cargo check` passes
- Pipeline compiles with MarianEngine integration
- `cargo test` passes

---

### 2D. Clean Whisper Params (whisper/params.rs, whisper/engine.rs)

**File**: `src-tauri/src/whisper/params.rs`

Remove `translate` field:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionParams {
    pub language: Option<String>,
    pub threads: u32,
    pub gpu: bool,
    // REMOVED: pub translate: bool,
}

impl Default for TranscriptionParams {
    fn default() -> Self {
        Self {
            language: Some("auto".to_string()),
            threads: 4,
            gpu: false,
        }
    }
}
```

**File**: `src-tauri/src/whisper/engine.rs`

Remove `set_translate` call (line 63):
```rust
// REMOVE: whisper_params.set_translate(params.translate);
```

**File**: `src-tauri/src/commands/transcription.rs`

Remove `translate: false` from params (line 36):
```rust
let params = TranscriptionParams {
    language,
    threads: threads.unwrap_or(4),
    gpu: false,
    // REMOVED: translate: false,
};
```

**Verification**:
- `cargo check` passes — no references to `translate` field remain
- `cargo test` passes
- `grep -r "translate" src-tauri/src/whisper/` shows no matches

---

### 2E. Wire TranslationEngine in lib.rs

**File**: `src-tauri/src/lib.rs`

Add `TranslationEngine` to app state:
```rust
use crate::translation::TranslationEngine;

// In run() function, after creating other states:
let translation_engine = Arc::new(Mutex::new(TranslationEngine::new()));

// Load Marian models if translation is enabled
if config.translation.enabled {
    let models_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("subtitledss")
        .join("models");
    if let Ok(engine) = translation_engine.lock() {
        if let Err(e) = engine.load_models(&models_dir) {
            tracing::warn!("Failed to load Marian models: {}", e);
        }
    }
}

// Register as Tauri managed state
app.manage(translation_engine.clone());
```

Pass to pipeline in capture command:
```rust
// In commands/capture.rs, add translation_engine parameter
```

**Verification**:
- `cargo check` passes
- App starts with Marian models loaded
- `cargo test` passes

---

### Phase 2 Verification Gate

```bash
cargo check
cargo test
bun run typecheck
bun run lint
```

---

## Phase 3: Frontend

**Goal**: Update Settings UI, Model Manager, and hooks to reflect Marian MT.
**Dependencies**: Phase 2 complete.
**Estimated scope**: 3 files modified.

### 3A. Update useSettings Hook (hooks/useSettings.ts)

**File**: `src/hooks/useSettings.ts`

Remove from `AppConfig.translation`:
```typescript
translation: {
    enabled: boolean;
    source_lang: string;
    target_lang: string;
    // REMOVED: engine: string;
    // REMOVED: libretranslate_url: string;
    show_original: boolean;
};
```

**Verification**:
- `bun run typecheck` passes
- No TypeScript errors

---

### 3B. Rewrite TranslationSettings (components/Settings/TranslationSettings.tsx)

**File**: `src/components/Settings/TranslationSettings.tsx`

Major changes:
1. Remove engine selector section (lines 97-127)
2. Remove LibreTranslate URL section (lines 176-192)
3. Limit language options to English and Spanish only
4. Remove Whisper translate warning (lines 166-171)
5. Add Marian model status display

Updated `TranslationSettingsProps`:
```typescript
interface TranslationSettingsProps {
    config: AppConfig;
    onSave: (config: AppConfig) => Promise<void>;
}
```

Updated state:
```typescript
const [enabled, setEnabled] = useState(config.translation.enabled);
const [sourceLang, setSourceLang] = useState(config.translation.source_lang);
const [targetLang, setTargetLang] = useState(config.translation.target_lang);
const [showOriginal, setShowOriginal] = useState(config.translation.show_original);
// REMOVED: engine, libretranslateUrl
```

Updated `handleSave`:
```typescript
const handleSave = async () => {
    await onSave({
        ...config,
        translation: {
            ...config.translation,
            enabled,
            source_lang: sourceLang,
            target_lang: targetLang,
            show_original: showOriginal,
        },
    });
};
```

Updated languages:
```typescript
const languages = [
    { value: "en", label: "English" },
    { value: "es", label: "Español" },
];
```

New model status section (when enabled):
```tsx
{enabled && (
    <div className="section">
        <div className="section-title">Marian MT Models</div>
        <div className="space-y-2">
            <ModelStatusCard direction="EN → ES" model="marian-en-es" />
            <ModelStatusCard direction="ES → EN" model="marian-es-en" />
        </div>
    </div>
)}
```

`ModelStatusCard` component shows:
- Model name and direction
- Download status (downloaded/missing)
- Load status (loaded/not loaded)
- Download/Delete buttons

**Verification**:
- `bun run typecheck` passes
- Manual: Open Translation Settings → no engine selector, no URL field
- Manual: Language dropdowns show only EN and ES

---

### 3C. Update Model Manager (components/ModelManager/ModelList.tsx)

**File**: `src/components/ModelManager/ModelList.tsx`

Add Marian model section after Whisper models:

```tsx
// Marian Translation Models
const marianModels: ModelInfo[] = [
    { name: "marian-en-es", filename: "marian-en-es/model.safetensors", url: "", size_mb: 300, sha256: "" },
    { name: "marian-es-en", filename: "marian-es-en/model.safetensors", url: "", size_mb: 300, sha256: "" },
];
```

Add new section in JSX:
```tsx
{/* Marian Translation Models */}
<div className="px-5 pt-3 pb-2">
    <h3 className="text-[13px] font-medium text-text-secondary">Translation Models</h3>
    <p className="text-[11px] text-text-muted mt-0.5">Marian MT for EN↔ES translation</p>
</div>

<div className="space-y-2">
    {marianModels.map((model) => (
        // Same card layout as Whisper models
        // Use marian-specific download/load/delete commands
    ))}
</div>
```

Update `checkDownloadedModels` to also check Marian models:
```typescript
const checkDownloadedModels = async () => {
    // Existing Whisper models check
    const whisperModels = await invoke<string[]>("list_downloaded_models");
    // New: Marian models check
    const marianModels = await invoke<string[]>("list_downloaded_marian_models");
    // Merge results
};
```

Update `handleDownload` to route Marian models to correct command:
```typescript
const handleDownload = async (modelName: string) => {
    if (modelName.startsWith("marian-")) {
        await invoke("download_marian_model", { modelName });
    } else {
        await invoke("download_model", { modelName });
    }
};
```

**Verification**:
- `bun run typecheck` passes
- Manual: Model Manager shows "Translation Models" section
- Manual: Download Marian model → progress shown → appears as downloaded

---

### Phase 3 Verification Gate

```bash
cargo check
bun run typecheck
bun run lint
```

---

## Phase 4: Legacy Removal

**Goal**: Delete dead code, remove unused dependencies, verify clean build.
**Dependencies**: Phase 3 complete.
**Estimated scope**: ~4 files modified/deleted.

### 4A. Delete libretranslate.rs

**Delete file**: `src-tauri/src/translation/libretranslate.rs`

Remove from `src-tauri/src/translation/mod.rs`:
```rust
// REMOVE: pub mod libretranslate;
```

**Verification**:
- `cargo check` passes
- `grep -r "libretranslate" src-tauri/src/` returns no matches
- `grep -r "LibreTranslate" src-tauri/src/` returns no matches

---

### 4B. Remove Dead Config Fields

Verify `libretranslate_url` is gone from:
- `src-tauri/src/settings/config.rs` (done in Phase 2B)
- `src/hooks/useSettings.ts` (done in Phase 3A)
- `src/components/Settings/TranslationSettings.tsx` (done in Phase 3B)
- `src-tauri/src/pipeline/transcriber.rs` (done in Phase 2C)

Verify `engine` field is gone from:
- `src-tauri/src/settings/config.rs` (done in Phase 2B)
- `src/hooks/useSettings.ts` (done in Phase 3A)
- `src/components/Settings/TranslationSettings.tsx` (done in Phase 3B)

Verify `translate` field is gone from:
- `src-tauri/src/whisper/params.rs` (done in Phase 2D)
- `src-tauri/src/whisper/engine.rs` (done in Phase 2D)
- `src-tauri/src/commands/transcription.rs` (done in Phase 2D)
- `src-tauri/src/pipeline/transcriber.rs` (done in Phase 2C)

**Verification**:
- `grep -r "libretranslate" src-tauri/src/` — no matches
- `grep -r "TranslationEngine" src-tauri/src/translation/` — only new Marian engine
- `grep -r "\.translate" src-tauri/src/whisper/` — no matches

---

### 4C. Add Marian Model Commands (commands/models.rs)

**File**: `src-tauri/src/commands/models.rs`

Add new Tauri commands:
```rust
#[tauri::command]
pub async fn download_marian_model(
    model_name: String,
    state: State<'_, Arc<Mutex<MarianModelManager>>>,
) -> Result<String, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager.download(&model_name).await.map_err(|e| e.to_string())?;
    Ok(format!("Marian model '{}' downloaded", model_name))
}

#[tauri::command]
pub async fn list_downloaded_marian_models(
    state: State<'_, Arc<Mutex<MarianModelManager>>>,
) -> Result<Vec<String>, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    // Return list of downloaded Marian model names
    todo!("Implement listing downloaded Marian models")
}

#[tauri::command]
pub async fn delete_marian_model(
    model_name: String,
    state: State<'_, Arc<Mutex<MarianModelManager>>>,
) -> Result<String, String> {
    let manager = state.lock().map_err(|e| e.to_string())?;
    manager.delete(&model_name).map_err(|e| e.to_string())?;
    Ok(format!("Marian model '{}' deleted", model_name))
}

#[tauri::command]
pub async fn load_marian_models(
    state: State<'_, Arc<Mutex<TranslationEngine>>>,
    model_manager: State<'_, Arc<Mutex<MarianModelManager>>>,
) -> Result<String, String> {
    let models_dir = {
        let mgr = model_manager.lock().map_err(|e| e.to_string())?;
        mgr.models_dir().clone()
    };
    let engine = state.lock().map_err(|e| e.to_string())?;
    engine.load_models(&models_dir).map_err(|e| e.to_string())?;
    Ok("Marian models loaded".to_string())
}
```

Register in `lib.rs` invoke_handler:
```rust
commands::models::download_marian_model,
commands::models::list_downloaded_marian_models,
commands::models::delete_marian_model,
commands::models::load_marian_models,
```

**Verification**:
- `cargo check` passes
- `cargo test` passes

---

### 4D. Add Unit Tests

**File**: `src-tauri/src/translation/marian.rs` — add tests module:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marian_engine_new() {
        let engine = MarianEngine::new(TranslationDirection::EnToEs);
        assert!(!engine.is_loaded());
    }

    #[test]
    fn test_translate_empty_string() {
        let engine = MarianEngine::new(TranslationDirection::EnToEs);
        assert_eq!(engine.translate("").unwrap(), "");
    }

    #[test]
    fn test_translation_direction() {
        let engine = MarianEngine::new(TranslationDirection::EsToEn);
        assert_eq!(engine.direction(), &TranslationDirection::EsToEn);
    }
}
```

**File**: `src-tauri/src/translation/model.rs` — add tests module:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_available_models() {
        let models = MarianModelManager::available_models();
        assert_eq!(models.len(), 2);
        assert!(models.iter().any(|m| m.direction == "en-es"));
        assert!(models.iter().any(|m| m.direction == "es-en"));
    }

    #[test]
    fn test_is_downloaded_false() {
        let manager = MarianModelManager::new(PathBuf::from("/tmp/test-models"));
        assert!(!manager.is_downloaded("marian-en-es"));
    }

    #[test]
    fn test_model_dir() {
        let manager = MarianModelManager::new(PathBuf::from("/tmp/test-models"));
        assert_eq!(manager.model_dir("marian-en-es"), PathBuf::from("/tmp/test-models/marian-en-es"));
    }
}
```

**Verification**:
- `cargo test` — all new tests pass
- `cargo test --lib translation` — translation module tests pass

---

### Phase 4 Verification Gate

```bash
cargo check
cargo test
bun run typecheck
bun run lint
```

---

## Dependency Graph

```
Phase 1 (Core Engine)
  ├─ 1A: Cargo.toml deps      ─── standalone
  ├─ 1B: marian.rs            ─── depends on 1A
  └─ 1C: model.rs             ─── depends on 1A

Phase 2 (Pipeline Integration)
  ├─ 2A: translation/mod.rs   ─── depends on 1B, 1C
  ├─ 2B: config.rs            ─── standalone (removes fields)
  ├─ 2C: transcriber.rs       ─── depends on 2A, 2B
  ├─ 2D: whisper cleanup      ─── depends on 2C
  └─ 2E: lib.rs wiring        ─── depends on 2A, 2C

Phase 3 (Frontend)
  ├─ 3A: useSettings.ts       ─── depends on 2B
  ├─ 3B: TranslationSettings  ─── depends on 3A
  └─ 3C: ModelList.tsx        ─── depends on 4C (commands)

Phase 4 (Legacy Removal)
  ├─ 4A: Delete libretranslate ─── depends on 2A
  ├─ 4B: Verify dead code     ─── depends on all above
  ├─ 4C: Marian commands      ─── depends on 1C, 2A
  └─ 4D: Unit tests           ─── depends on 1B, 1C
```

---

## File Change Summary

| File | Phase | Change Type | Description |
|------|-------|-------------|-------------|
| `src-tauri/Cargo.toml` | 1A | MODIFY | Add candle-core, candle-transformers, candle-nn, tokenizers |
| `src-tauri/src/translation/marian.rs` | 1B | NEW | MarianEngine struct with candle inference |
| `src-tauri/src/translation/model.rs` | 1C | NEW | Marian model download, load, verify |
| `src-tauri/src/translation/mod.rs` | 2A | REWRITE | Remove libretranslate dispatch, add Marian dispatch |
| `src-tauri/src/settings/config.rs` | 2B | MODIFY | Remove libretranslate_url, engine; add migration |
| `src-tauri/src/pipeline/transcriber.rs` | 2C | MODIFY | Replace LibreTranslate call with MarianEngine |
| `src-tauri/src/whisper/params.rs` | 2D | MODIFY | Remove translate field |
| `src-tauri/src/whisper/engine.rs` | 2D | MODIFY | Remove set_translate call |
| `src-tauri/src/commands/transcription.rs` | 2D | MODIFY | Remove translate field from params |
| `src-tauri/src/lib.rs` | 2E | MODIFY | Wire TranslationEngine state, register commands |
| `src/hooks/useSettings.ts` | 3A | MODIFY | Remove libretranslate_url from AppConfig |
| `src/components/Settings/TranslationSettings.tsx` | 3B | REWRITE | Remove engine selector, URL field; limit languages |
| `src/components/ModelManager/ModelList.tsx` | 3C | MODIFY | Add Marian model section |
| `src-tauri/src/translation/libretranslate.rs` | 4A | DELETE | Remove entire file |
| `src-tauri/src/commands/models.rs` | 4C | MODIFY | Add Marian model commands |

---

## Success Criteria Mapping

| Criterion | Phase | Requirement |
|-----------|-------|-------------|
| SC-001: <200ms translation latency | 1B | FR-005, FR-006 |
| SC-002: Model download <60s | 1C | FR-008, FR-009, FR-010 |
| SC-003: Cold start <3s | 2E | FR-003 |
| SC-004: RAM <700MB with translation | 1B | FR-007 |
| SC-005: Zero libretranslate references | 4A, 4B | FR-034, FR-035 |
| SC-006: cargo build zero warnings | All | FR-035 |
| SC-007: cargo test passes | All | FR-035 |
| SC-008: bun run typecheck passes | 3A, 3B | FR-033 |
| SC-009: EN→ES and ES→EN both work | 1B, 2A | FR-002 |
| SC-010: Legacy config migration | 2B | FR-023 |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Marian model incompatible with candle-transformers | HIGH | Verify Helsinki-NLP/opus-mt models are standard Marian encoder-decoder format. Test with candle examples before full implementation. |
| CPU inference >200ms per segment | MEDIUM | Benchmark early in Phase 1B. If too slow, consider quantized models or batch processing. |
| Model download fails silently | LOW | Follow existing ModelDownloader pattern with checksum verification and error toasts. |
| Legacy config migration breaks existing installs | MEDIUM | Test with actual config.toml files from v0.1.0. Use `serde(default)` and tolerate unknown fields. |
