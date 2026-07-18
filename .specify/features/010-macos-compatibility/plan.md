# Implementation Plan: macOS Compatibility

**Feature Branch**: `010-macos-compatibility`
**Created**: 2026-07-17
**Spec**: `.specify/features/010-macos-compatibility/spec.md`
**Status**: Draft

## Summary

Extend subtitledss from Linux-only (Arch/Wayland/Hyprland + PipeWire) to cross-platform Linux + macOS. The changes are primarily in build configuration (platform-conditional Cargo dependencies), audio device detection (CoreAudio naming), keyboard shortcuts (Cmd vs Ctrl), and DMG packaging. Core functionality (whisper transcription, overlay, translation, video processing) is already cross-platform through Tauri, CPAL, and whisper-rs.

---

## Technical Context

**Language/Version**: Rust 2021 edition, TypeScript 5.x
**Primary Dependencies**: cpal (CoreAudio on macOS), whisper-rs (Metal feature), tauri 2, dirs 5
**Storage**: SQLite + FTS5 (bundled — cross-platform), TOML config
**Testing**: `cargo test`, `bun run typecheck`
**Target Platform**: Linux (Arch/Wayland) + macOS 11+ (Big Sur+)
**Project Type**: Desktop app (Tauri 2)
**Performance Goals**: <500ms transcription latency, Metal GPU acceleration on Apple Silicon
**Constraints**: Single binary, no external services, offline after model download
**Scale/Scope**: 2 platforms, ~10 files modified, 0 new Rust modules

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Offline-First | ✅ PASS | No new network dependencies. BlackHole is a local virtual driver. |
| II. Real-Time Performance | ✅ PASS | CoreAudio provides low-latency capture. Metal acceleration improves inference. |
| III. Modular Architecture | ✅ PASS | Platform-specific code is isolated via `cfg(target_os)` attributes. No new modules. |
| IV. Linux-Native | ⚠️ EXTENDED | Constitution says "Linux-Native (Arch/Wayland/Hyprland)". This feature EXTENDS to macOS while preserving Linux as primary. Linux behavior is unchanged. |
| V. Test-First | ✅ PASS | Existing tests run on both platforms. No new test infrastructure needed. |

**Constitution Note**: Principle IV states "Linux-Native". This feature does NOT replace Linux support — it extends to macOS. The Linux build and behavior remain identical. The `cfg` attributes ensure zero impact on Linux builds.

---

## Project Structure

### Documentation (this feature)

```text
.specify/features/010-macos-compatibility/
├── plan.md              # This file
├── research.md          # Platform research findings
├── data-model.md        # Platform detection data model
├── quickstart.md        # macOS build & run guide
└── tasks.md             # Task board (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src-tauri/
├── Cargo.toml                    # MODIFY — platform-conditional deps, metal feature
├── tauri.conf.json               # MODIFY — macOS bundle config (DMG, min OS)
└── src/
    ├── lib.rs                    # MODIFY — platform-aware shortcuts (Cmd vs Ctrl)
    ├── audio/
    │   └── capture.rs            # MODIFY — macOS device detection (BlackHole)
    ├── video/
    │   └── processor.rs          # MODIFY — macOS FFmpeg error message
    └── whisper/
        └── engine.rs             # VERIFY — works with Metal backend (no changes expected)

src/
├── components/
│   └── Settings/
│       ├── AudioSettings.tsx     # MODIFY — macOS BlackHole guide message
│       └── ShortcutsSettings.tsx # MODIFY — display Cmd on macOS
└── hooks/
    └── useSettings.ts            # VERIFY — no changes expected

config/
└── default.toml                  # MODIFY — platform-aware default shortcuts

aur/
└── PKGBUILD                      # NO CHANGE — Linux-only packaging
```

**Structure Decision**: Single-project Tauri app. No new Rust modules — platform differences are handled via `cfg(target_os)` attributes and runtime detection. The existing module structure is preserved. Frontend changes are minimal (conditional UI messages).

---

## Complexity Tracking

No constitution violations requiring justification. This feature EXTENDS platform support while preserving existing Linux behavior. All platform-specific code is isolated via `cfg` attributes.

