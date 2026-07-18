# Implementation Plan: Translation Integration QA Fixes

**Feature Branch**: `006-translation-qa-fixes`
**Created**: 2026-07-13
**Spec**: `.specify/features/006-translation-qa-fixes/spec.md`
**Status**: Draft

## Summary

13 issues across 12 user stories, 35 functional requirements. Organized into 4 phases with clear dependencies. Each phase targets a logical grouping of fixes that can be verified independently.

---

## Phase 1: Critical Fixes (P1)

**Goal**: Fix the 3 critical bugs that cause runtime errors or UI desync.
**Dependencies**: None — this is the foundation.
**Estimated scope**: ~8 files modified, 1 new command.

### 1A. Delete History Entry Command (Issue #1 — FR-001..FR-004)

**Problem**: Frontend calls `invoke("delete_history_entry", { id })` but no such Tauri command exists. Clicking the trash icon throws a runtime error.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/commands/history.rs` | Add `delete_history_entry` command function |
| `src-tauri/src/lib.rs` | Register `commands::history::delete_history_entry` in `invoke_handler` |
| `src/components/History/HistoryList.tsx` | Add toast notification on success/error in `handleDeleteEntry` |

**Detailed changes**:

**`src-tauri/src/commands/history.rs`** — Add after `clear_history` (line ~34):
```rust
#[tauri::command]
pub async fn delete_history_entry(
    id: i64,
    state: State<'_, Arc<Mutex<HistoryDb>>>,
) -> Result<(), String> {
    let db = state.lock().map_err(|e| e.to_string())?;
    db.delete(id).map_err(|e| e.to_string())?;
    Ok(())
}
```

**`src-tauri/src/lib.rs`** — Add to `invoke_handler` (after line 103):
```rust
commands::history::delete_history_entry,
```

**`src/components/History/HistoryList.tsx`** — Modify `handleDeleteEntry` (lines 127-135):
- Import `useToast` hook
- Wrap in try/catch with `toast.success("Entry deleted")` on success and `toast.error(msg)` on failure
- Show toast notification per FR-003, FR-004

**Verification**:
- `cargo check` passes
- `cargo test` — existing history tests pass (db.delete already tested)
- Manual: Open History tab → click trash → entry removed, success toast shown

---

### 1B. Translation State Sync on Startup (Issue #2 — FR-005..FR-006)

**Problem**: `App.tsx` initializes `translationEnabled` to `false` (line 26) regardless of config. After restart, the toggle shows disabled even if config has `enabled: true`.

**Files to modify**:

| File | Change |
|------|--------|
| `src/App.tsx` | Read config on mount to initialize `translationEnabled` state |

**Detailed changes**:

**`src/App.tsx`** — Modify the `useEffect` at lines 89-108:
- Add a call to `invoke<AppConfig>("get_config")` at the start of the mount effect
- Set `setTranslationEnabled(config.translation.enabled)` from the loaded config
- Also set `setOverlayVisible` from config or from querying overlay window state

The mount effect (line 89) currently calls `loadModelState()` and `loadAudioDevice()`. Add a new `loadTranslationState()` function:
```typescript
const loadTranslationState = async () => {
  try {
    const config = await invoke<AppConfig>("get_config");
    setTranslationEnabled(config.translation.enabled);
  } catch (err) {
    console.error("Failed to load translation state:", err);
  }
};
```
Call it inside the mount `useEffect` alongside the existing `loadModelState()` and `loadAudioDevice()` calls.

**Verification**:
- `bun run typecheck` passes
- Manual: Enable translation → restart app → toggle shows enabled

---

### 1C. Overlay Toggle Sync (Issue #11 — FR-028..FR-030)

**Problem**: `App.tsx` line 55 uses optimistic state inversion (`setOverlayVisible((prev) => !prev)`) instead of using the backend return value. The `toggle_overlay` command already returns `Result<bool, String>` (see `commands/overlay.rs` line 5), but the frontend ignores it.

**Files to modify**:

| File | Change |
|------|--------|
| `src/App.tsx` | Use return value from `invoke("toggle_overlay")` instead of `!prev` |

**Detailed changes**:

**`src/App.tsx`** — Modify `toggleOverlay` (lines 52-59):
```typescript
const toggleOverlay = useCallback(async () => {
  try {
    const visible = await invoke<boolean>("toggle_overlay");
    setOverlayVisible(visible);
  } catch (err) {
    toast.error("Failed to toggle overlay");
    console.error("Failed to toggle overlay:", err);
  }
}, [toast]);
```

This uses the actual return value from the backend (`true` = shown, `false` = hidden), so system tray toggles that trigger the event will still go through the backend and get the correct state back.

Additionally, the system tray `"show_overlay"` handler in `lib.rs` (lines 171-177) should emit a `"toggle-overlay"` event so the frontend updates when toggled from the tray. Currently it directly calls `overlay.hide()`/`overlay.show()` without notifying the frontend. Change the tray handler to emit the event:
```rust
"show_overlay" => {
    let _ = app.emit("toggle-overlay", ());
}
```
This makes the tray action go through the same event path as the global shortcut, and the frontend will call `toggle_overlay` which returns the new state.

**Verification**:
- `bun run typecheck` passes
- Manual: Toggle overlay from system tray → main window button updates correctly

---

### Phase 1 Verification Gate

```bash
cargo check                    # Rust compiles
cargo test                     # All existing tests pass
bun run typecheck              # TypeScript compiles
bun run lint                   # No new lint errors
```

---

## Phase 2: Moderate Fixes (P2)

**Goal**: Fix 4 moderate bugs affecting export, shortcuts, HTTP reliability, and config merging.
**Dependencies**: Phase 1 complete.
**Estimated scope**: ~6 files modified.

### 2A. SRT/VTT Export Include Translations (Issue #3 — FR-007..FR-009)

**Problem**: `to_srt()` and `to_vtt()` in `export.rs` always use `entry.original_text` (lines 69, 86). They ignore `entry.translation`. The `to_txt()` function (lines 91-103) already handles this correctly.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/commands/export.rs` | Update `to_srt()` and `to_vtt()` to include translations |

