# Quickstart: macOS Compatibility

**Feature**: `010-macos-compatibility`
**Date**: 2026-07-17

---

## Prerequisites

### Required

- **macOS 11.0+** (Big Sur or later)
- **Rust** (latest stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Bun** (latest): `curl -fsSL https://bun.sh/install | bash`
- **Xcode Command Line Tools**: `xcode-select --install`

### Optional

- **FFmpeg** (for video transcription): `brew install ffmpeg`
- **BlackHole** (for system audio capture): https://github.com/ExistentialAudio/BlackHole

---

## Build

### CPU-Only Build

```bash
# Clone the repo
git clone https://github.com/danieldussan/subtitledss.git
cd subtitledss

# Install frontend dependencies
bun install

# Build for macOS
cd src-tauri
cargo build --release
```

### Metal GPU-Accelerated Build (Apple Silicon)

```bash
cd src-tauri
cargo build --release --features metal
```

### Full Tauri Build (produces DMG)

```bash
cd src-tauri
cargo tauri build
```

The DMG will be at: `src-tauri/target/release/bundle/dmg/subtitledss_1.1.0_aarch64.dmg`

---

## Install

### From DMG

1. Open the `.dmg` file
2. Drag `subtitledss.app` to the `Applications` folder
3. Launch from Launchpad or Applications

### First Launch

macOS may show an "unidentified developer" warning. To bypass:

```bash
xattr -cr /Applications/subtitledss.app
```

Then launch normally.

---

## Microphone Permission

On first launch, macOS will prompt for microphone access. Click **Allow**.

If you denied it by accident:
1. Open **System Preferences** → **Security & Privacy** → **Privacy** → **Microphone**
2. Check the box next to `subtitledss`

---

## System Audio Capture (BlackHole)

macOS does not expose system audio to apps directly. To capture system audio:

1. **Install BlackHole**:
   ```bash
   brew install --cask blackhole-2ch
   ```

2. **Create a Multi-Output Device** (to hear audio while capturing):
   - Open **Audio MIDI Setup** (in Applications > Utilities)
   - Click the `+` button → **Create Multi-Output Device**
   - Check both **BlackHole 2ch** and your **Built-in Output** (speakers/headphones)
   - Set your Built-in Output as the master device

3. **Set System Output**:
   - Open **System Preferences** → **Sound** → **Output**
   - Select **Multi-Output Device**

4. **In subtitledss**:
   - Open Audio Settings
   - Select **System** as audio source
   - The device list should show **BlackHole 2ch** — select it
   - Start capture

---

## Verify Metal GPU Acceleration

After building with `--features metal`:

1. Launch the app
2. Open Whisper Settings
3. Load a model (e.g., "base")
4. Check the tracing log — you should see Metal backend initialization
5. Start capture and transcribe — CPU usage should be lower than CPU-only mode

---

## Troubleshooting

### "cargo build" fails with PipeWire errors

This should not happen after the macOS compatibility changes. If it does, verify you're building from the updated codebase.

### Audio capture produces silence

- If using microphone: Check macOS microphone permission (System Preferences → Security & Privacy → Privacy → Microphone)
- If using BlackHole: Ensure BlackHole is configured as the system output device and the Multi-Output Device is set up correctly

### Overlay doesn't appear

- The overlay uses Tauri's transparent window feature. On macOS, this requires the app to be in the foreground or have accessibility permissions.
- Try toggling the overlay off and on again with `Cmd+Shift+O`.

### FFmpeg not found

```bash
# Install FFmpeg
brew install ffmpeg

# Verify
ffmpeg -version
```

### DMG shows "app is damaged"

```bash
# Remove quarantine attribute
xattr -cr /Applications/subtitledss.app
```

---

## Run Tests

```bash
# Rust tests
cd src-tauri
cargo test

# Frontend typecheck
cd ..
bun run typecheck

# Frontend lint
bun run lint
```

---

## Performance Check

On Apple Silicon with Metal enabled:

- **Cold start**: < 3 seconds (model load)
- **Transcription latency**: < 500ms (audio chunk to overlay)
- **CPU usage**: < 40% with Base model
- **RAM usage**: < 700MB with Base model + translation
