# Implementation Plan: Accumulative Overlay + Translation

**Branch**: `feature/005-overlay-translation` | **Date**: 2026-07-11 | **Spec**: [spec.md](./spec.md)

**Target Version**: `0.2.0` | **Base Branch**: `main`

**Input**: Feature specification from `.specify/features/005-overlay-translation/spec.md`

## Summary

Implement accumulative subtitle overlay with gradual fade-out and translation support. The overlay will accumulate transcription lines (max 4 visible), display each for ~10s, then fade out over ~3s. Translation will use Whisper translate by default (offline, English only) with LibreTranslate as an alternative for multi-language support.

## Branch Strategy

- **Development branch**: `feature/005-overlay-translation`
- **Base branch**: `main`
- **Merge strategy**: Squash merge → `main`
- **Version bump**: `Cargo.toml` + `tauri.conf.json` → `0.2.0` on merge
- **Release tag**: `v0.2.0`
- **CI requirement**: All checks must pass before merge

## Technical Context

**Language/Version**: Rust 2021 + TypeScript/React 19 + Bun

**Primary Dependencies**: whisper-rs 0.16, reqwest 0.12, Tauri 2, Framer Motion, Tailwind CSS 4

**Storage**: SQLite + FTS5 (existing history DB, adds `translation` column support)

**Testing**: `cargo test` (Rust), `bun run typecheck` (TypeScript)

**Target Platform**: Linux (Hyprland, Wayland), future Windows/macOS

**Project Type**: Desktop application (Tauri 2)

**Performance Goals**: <1s additional latency for translation, 60fps overlay transitions

**Constraints**: Must work fully offline for Whisper translate; LibreTranslate optional

**Scale/Scope**: Single-user desktop app, 11 files modified, 3 files created

## Constitution Check

- **Offline-first**: Whisper translate is fully offline. LibreTranslate is optional.
- **Open source**: MIT license, no external API keys required.
- **Performance**: Translation adds <1s latency. Overlay maintains 60fps.
- **Configurability**: All new parameters have sensible defaults.

## Project Structure

### Documentation (this feature)

```text
.specify/features/005-overlay-translation/
├── spec.md              # Feature specification
├── plan.md              # This file
└── tasks.md             # Task board
```

### Source Code (repository root)

```text
src-tauri/src/
├── settings/
│   └── config.rs           # Extend TranslationConfig + OverlayConfig
├── whisper/
│   └── engine.rs           # Enable set_translate()
├── translation/            # NEW: translation module
│   ├── mod.rs              # TranslationEngine enum + translate()
│   └── libretranslate.rs   # LibreTranslate HTTP client
├── pipeline/
│   └── transcriber.rs      # Integrate translation + enriched events
├── lib.rs                  # Register translation mod + shortcut
└── commands/
    └── capture.rs          # Pass translation config to pipeline

src/
├── components/Settings/
│   ├── SettingsPanel.tsx   # Add Translation tab
│   └── TranslationSettings.tsx  # NEW: translation UI
├── hooks/
│   └── useSettings.ts      # Extend AppConfig interface
└── ...

public/
└── overlay.html            # REWRITE: multi-line accumulative overlay

src-tauri/
└── tauri.conf.json         # Increase overlay height to 200px
```

**Structure Decision**: Existing Tauri 2 desktop app structure. New `translation/` module in Rust backend. Frontend remains React components + vanilla JS overlay.

## Technical Design

### Accumulative Overlay System

The overlay (`public/overlay.html`) will be rewritten from a single `<p>` to a dynamic multi-line system:

```javascript
// Each line is an independent DOM element
class SubtitleLine {
  constructor(id, text, translation, displayTime, fadeTime) {
    this.id = id;
    this.text = text;
    this.translation = translation;
    this.displayTime = displayTime;  // 10s default
    this.fadeTime = fadeTime;        // 3s default
    this.createdAt = Date.now();
    this.opacity = 1.0;
  }

  get elapsed() { return Date.now() - this.createdAt; }
  get shouldFade() { return this.elapsed > this.displayTime; }
  get shouldRemove() { return this.elapsed > this.displayTime + this.fadeTime; }
  get currentOpacity() {
    if (!this.shouldFade) return 1.0;
    const fadeElapsed = this.elapsed - this.displayTime;
    return Math.max(0, 1.0 - (fadeElapsed / this.fadeTime));
  }
}
```