**Detailed changes**:

**`src-tauri/src/commands/export.rs`** — Create a helper function and update both formatters:

Add a helper function (after line 22):
```rust
/// Determine the display text for an entry based on translation availability.
/// When translation is present, shows translation with original in parentheses.
fn display_text(entry: &ExportEntry) -> String {
    match &entry.translation {
        Some(translation) => format!("{} ({})", translation, entry.original_text),
        None => entry.original_text.clone(),
    }
}
```

Modify `to_srt()` (line 69): Replace `entry.original_text` with `display_text(entry)`.
Modify `to_vtt()` (line 86): Replace `entry.original_text` with `display_text(entry)`.

Update the existing tests to verify translation inclusion:
- `test_srt_format`: Add assertion that SRT output contains the translation text
- `test_vtt_format`: Add assertion that VTT output contains the translation text
- Add new test `test_srt_translation_only`: Verify behavior when only translation exists

**Verification**:
- `cargo test` — all export tests pass (including new assertions)
- `cargo test --lib commands::export` — specifically tests this module

---

### 2B. Shortcuts from Config (Issue #4 — FR-010..FR-012)

**Problem**: Global shortcuts in `lib.rs` (lines 205-227) are hardcoded as `Ctrl+Shift+S`, `Ctrl+Shift+O`, `Ctrl+Shift+T`. The `ShortcutsConfig` in the TOML file is never read for shortcut registration.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | Read `config.shortcuts` and use those values for global shortcut registration |

**Detailed changes**:

**`src-tauri/src/lib.rs`** — Replace hardcoded shortcuts (lines 205-227) with config-driven registration:

The `config` variable is already available in the `.setup()` closure (line 118). Parse each shortcut string from `config.shortcuts` and register them.

Add a helper function inside `run()` or at module level:
```rust
fn parse_shortcut(s: &str) -> Option<Shortcut> {
    // Parse strings like "Ctrl+Shift+S" into Shortcut structs
    // Returns None if invalid, logs warning
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in &parts {
        match *part {
            "Ctrl" | "Control" => modifiers |= Modifiers::CONTROL,
            "Shift" => modifiers |= Modifiers::SHIFT,
            "Alt" => modifiers |= Modifiers::ALT,
            "Super" | "Meta" | "Win" => modifiers |= Modifiers::SUPER,
            "S" => code = Some(Code::KeyS),
            "O" => code = Some(Code::KeyO),
            "T" => code = Some(Code::KeyT),
            "H" => code = Some(Code::KeyH),
            _ => {
                tracing::warn!("Unknown shortcut key: {}", part);
                return None;
            }
        }
    }

    code.map(|c| Shortcut::new(Some(modifiers), c))
}
```

