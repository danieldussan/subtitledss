# Quickstart: Candle + Marian MT Translation

**Feature**: `007-candle-marian-translation`
**Date**: 2026-07-13

## Prerequisites

- Rust toolchain (stable, edition 2021)
- Node.js/Bun for frontend
- Internet connection (for model download on first launch)

## Build

```bash
# Install dependencies
cd src-tauri && cargo check

# Build the app
cd .. && bun run build && cd src-tauri && cargo build --release
```

## Test

```bash
# Rust tests
cd src-tauri && cargo test

# TypeScript check
cd .. && bun run typecheck

# Lint
bun run lint
```

## Verify Translation Works

1. **Launch the app**:
   ```bash
   cd src-tauri && cargo run
   ```

2. **Enable translation**:
   - Open Settings → Translation tab
   - Toggle "Enable Translation" ON
   - Set Source Language: English
   - Set Target Language: Español
   - Click Save

3. **Download Marian models** (first launch):
   - Open Model Manager tab
   - See "Translation Models" section
   - Click Download on "marian-en-es" and "marian-es-en"
   - Wait for download to complete (~300MB each)

4. **Load models**:
   - Click Load on both Marian models
   - Models show "Active" badge when loaded

5. **Test transcription**:
   - Start audio capture (Ctrl+Shift+S)
   - Speak English audio
   - Overlay should show:
     - Original English text
     - Spanish translation below

6. **Test bidirectional**:
   - Change Source Language to Español
   - Change Target Language to English
   - Save settings
   - Speak Spanish audio
   - Overlay should show Spanish original + English translation

## Verify Model Manager

1. Open Model Manager
2. Whisper models section shows: tiny, base, small, medium, large-v3
3. Translation Models section shows: marian-en-es, marian-es-en
4. Each shows: name, direction, size, download/load/delete actions
5. Downloaded models show file size
6. Loaded models show "Active" badge

## Verify Settings UI

1. Open Settings → Translation
2. No engine selector (Marian is the only engine)
3. No URL field (no external service)
4. Language dropdowns show only: English, Español
5. "Show Original Text" toggle works
6. Model status shows download/load state

## Verify Legacy Config Migration

1. Create a test config.toml with old format:
   ```toml
   [translation]
   enabled = true
   engine = "whisper"
   libretranslate_url = "http://localhost:5000"
   source_lang = "en"
   target_lang = "es"
   show_original = true
   ```
2. Launch the app
3. App should load without error
4. Translation settings should show Marian MT (not whisper/libretranslate)
5. Config saves in new format (without engine/libretranslate_url)

## Performance Check

1. Enable translation with Base whisper model
2. Start audio capture
3. Monitor CPU usage (should stay below 40% on 4-core)
4. Monitor RAM (should stay below 700MB total)
5. Translation latency should be imperceptible (<200ms)

## Troubleshooting

### Models not downloading
- Check internet connection
- Check `~/.local/share/subtitledss/models/` directory exists
- Check disk space (need ~600MB for both Marian models)

### Translation not working
- Verify models are loaded (Model Manager shows "Active")
- Check source_lang and target_lang are different
- Check translation is enabled in Settings
- Check logs for error messages

### App crashes on startup
- Check if candle-core is compatible with CPU
- Try building without GPU features: `cargo build --no-default-features`
- Check Rust toolchain version (need stable 2021 edition)

### Legacy config not migrating
- Check config.toml is valid TOML
- Check for syntax errors
- Delete config.toml to reset to defaults
- Reconfigure settings manually