Lines are managed in a `lines[]` array. Each `requestAnimationFrame` tick updates opacity and removes expired lines. New lines are appended to the end; old lines are shifted up visually.

### Translation Pipeline

```rust
// In pipeline/transcriber.rs
// After transcription, before emit:
let (original_text, translated_text) = if config.translation.enabled {
    let original = segments_to_text(&segments);
    let translated = match config.translation.engine.as_str() {
        "whisper" => {
            // Re-transcribe with translate=true if source != target
            if config.translation.target_lang == "en" {
                Some(engine.transcribe_with_translate(&audio_chunk, &params)?)
            } else { None }
        }
        "libretranslate" => {
            translation::libretranslate::translate(
                &original, &config.translation
            ).await.ok()
        }
        _ => None
    };
    (original, translated)
} else {
    (segments_to_text(&segments), None)
};

// Emit enriched event
app_handle.emit("transcription", json!({
    "id": chrono::Utc::now().timestamp_millis(),
    "text": original_text,
    "translation": translated_text,
    "start": ...,
    "end": ...,
}));
```

### Config Extensions

```rust
// TranslationConfig - extend existing
pub struct TranslationConfig {
    pub enabled: bool,
    pub source_lang: String,       // existing
    pub target_lang: String,       // existing
    pub engine: String,            // NEW: "whisper" | "libretranslate"
    pub libretranslate_url: String, // NEW: default "http://localhost:5000"
    pub show_original: bool,       // NEW: show original below translation
}

// OverlayConfig - extend existing
pub struct OverlayConfig {
    // ... existing fields ...
    pub display_duration_ms: u64,   // NEW: default 10000
    pub fade_duration_ms: u64,      // NEW: default 3000
    pub max_visible_lines: usize,   // NEW: default 4
    pub line_gap: u32,              // NEW: default 4 (px)
}
```

## Complexity Tracking

No constitution violations. All additions are backward-compatible with sensible defaults.

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Overlay rewrite breaks transparency | High | Test on Hyprland, fallback to CSS-only |
| LibreTranslate service unavailable | Medium | Graceful fallback to original text |
| Whisper translate adds too much latency | Low | Translate is fast (<500ms for short chunks) |
| Config migration breaks existing configs | Medium | Add defaults for new fields, test deserialization |

## Execution Order

1. **Config expansion** (config.rs) — All other changes depend on this
2. **Whisper translate** (engine.rs) — 1-line change, immediate value
3. **Overlay rewrite** (overlay.html + tauri.conf.json) — Independent of translation
4. **Translation module** (translation/) — Backend translation logic
5. **Pipeline integration** (transcriber.rs) — Connect translation to pipeline
6. **UI Settings** (TranslationSettings.tsx) — User control
7. **Shortcut** (lib.rs) — Quick toggle

## Release Checklist (v0.2.0)

Before merging to `main`:

- [ ] All 39 tasks complete
- [ ] `cargo test` — all pass
- [ ] `cargo clippy` — no warnings
- [ ] `bun run typecheck` — clean
- [ ] `bun run lint` — clean
- [ ] Manual test: overlay accumulates lines correctly
- [ ] Manual test: Whisper translate works (foreign language → English)
- [ ] Manual test: LibreTranslate works (if service running)
- [ ] Manual test: Ctrl+Shift+T toggles translation
- [ ] Config migration: existing 0.1.x configs load with new defaults
- [ ] Version bumped to 0.2.0 in `Cargo.toml` and `tauri.conf.json`
- [ ] PR created targeting `main`
- [ ] All CI checks pass