Then in the setup closure, replace the three hardcoded shortcut blocks:
```rust
// Read shortcuts from config
let shortcuts = {
    let cfg = app.state::<Arc<Mutex<AppConfig>>>();
    let cfg = cfg.lock().unwrap();
    cfg.shortcuts.clone()
};

// Register toggle_capture shortcut
if let Some(shortcut) = parse_shortcut(&shortcuts.toggle_capture) {
    app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            tracing::info!("Global shortcut: toggle capture");
            let _ = app.emit("toggle-capture", ());
        }
    })?;
} else {
    tracing::warn!("Invalid toggle_capture shortcut '{}', using default", shortcuts.toggle_capture);
    let default = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS);
    app.global_shortcut().on_shortcut(default, move |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let _ = app.emit("toggle-capture", ());
        }
    })?;
}

// Similarly for toggle_overlay and toggle_translation
```

**Verification**:
- `cargo check` passes
- Manual: Change shortcut in config.toml → restart → new shortcut works

---

### 2C. HTTP Timeout for LibreTranslate (Issue #5 — FR-013..FR-016)

**Problem**: `libretranslate.rs` creates a `reqwest::Client::new()` (line 34) with no timeout. A hung server blocks the pipeline indefinitely.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/translation/libretranslate.rs` | Add configurable timeout to HTTP client |
| `src-tauri/src/settings/config.rs` | Add `timeout_seconds` field to `TranslationConfig` |
| `src-tauri/src/pipeline/transcriber.rs` | Pass timeout config to translation calls |

**Detailed changes**:

**`src-tauri/src/settings/config.rs`** — Add `timeout_seconds` to `TranslationConfig` (line 51):
```rust
pub struct TranslationConfig {
    pub enabled: bool,
    pub source_lang: String,
    pub target_lang: String,
    pub engine: String,
    pub libretranslate_url: String,
    pub show_original: bool,
    pub timeout_seconds: u64,  // NEW: default 10
}
```

Update `Default` impl (line 101):
```rust
translation: TranslationConfig {
    // ... existing fields ...
    timeout_seconds: 10,
},
```

**`src-tauri/src/translation/libretranslate.rs`** — Add timeout parameter:
```rust
pub async fn translate(
    text: &str,
    source_lang: &str,
    target_lang: &str,
    url: &str,
    timeout_secs: u64,  // NEW parameter
) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;
    // ... rest unchanged
}
```

**`src-tauri/src/pipeline/transcriber.rs`** — Pass timeout to translate call (line 180):
```rust
match crate::translation::libretranslate::translate(
    &text,
    &source_lang,
    &target_lang,
    &libretranslate_url,
    config.translation.timeout_seconds,
).await {
```

Note: `config` is available in the `start()` method. We need to capture `timeout_seconds` before the async block alongside the other config values (line 56).

**Verification**:
- `cargo check` passes
- `cargo test` passes (update existing serialization test in config.rs)
- `bun run typecheck` passes (AppConfig TypeScript interface needs `timeout_seconds` field)

**Also update**:
- `src/hooks/useSettings.ts` — Add `timeout_seconds: number` to the `translation` section of `AppConfig` interface (line 35)
- `src/App.tsx` — No changes needed (toggle reads config via invoke)

---

### 2D. Settings Config Deep-Merge (Issue #12 — FR-031..FR-032)

**Problem**: `useSettings.ts` line 86 uses `{ ...config, ...updates }` — a shallow merge. Calling `updateConfig({ translation: { enabled: true } })` wipes out `source_lang`, `target_lang`, etc.

**Files to modify**:

| File | Change |
|------|--------|
| `src/hooks/useSettings.ts` | Replace shallow merge with recursive deep merge |

**Detailed changes**:

**`src/hooks/useSettings.ts`** — Replace the `updateConfig` function (lines 83-90):

Add a deep merge utility function:
```typescript
function deepMerge<T extends Record<string, unknown>>(target: T, source: Partial<T>): T {
  const result = { ...target };
  for (const key of Object.keys(source) as Array<keyof T>) {
    const sourceVal = source[key];
    const targetVal = target[key];
    if (
      sourceVal !== null &&
      sourceVal !== undefined &&
      typeof sourceVal === "object" &&
      !Array.isArray(sourceVal) &&
      typeof targetVal === "object" &&
      targetVal !== null &&
      !Array.isArray(targetVal)
    ) {
      (result as Record<string, unknown>)[key as string] = deepMerge(
        targetVal as Record<string, unknown>,
        sourceVal as Record<string, unknown>,
      );
    } else if (sourceVal !== undefined) {
      result[key] = sourceVal as T[keyof T];
    }
  }
  return result;
}
```

Update `updateConfig`:
```typescript
const updateConfig = useCallback(
  (updates: Partial<AppConfig>) => {
    if (!config) return;
    const newConfig = deepMerge(config, updates);
    saveConfig(newConfig);
  },
  [config, saveConfig],
);
```

**Verification**:
- `bun run typecheck` passes
- Manual: Change only `translation.enabled` in Settings → save → other translation fields preserved

---

### Phase 2 Verification Gate

```bash
cargo check
cargo test                     # All tests pass including export tests
bun run typecheck
bun run lint
```

---

## Phase 3: Architecture Cleanup (P3)

**Goal**: Clean up translation dispatcher, engine constraints, show_original, and config hot-reload.
**Dependencies**: Phase 2 complete (timeout_seconds field needed).
**Estimated scope**: ~4 files modified.

### 3A. Translation Dispatcher Cleanup (Issue #6 — FR-017..FR-020)

**Problem**: Pipeline calls `crate::translation::libretranslate::translate()` directly (line 180 of `transcriber.rs`) instead of going through the dispatcher. The `translation/mod.rs` dispatcher exists but has its own `TranslationConfig` struct (line 20) that duplicates `settings::config::TranslationConfig`.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/translation/mod.rs` | Remove dead `TranslationConfig` struct, update dispatcher to accept `settings::config::TranslationConfig` |
| `src-tauri/src/pipeline/transcriber.rs` | Call `crate::translation::translate()` dispatcher instead of `libretranslate::translate()` directly |

**Detailed changes**:

**`src-tauri/src/translation/mod.rs`**:

1. Remove the `TranslationConfig` struct (lines 20-27) — it's dead code, the pipeline uses `settings::config::TranslationConfig`.

2. Update the `translate` function signature to accept `&settings::config::TranslationConfig`:
```rust
use crate::settings::config::TranslationConfig;

pub async fn translate(
    text: &str,
    config: &TranslationConfig,
) -> anyhow::Result<String> {
    if !config.enabled {
        return Ok(text.to_string());
    }

    if config.source_lang == config.target_lang {
        return Ok(text.to_string());
    }

    match config.engine.as_str() {
        "whisper" => {
            // Whisper translate is handled in the pipeline via whisper_params.set_translate(true)
            // This function is a fallback — return original text
            Ok(text.to_string())
        }
        "libretranslate" => {
            libretranslate::translate(
                text,
                &config.source_lang,
                &config.target_lang,
                &config.libretranslate_url,
                config.timeout_seconds,
            )
            .await
        }
        _ => {
            tracing::warn!("Unknown translation engine: {}, falling back to no translation", config.engine);
            Ok(text.to_string())
        }
    }
}
```

3. Remove the `TranslationEngine` enum (lines 5-18) — no longer needed since we match on the string directly.

**`src-tauri/src/pipeline/transcriber.rs`** — Replace direct libretranslate call (lines 179-197):

Replace:
```rust
let translation = if translation_enabled && translation_engine == "libretranslate" {
    match crate::translation::libretranslate::translate(...)
```

With:
```rust
let translation = if translation_enabled {
    match crate::translation::translate(&text, &config).await {
```

But wait — `config` is not available inside the `tokio::spawn` block because it's moved. We need to either:
- Clone the `config` before the spawn, OR
- Use `Arc<AppConfig>` shared state

Since Phase 3C will handle hot-reload with shared state, for now clone the full config before the spawn:
```rust
let config_clone = config.clone();
// ... in the spawn block:
let translation = if config_clone.translation.enabled {
    match crate::translation::translate(&text, &config_clone.translation).await {
        Ok(t) => { info!("Translation: {}", t); Some(t) }
        Err(e) => { warn!("Translation failed: {}", e); None }
    }
} else {
    None
};
```

**Verification**:
- `cargo check` passes
- `cargo test` passes
- No dead code warnings for translation module

---

### 3B. Pipeline Respects show_original (Issue #7 — FR-021..FR-023)

**Problem**: The pipeline always emits both `text` and `translation` in the transcription event (line 214). The `show_original` config field is never checked, so the frontend always receives both.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/pipeline/transcriber.rs` | Conditionally emit original text based on `show_original` |

**Detailed changes**:

**`src-tauri/src/pipeline/transcriber.rs`** — Modify the emit block (lines 214-221):

Add `show_original` to the cloned config values (it's already in `config_clone.translation.show_original`).

After computing `translation`, conditionally build the emit payload:
```rust
let show_original = config_clone.translation.show_original;
let display_text = if translation.is_some() && !show_original {
    // When show_original is false and we have a translation,
    // only emit the translation as the primary text
    translation.as_ref().unwrap().clone()
} else {
    text.clone()
};

let _ = app_handle.emit("transcription", serde_json::json!({
    "id": chrono::Utc::now().timestamp_millis(),
    "text": display_text,
    "translation": if show_original { translation } else { None },
    "start": segments.first().map(|s| s.start).unwrap_or(0.0),
    "end": segments.last().map(|s| s.end).unwrap_or(0.0),
    "speed_ratio": speed_ratio,
}));
```

Also update the history DB insert to respect `show_original`:
```rust
if let Ok(db) = history_db.lock() {
    let history_translation = if show_original { translation.as_deref() } else { None };
    let history_text = if !show_original && translation.is_some() {
        translation.as_ref().unwrap().as_str()
    } else {
        &text
    };
    if let Err(e) = db.insert(&language, history_text, history_translation, None) {
        warn!("Failed to insert history: {}", e);
    }
}
```

**Verification**:
- `cargo check` passes
- Manual: Set `show_original = false` → transcribe → overlay shows only translated text

---

### 3C. Whisper Engine Constraints (Issue #7 — FR-021)

**Problem**: Whisper translate mode only supports English output. The UI already shows a warning (TranslationSettings.tsx line 166-170), but the pipeline doesn't enforce it.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/pipeline/transcriber.rs` | Add runtime warning when Whisper is used with non-English target |

**Detailed changes**:

**`src-tauri/src/pipeline/transcriber.rs`** — Add check in the `start()` method before the spawn:

```rust
// Warn about Whisper engine constraints
if config.translation.enabled && config.translation.engine == "whisper" 
    && config.translation.target_lang != "en" {
    tracing::warn!(
        "Whisper translate mode only supports English output. \
         Target language '{}' will be ignored. Use LibreTranslate for other languages.",
        config.translation.target_lang
    );
    // Emit warning to frontend
    let _ = app_handle.emit("pipeline-warning", serde_json::json!({
        "type": "translation_engine_constraint",
        "message": "Whisper translate only supports English. Translating to English instead.",
    }));
}
```

**Verification**:
- `cargo check` passes
- Manual: Select Whisper engine + Spanish target → warning toast appears on start

---

### 3D. Pipeline Config Hot-Reload (Issue #8 — FR-024..FR-025)

**Problem**: Pipeline clones config once at start (lines 49-56 of `transcriber.rs`). Changes via shortcut toggle or Settings require restart.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/pipeline/transcriber.rs` | Use `Arc<Mutex<AppConfig>>` instead of cloned config for translation settings |
| `src-tauri/src/lib.rs` | Pass `config_arc` to pipeline instead of cloned config |
| `src-tauri/src/commands/capture.rs` | Pass `config_arc` to pipeline start |

**Detailed changes**:

**`src-tauri/src/pipeline/transcriber.rs`**:

Change the `start()` signature to accept `config: Arc<Mutex<AppConfig>>` instead of `config: AppConfig`:

```rust
pub fn start(
    &mut self,
    buffer: Arc<Mutex<RingBuffer>>,
    engine: Arc<Mutex<WhisperEngine>>,
    history_db: Arc<Mutex<HistoryDb>>,
    app_handle: tauri::AppHandle,
    config: Arc<Mutex<AppConfig>>,  // Changed from AppConfig
) {
```

Inside the spawn loop, read translation config dynamically:
```rust
let handle = tokio::spawn(async move {
    let mut last_samples_pushed: u64 = 0;

    loop {
        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Read current translation config (cheap clone of small struct)
        let (translation_enabled, translation_engine, source_lang, target_lang, 
             libretranslate_url, timeout_seconds, show_original) = {
            match config.lock() {
                Ok(cfg) => (
                    cfg.translation.enabled,
                    cfg.translation.engine.clone(),
                    cfg.translation.source_lang.clone(),
                    cfg.translation.target_lang.clone(),
                    cfg.translation.libretranslate_url.clone(),
                    cfg.translation.timeout_seconds,
                    cfg.translation.show_original,
                ),
                Err(_) => continue,
            }
        };

        // ... rest of loop uses these values instead of cloned config
    }
});
```

**`src-tauri/src/commands/capture.rs`** — Update the call to `pipeline.start()` to pass `config_arc` instead of a cloned config.

**Verification**:
- `cargo check` passes
- Manual: Start capture → toggle translation via shortcut → next chunk reflects change without restart

---

### Phase 3 Verification Gate

```bash
cargo check
cargo test
bun run typecheck
```

---

## Phase 4: Code Cleanup (P3)

**Goal**: Remove dead code and redundant logic.
**Dependencies**: Phase 3 complete.
**Estimated scope**: ~4 files modified/deleted.

### 4A. Remove Dead auto Logic in libretranslate.rs (Issue #10 — FR-026)

**Problem**: Lines 38-42 have a redundant if/else:
```rust
source: if source_lang == "auto" {
    "auto".to_string()
} else {
    source_lang.to_string()
},
```
This always produces the same result as `source_lang.to_string()`.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/translation/libretranslate.rs` | Simplify to direct assignment |

**Detailed changes**:

Replace lines 38-42 with:
```rust
source: source_lang.to_string(),
```

**Verification**:
- `cargo test` — existing serialization test still passes

---

### 4B. Remove Dead Model Manager Code (Issue #13 — FR-033..FR-035)

**Problem**: Two duplicate `ModelManager`/`ModelInfo`/`ModelDownloader` implementations exist:
- `whisper::model::{ModelManager, ModelInfo}` — **USED** by `commands/models.rs`, `lib.rs`
- `models::{ModelManager, ModelDownloader, ModelInfo}` — **DEAD CODE** except `ModelDownloader` is used by `commands/models.rs`

The `models::manager::ModelManager` is a near-exact duplicate of `whisper::model::ModelManager`. The `models::downloader::ModelDownloader` and `models::downloader::ModelInfo` are used by `commands/models.rs` for downloading.

**Analysis of `commands/models.rs`**:
- `download_model` (line 10): Uses `ModelManager` (from whisper) for `is_downloaded` check, then `ModelDownloader` (from models) for actual download
- `delete_model` (line 33): Uses `ModelManager` (from whisper)
- `list_downloaded_models` (line 45): Uses `ModelManager` (from whisper)
- `load_model` (line 53): Uses `ModelManager` (from whisper)

So `models::manager::ModelManager` is truly dead (never imported by `commands/models.rs`). But `models::downloader::{ModelDownloader, ModelInfo}` IS used.

**Strategy**: Remove `models::manager` module entirely. Keep `models::downloader` but consider moving it to `whisper::model` for consistency. For now, just remove the dead module.

**Files to modify**:

| File | Change |
|------|--------|
| `src-tauri/src/models/mod.rs` | Remove `pub mod manager` and `pub use manager::ModelManager` |
| `src-tauri/src/models/manager.rs` | Delete file |
| `src-tauri/src/lib.rs` | No change needed — it imports `whisper::model::ModelManager` (line 20), not `models::manager` |

**Detailed changes**:

**`src-tauri/src/models/mod.rs`** — Change to:
```rust
pub mod downloader;

pub use downloader::ModelDownloader;
```

**Delete** `src-tauri/src/models/manager.rs`.

**Verification**:
- `cargo check` passes (no remaining references to `models::manager::ModelManager`)
- `cargo test` passes
- `cargo test --lib models` if any tests exist

---

### 4C. Remove Dead TranslationConfig Enum (Already handled in 3A)

The `TranslationEngine` enum and `TranslationConfig` struct in `translation/mod.rs` are removed as part of Phase 3A. No additional work needed here.

---

### Phase 4 Verification Gate

```bash
cargo check
cargo test
cargo test --lib            # Unit tests
bun run typecheck
bun run lint
```

---

## Dependency Graph

```
Phase 1 (Critical)
  ├─ 1A: Delete History Entry    ─── standalone
  ├─ 1B: Translation State Sync  ─── standalone
  └─ 1C: Overlay Toggle Sync     ─── standalone

Phase 2 (Moderate)
  ├─ 2A: SRT/VTT Export          ─── standalone
  ├─ 2B: Shortcuts from Config   ─── standalone
  ├─ 2C: HTTP Timeout            ─── standalone (adds timeout_seconds to config)
  └─ 2D: Deep Merge              ─── standalone

Phase 3 (Architecture)
  ├─ 3A: Dispatcher Cleanup      ─── depends on 2C (timeout_seconds field)
  ├─ 3B: show_original           ─── depends on 3A (dispatcher uses correct config)
  ├─ 3C: Whisper Constraints     ─── standalone
  └─ 3D: Hot-Reload              ─── depends on 3A (dispatcher uses Arc<Config>)

Phase 4 (Cleanup)
  ├─ 4A: Dead auto Logic         ─── standalone
  └─ 4B: Dead Model Manager      ─── standalone
```

---

## File Change Summary

| File | Phase | Changes |
|------|-------|---------|
| `src-tauri/src/commands/history.rs` | 1A | Add `delete_history_entry` command |
| `src-tauri/src/lib.rs` | 1A, 1C, 2B, 3D | Register command, fix tray handler, config-driven shortcuts, pass Arc to pipeline |
| `src/App.tsx` | 1B, 1C, 2C | Load translation state on mount, use backend return for overlay toggle |
| `src-tauri/src/commands/export.rs` | 2A | Add `display_text()` helper, update `to_srt()` and `to_vtt()` |
| `src-tauri/src/translation/libretranslate.rs` | 2C, 4A | Add timeout parameter, simplify auto logic |
| `src-tauri/src/settings/config.rs` | 2C | Add `timeout_seconds` to `TranslationConfig` |
| `src/hooks/useSettings.ts` | 2C, 2D | Add `timeout_seconds` to interface, implement deep merge |
| `src-tauri/src/translation/mod.rs` | 3A | Remove dead `TranslationConfig`/`TranslationEngine`, use `settings::config::TranslationConfig` |
| `src-tauri/src/pipeline/transcriber.rs` | 3A, 3B, 3C, 3D | Use dispatcher, respect show_original, Whisper warning, hot-reload config |
| `src-tauri/src/models/mod.rs` | 4B | Remove `manager` module |
| `src-tauri/src/models/manager.rs` | 4B | DELETE |
| `src/components/History/HistoryList.tsx` | 1A | Add toast notifications to delete handler |

---

## Success Criteria Mapping

| Criterion | Phase | Requirement |
|-----------|-------|-------------|
| SC-001: Trash icon deletes 100% | 1A | FR-001..FR-004 |
| SC-002: Toggle matches config on restart | 1B | FR-005..FR-006 |
| SC-003: SRT/VTT contain translations | 2A | FR-007..FR-009 |
| SC-004: Custom shortcuts active after restart | 2B | FR-010..FR-012 |
| SC-005: Timeout within 10-15s | 2C | FR-013..FR-016 |
| SC-006: show_original respected | 3B | FR-022..FR-023 |
| SC-007: Whisper warning for non-English | 3C | FR-021 |
| SC-008: Shortcut toggle takes effect immediately | 3D | FR-024..FR-025 |
| SC-009: cargo test passes | All | FR-035 |
| SC-010: bun run typecheck passes | All | — |
| SC-011: Overlay toggle always reflects state | 1C | FR-028..FR-030 |
| SC-012: updateConfig preserves nested fields | 2D | FR-031..FR-032 |
| SC-013: Dead models removed, compiles | 4B | FR-033..FR-035 |