---

## Phase 1: Build System & Dependencies

**Goal**: Make `cargo build` compile on macOS without errors.
**Dependencies**: None — foundation layer.
**Estimated scope**: 2 files modified.

### 1A. Platform-Conditional cpal in Cargo.toml

**File**: `src-tauri/Cargo.toml`

Current:
```toml
cpal = { version = "0.18", features = ["pipewire"] }
```

Replace with platform-conditional:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
cpal = { version = "0.18", features = ["pipewire"] }

[target.'cfg(target_os = "macos")'.dependencies]
cpal = { version = "0.18" }

[target.'cfg(target_os = "windows")'.dependencies]
cpal = { version = "0.18" }
```

**Why**: PipeWire is a Linux-only audio backend. On macOS, CPAL uses CoreAudio natively (no feature flags needed). On Windows, it uses WASAPI.

**Verification**: `cargo check` on macOS passes (no PipeWire link errors).

---

### 1B. Metal Feature Flag in Cargo.toml

**File**: `src-tauri/Cargo.toml`

Add to `[features]`:
```toml
[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
cuda = ["whisper-rs/cuda"]
vulkan = ["whisper-rs/vulkan"]
metal = ["whisper-rs/metal"]
gpu = ["cuda"]
```

Add platform-conditional whisper-rs:
```toml
[target.'cfg(target_os = "macos")'.dependencies]
whisper-rs = { version = "0.16", features = ["tracing_backend", "metal"] }

[target.'cfg(target_os = "linux")'.dependencies]
whisper-rs = { version = "0.16", features = ["tracing_backend"] }
```

**Why**: Metal is Apple's GPU API. whisper-rs supports it via the `metal` feature. On Linux, CUDA/Vulkan are separate features.

**Verification**: `cargo check` on macOS with `--features metal` passes.

---

### 1C. Update Package Description

**File**: `src-tauri/Cargo.toml`

Change:
```toml
description = "Real-time subtitle overlay for Linux"
```
To:
```toml
description = "Real-time subtitle overlay for Linux and macOS"
```

**Verification**: `cargo check` passes.

---

### Phase 1 Verification Gate

```bash
cargo check          # macOS — no PipeWire errors
cargo check          # Linux — no regression (pipewire feature still active)
```

---

## Phase 2: Audio & Platform Detection

**Goal**: Make audio capture work on macOS and add platform-specific UX.
**Dependencies**: Phase 1 complete.
**Estimated scope**: 3 files modified.

### 2A. macOS Device Detection in capture.rs

**File**: `src-tauri/src/audio/capture.rs`

Current device classification (lines 77-81):
```rust
let is_monitor = name.contains("Monitor")
    || name.contains("monitor")
    || name.contains("sink")
    || name.contains("Sink");
let kind = if is_monitor { "system" } else { "mic" };
```

Replace with platform-conditional:
```rust
#[cfg(target_os = "macos")]
let is_monitor = name.contains("BlackHole")
    || name.contains("Soundflower")
    || name.contains("Aggregate Device")
    || name.contains("System Audio");

#[cfg(target_os = "linux")]
let is_monitor = name.contains("Monitor")
    || name.contains("monitor")
    || name.contains("sink")
    || name.contains("Sink");

#[cfg(target_os = "windows")]
let is_monitor = name.contains("Stereo Mix")
    || name.contains("virtual");

let kind = if is_monitor { "system" } else { "mic" };
```

**Why**: PipeWire names system audio as "Monitor of ..." or "...sink". macOS virtual audio drivers (BlackHole, Soundflower) use their own names. Windows uses "Stereo Mix".

**Verification**: `cargo check` passes on all platforms; `cargo test` passes.

---

### 2B. FFmpeg Error Message for macOS

**File**: `src-tauri/src/video/processor.rs`

In `extract_audio()` and `get_duration()`, the error messages are generic. Add macOS-specific hint:

```rust
#[cfg(target_os = "macos")]
let ffmpeg_hint = "\n\nHint: Install FFmpeg on macOS with: brew install ffmpeg";

#[cfg(target_os = "linux")]
let ffmpeg_hint = "";

// In error messages, append ffmpeg_hint
```

**Why**: macOS users may not have FFmpeg installed. A helpful message prevents confusion.

**Verification**: `cargo check` passes.

---

### 2C. BlackHole Guide in AudioSettings.tsx

**File**: `src/components/Settings/AudioSettings.tsx`

Add a platform-conditional message when "System" audio source is selected on macOS:

```tsx
{audioSource === "system" && platform === "darwin" && (
  <div className="rounded-lg bg-yellow-500/10 border border-yellow-500/20 p-3 text-sm">
    <p className="text-yellow-400 font-medium">macOS System Audio</p>
    <p className="text-text-secondary mt-1">
      macOS requires a virtual audio driver to capture system audio.
      Install <a href="https://github.com/ExistentialAudio/BlackHole" className="text-blue-400 underline">BlackHole</a> and set it as your audio output device.
    </p>
  </div>
)}
```

Detect platform via Tauri's `navigator.userAgent` or a new `get_platform` Tauri command.

**Verification**: `bun run typecheck` passes; manual: macOS shows guide, Linux does not.

---

### Phase 2 Verification Gate

```bash
cargo check
cargo test
bun run typecheck
bun run lint
```

---

## Phase 3: Keyboard Shortcuts

**Goal**: Use `Cmd` on macOS, `Ctrl` on Linux for global shortcuts.
**Dependencies**: Phase 1 complete (parallel with Phase 2).
**Estimated scope**: 2 files modified.

### 3A. Platform-Aware Shortcuts in lib.rs

**File**: `src-tauri/src/lib.rs`

Current (lines 304-326):
```rust
let ctrl_shift_s = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS);
// ... etc
```

Replace with:
```rust
#[cfg(target_os = "macos")]
use tauri_plugin_global_shortcut::Modifiers;
#[cfg(target_os = "macos")]
let modifier = Modifiers::META;  // Cmd key on macOS

#[cfg(target_os = "linux")]
let modifier = Modifiers::CONTROL;  // Ctrl key on Linux

#[cfg(target_os = "windows")]
let modifier = Modifiers::CONTROL;  // Ctrl key on Windows

let ctrl_shift_s = Shortcut::new(Some(modifier | Modifiers::SHIFT), Code::KeyS);
let ctrl_shift_o = Shortcut::new(Some(modifier | Modifiers::SHIFT), Code::KeyO);
let ctrl_shift_t = Shortcut::new(Some(modifier | Modifiers::SHIFT), Code::KeyT);
```

**Why**: macOS convention is `Cmd` for primary shortcuts. `Ctrl` on macOS is reserved for system-level operations.

**Verification**: `cargo check` passes on both platforms; `Cmd+Shift+S` works on macOS.

---

### 3B. Platform-Aware Default Shortcuts in config

**File**: `config/default.toml`

This is tricky because TOML is static. Two approaches:

**Approach A (recommended)**: Keep `Ctrl` in the TOML file and override at runtime on macOS:
```rust
// In lib.rs, after loading config:
#[cfg(target_os = "macos")]
{
    // Override shortcut strings for display purposes
    config.shortcuts.toggle_capture = "Cmd+Shift+S".to_string();
    config.shortcuts.toggle_overlay = "Cmd+Shift+O".to_string();
    config.shortcuts.toggle_translation = "Cmd+Shift+T".to_string();
}
```

**Approach B**: Use platform-conditional compilation for the config file. Not practical with TOML.

Approach A is cleaner — the TOML stays Linux-centric, and macOS overrides at runtime.

**Verification**: `cargo check` passes; shortcuts display correctly on both platforms.

---

### Phase 3 Verification Gate

```bash
cargo check
cargo test
bun run typecheck
```

---

## Phase 4: Packaging & Distribution

**Goal**: Build DMG for macOS distribution.
**Dependencies**: Phase 1-3 complete.
**Estimated scope**: 1 file modified.

### 4A. macOS Bundle Configuration in tauri.conf.json

**File**: `src-tauri/tauri.conf.json`

Add macOS section to `bundle`:
```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "macOS": {
      "minimumSystemVersion": "11.0",
      "dmg": {
        "appPosition": { "x": 180, "y": 170 },
        "applicationFolderPosition": { "x": 480, "y": 170 },
        "windowSize": { "width": 660, "height": 400 }
      },
      "signingIdentity": null,
      "entitlements": null
    },
    "linux": {
      "deb": {
        "depends": [
          "libasound2-dev",
          "libpipewire-0.3-dev"
        ]
      }
    }
  }
}
```

**Why**: Tauri needs macOS-specific bundle config for DMG generation. `minimumSystemVersion: "11.0"` ensures the app won't run on unsupported macOS versions.

**Verification**: `cargo tauri build` produces a `.dmg` file on macOS.

---

### 4B. Entitlements for Microphone Access

**File**: `src-tauri/macOS.entitlements` (NEW)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.device.audio-input</key>
    <true/>
</dict>
</plist>
```

Reference in `tauri.conf.json`:
```json
"macOS": {
    "entitlements": "./macOS.entitlements"
}
```

**Why**: macOS requires explicit entitlement for microphone access. Without it, the app will crash or be denied access.

**Verification**: App requests microphone permission on first launch.

---

### Phase 4 Verification Gate

```bash
cargo tauri build     # On macOS — produces .dmg
```

---

## Phase 5: CI/CD Release Workflow

**Goal**: Add macOS build job to the release workflow so GitHub Releases produce Linux + macOS artifacts.
**Dependencies**: Phase 1-4 complete (code must compile on macOS before CI can build it).
**Estimated scope**: 1 file modified.

### 5A. Add macOS Build Job to release.yml

**File**: `.github/workflows/release.yml`

Current structure:
```yaml
jobs:
  build-linux:
    runs-on: ubuntu-latest
    # ... Linux build steps ...
  create-release:
    needs: build-linux
    # ... create release with Linux artifacts only ...
```

New structure:
```yaml
jobs:
  build-linux:
    runs-on: ubuntu-latest
    # ... existing Linux build steps (unchanged) ...

  build-macos:
    name: Build macOS
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - uses: oven-sh/setup-bun@v2
        with:
          bun-version: ${{ env.BUN_VERSION }}

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Install frontend dependencies
        run: bun install --frozen-lockfile

      - name: Build frontend
        run: bun run build

      - name: Build Rust backend (Metal)
        run: cargo build --release --features metal
        working-directory: src-tauri

      - name: Build DMG
        run: cargo tauri build
        working-directory: src-tauri

      - name: Build raw binary
        run: |
          tar -czf subtitledss-${GITHUB_REF_NAME#v}-aarch64.tar.gz \
            -C src-tauri/target/release subtitledss

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: macos-builds
          path: |
            src-tauri/target/release/bundle/dmg/*.dmg
            subtitledss-*.tar.gz

  create-release:
    name: Create Release
    needs: [build-linux, build-macos]  # BOTH must succeed
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download Linux artifacts
        uses: actions/download-artifact@v4
        with:
          name: linux-builds

      - name: Download macOS artifacts
        uses: actions/download-artifact@v4
        with:
          name: macos-builds

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          name: "subtitledss ${{ github.ref_name }}"
          body: |
            ## subtitledss ${{ github.ref_name }}

            Real-time subtitle overlay for Linux and macOS, powered by Whisper.cpp.

            ### Installation

            **macOS (DMG):**
            1. Download `subtitledss-*.dmg`
            2. Open the DMG and drag `subtitledss.app` to Applications
            3. First launch: right-click → Open (unsigned app)
            4. Microphone access: allow in System Preferences → Security & Privacy

            **macOS (manual):**
            ```bash
            tar -xzf subtitledss-*.tar.gz
            ./subtitledss
            ```

            **Linux (AppImage):**
            ```bash
            chmod +x subtitledss-*.AppImage
            ./subtitledss-*.AppImage
            ```

            **Linux (Debian/Ubuntu):**
            ```bash
            sudo dpkg -i subtitledss-*.deb
            ```

            **Linux (Arch):**
            ```bash
            tar -xzf subtitledss-*.tar.gz
            sudo cp subtitledss /usr/bin/subtitledss
            ```

            **System Audio on macOS:**
            Requires [BlackHole](https://github.com/ExistentialAudio/BlackHole) for system audio capture.
            See README for setup instructions.

            **GPU Acceleration:**
            - macOS: Metal (Apple Silicon) — included in DMG build
            - Linux: CUDA/Vulkan — build from source with `--features cuda`

            ### Changes
            - Real-time transcription with whisper.cpp
            - System audio capture (PipeWire on Linux, BlackHole on macOS)
            - Transparent subtitle overlay
            - System tray with global shortcuts
            - History with FTS5 search
            - Export to SRT/VTT/TXT/JSON
            - macOS support with Metal GPU acceleration
          files: |
            subtitledss-*.AppImage
            subtitledss-*.deb
            subtitledss-*.tar.gz
            subtitledss-*.dmg
          draft: false
          prerelease: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**Key changes**:
1. New `build-macos` job on `macos-latest` runner
2. Builds with `--features metal` for Apple Silicon GPU acceleration
3. Uses `cargo tauri build` to produce DMG (not manual DMG creation)
4. `create-release` now depends on `[build-linux, build-macos]` — both must pass
5. Release body updated with macOS instructions
6. 5 artifacts attached: AppImage, .deb, Linux .tar.gz, macOS DMG, macOS .tar.gz

**Verification**: Push a `v*` tag → both build jobs run in parallel → release created with all 5 artifacts.

---

### 5B. Update CI Workflow for macOS

**File**: `.github/workflows/ci.yml`

The CI workflow runs on push/PR. It should also test on macOS to catch issues early:

```yaml
jobs:
  # ... existing jobs ...

  rust-check-macos:
    name: Rust Check (macOS)
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: oven-sh/setup-bun@v2
        with:
          bun-version: ${{ env.BUN_VERSION }}
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
      - name: Install frontend dependencies
        run: bun install --frozen-lockfile
      - name: Build frontend
        run: bun run build
      - name: Rust check
        run: cargo check --all-targets
        working-directory: src-tauri
```

**Why**: Catch macOS compilation errors on every push/PR, not just at release time.

---

### Phase 5 Verification Gate

```bash
# Verify workflow syntax
cat .github/workflows/release.yml | head -5  # valid YAML

# On macOS machine:
cargo build --release --features metal  # must compile
cargo tauri build                        # must produce DMG
```

---

## Phase 6: Polish & Cross-Cutting

**Goal**: Verify everything works end-to-end, fix edge cases.
**Dependencies**: Phase 1-5 complete.

### 5A. Verify Overlay on macOS

- Test transparent overlay window on macOS
- Verify always-on-top behavior
- Verify drag functionality
- Document any macOS-specific quirks (e.g., fullscreen Spaces)

### 5B. Verify Shortcuts Settings UI

- Open Shortcuts Settings on macOS — confirm `Cmd` is displayed
- Open Shortcuts Settings on Linux — confirm `Ctrl` is displayed (no regression)

### 5C. Verify Video Transcription

- Test FFmpeg detection on macOS
- Test video transcription with FFmpeg installed
- Test error message with FFmpeg missing

### 5D. Documentation Updates

- Update `README.md` with macOS installation instructions
- Add macOS to the "Supported Platforms" section
- Document BlackHole setup for system audio

---

## Dependency Graph

```
Phase 1 (Build System)
  ├─ 1A: Cargo.toml cpal        ─── standalone
  ├─ 1B: Cargo.toml metal       ─── standalone
  └─ 1C: Cargo.toml description ─── standalone

Phase 2 (Audio & Platform)
  ├─ 2A: capture.rs detection   ─── depends on 1A
  ├─ 2B: processor.rs FFmpeg    ─── standalone
  └─ 2C: AudioSettings.tsx      ─── standalone

Phase 3 (Shortcuts)
  ├─ 3A: lib.rs shortcuts       ─── depends on 1A
  └─ 3B: config defaults        ─── depends on 3A

Phase 4 (Packaging)
  ├─ 4A: tauri.conf.json DMG    ─── depends on 1A, 1B
  └─ 4B: macOS.entitlements     ─── depends on 4A

Phase 5 (CI/CD)
  ├─ 5A: release.yml macOS job  ─── depends on 1A, 1B, 4A
  └─ 5B: ci.yml macOS check     ─── depends on 1A, 1B

Phase 6 (Polish)
  └─ 6A-6D: Verification        ─── depends on all above
```

---

## File Change Summary

| File | Phase | Change Type | Description |
|------|-------|-------------|-------------|
| `src-tauri/Cargo.toml` | 1A, 1B, 1C | MODIFY | Platform-conditional cpal, add metal feature, update description |
| `src-tauri/src/audio/capture.rs` | 2A | MODIFY | macOS device detection (BlackHole/Soundflower) |
| `src-tauri/src/video/processor.rs` | 2B | MODIFY | macOS FFmpeg error hint |
| `src/components/Settings/AudioSettings.tsx` | 2C | MODIFY | BlackHole setup guide for macOS |
| `src-tauri/src/lib.rs` | 3A | MODIFY | Platform-aware shortcut modifiers (Cmd vs Ctrl) |
| `config/default.toml` | 3B | NO CHANGE | Runtime override on macOS (approach A) |
| `src-tauri/tauri.conf.json` | 4A | MODIFY | macOS bundle config, DMG settings, min OS |
| `src-tauri/macOS.entitlements` | 4B | NEW | Microphone entitlement for macOS |
| `.github/workflows/release.yml` | 5A | MODIFY | Add macOS build job, update release body |
| `.github/workflows/ci.yml` | 5B | MODIFY | Add macOS rust-check job |
| `README.md` | 6D | MODIFY | macOS installation instructions |

---

## Success Criteria Mapping

| Criterion | Phase | Requirement |
|-----------|-------|-------------|
| SC-001: cargo build on macOS | 1A, 1B | FR-001, FR-002, FR-005 |
| SC-002: Metal build | 1B | FR-014 |
| SC-003: App launches on macOS | 4A | FR-024, FR-025, FR-026 |
| SC-004: Mic capture works | 2A | FR-006, FR-007, FR-008 |
| SC-005: Metal acceleration | 1B | FR-015, FR-016 |
| SC-006: Cmd shortcuts | 3A | FR-018, FR-019, FR-020 |
| SC-007: DMG builds | 4A | FR-024, FR-025 |
| SC-008: Overlay works | 6A | FR-021, FR-022 |
| SC-009: cargo test passes | All | FR-005 |
| SC-010: typecheck passes | 2C | — |
| SC-011: BlackHole guide | 2C | FR-011, FR-012, FR-013 |
| SC-012: FFmpeg error | 2B | FR-028, FR-029 |
| SC-013: 5 release artifacts | 5A | FR-033, FR-038, FR-039 |
| SC-014: CI completes <15min | 5A | FR-033, FR-034, FR-035 |
| SC-015: DMG installs on fresh Mac | 5A | FR-036, FR-040 |

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| CPAL CoreAudio doesn't support all macOS audio devices | MEDIUM | Test with built-in mic, AirPods, and BlackHole. File CPAL issue if needed. |
| whisper-rs metal feature fails to compile | HIGH | Verify whisper-rs 0.16 metal support. Fall back to CPU-only if needed. |
| Tauri transparent window doesn't work on macOS | LOW | Tauri 2 uses WKWebView which supports transparency. Verify early in Phase 6. |
| BlackHole not detected by CPAL | MEDIUM | CPAL lists all CoreAudio devices. BlackHole appears as a virtual device. Test explicitly. |
| DMG signing required for distribution | LOW | For v1, unsigned DMG works with `xattr -cr` override. Notarization is a future concern. |
| Global shortcuts conflict with macOS system shortcuts | MEDIUM | `Cmd+Shift+letter` is generally safe. Test for conflicts with macOS Spotlight, etc. |
| macOS CI runner is slow or unavailable | LOW | GitHub `macos-latest` is reliable. If unavailable, the release is blocked until the runner returns. |
| Metal feature increases macOS build time | LOW | Metal compilation adds ~2-3 minutes. Total macOS build still under 10 minutes. |
