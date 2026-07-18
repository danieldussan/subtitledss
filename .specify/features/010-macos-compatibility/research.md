# Research: macOS Compatibility

**Feature**: `010-macos-compatibility`
**Date**: 2026-07-17

---

## 1. CPAL Audio Backend on macOS

**Question**: Does CPAL support CoreAudio on macOS natively, and what feature flags are needed?

**Finding**: CPAL uses CoreAudio as its default backend on macOS. No feature flags are needed — the `pipewire` feature is Linux-only. When compiled on macOS, `cpal::default_host()` automatically returns a CoreAudio host.

**Source**: CPAL documentation + crate source. The `pipewire` feature gates `cpal-backend-pipewire` which is Linux-only. On macOS, the CoreAudio backend is always compiled.

**Impact**: Simple platform-conditional dependency in Cargo.toml. No code changes needed in `capture.rs` for basic audio capture.

---

## 2. whisper-rs Metal Support

**Question**: Does whisper-rs 0.16 support Metal GPU acceleration on macOS?

**Finding**: Yes. whisper-rs exposes a `metal` feature flag that enables Metal GPU acceleration via whisper.cpp's Metal backend. This works on Apple Silicon (M1/M2/M3/M4) and Intel Macs with Metal-capable GPUs.

**Source**: whisper-rs crate features, whisper.cpp Metal backend documentation.

**Impact**: Add `metal = ["whisper-rs/metal"]` feature flag. On macOS, default whisper-rs features should include `metal`. On Linux, keep existing `cuda`/`vulkan` features.

---

## 3. BlackHole Virtual Audio Driver

**Question**: How does BlackHole appear in CPAL device listings on macOS?

**Finding**: BlackHole installs as a CoreAudio virtual device. CPAL lists all CoreAudio devices via `host.input_devices()`. BlackHole appears as a device named "BlackHole 2ch" (or "BlackHole 16ch" for the multi-channel version). It can be selected as an input device for system audio capture.

**Source**: BlackHole GitHub repository, CoreAudio documentation.

**Impact**: Device detection in `capture.rs` needs to check for "BlackHole" and "Soundflower" names on macOS instead of PipeWire "Monitor"/"sink" names.

---

## 4. Tauri Transparent Window on macOS

**Question**: Does Tauri 2's transparent window feature work on macOS?

**Finding**: Yes. Tauri 2 uses WKWebView on macOS, which supports transparent backgrounds. The `transparent: true` + `decorations: false` + `alwaysOnTop: true` configuration works on macOS. The overlay window should function identically to Linux.

**Source**: Tauri 2 documentation, WKWebView transparency support.

**Impact**: No changes needed for the overlay. Verify during Phase 5 testing.

---

## 5. macOS System Tray / Menu Bar

**Question**: How does Tauri 2 handle system tray on macOS?

**Finding**: Tauri 2's `TrayIconBuilder` works on macOS and places the icon in the menu bar (top-right area). The tray menu appears as a dropdown from the menu bar icon. This is standard macOS behavior.

**Source**: Tauri 2 tray documentation.

**Impact**: No changes needed. The existing tray code in `lib.rs` is cross-platform.

---

## 6. macOS Microphone Permission

**Question**: What does macOS require for microphone access?

**Finding**: macOS requires:
1. An `NSMicrophoneUsageDescription` key in `Info.plist` (Tauri adds this automatically when audio permissions are declared)
2. An entitlement `com.apple.security.device.audio-input` in the app's entitlements
3. User approval via the system permission dialog on first use

**Source**: Apple Developer Documentation, Tauri 2 macOS entitlements.

**Impact**: Create `macOS.entitlements` file with the audio-input entitlement. Reference it in `tauri.conf.json`.

---

## 7. macOS DMG Building with Tauri

**Question**: How does Tauri 2 build DMG files?

**Finding**: Tauri 2 automatically builds a DMG when `cargo tauri build` runs on macOS. The DMG configuration is in `tauri.conf.json` under `bundle.macOS.dmg`. The DMG includes the `.app` bundle and a symlink to `/Applications` for drag-to-install.

**Source**: Tauri 2 bundling documentation.

**Impact**: Add `macOS` section to `tauri.conf.json` with `minimumSystemVersion`, DMG layout, and entitlements reference.

---

## 8. `dirs` Crate on macOS

**Question**: Does the `dirs` crate return correct macOS paths?

**Finding**: Yes. `dirs::data_dir()` on macOS returns `~/Library/Application Support/`. The app's data directory becomes `~/Library/Application Support/subtitledss/`. This is the standard macOS location for app data.

**Source**: `dirs` crate documentation.

**Impact**: No changes needed. The existing `dirs::data_dir()` calls work correctly on macOS.

---

## 9. Global Shortcuts: Cmd vs Ctrl

**Question**: How do Tauri global shortcuts work with macOS Cmd key?

**Finding**: Tauri's `Modifiers::META` maps to the `Cmd` (⌘) key on macOS. On Linux/Windows, `Modifiers::CONTROL` maps to `Ctrl`. To use `Cmd+Shift+S` on macOS, use `Modifiers::META | Modifiers::SHIFT`.

**Source**: Tauri 2 global shortcut documentation, tauri-plugin-global-shortcut.

**Impact**: Use `#[cfg(target_os = "macos")]` to select `Modifiers::META` on macOS and `Modifiers::CONTROL` on Linux.

---

## 10. FFmpeg on macOS

**Question**: Is FFmpeg available on macOS and how is it typically installed?

**Finding**: FFmpeg is available via Homebrew (`brew install ffmpeg`). It is not pre-installed on macOS. The binary is typically at `/opt/homebrew/bin/ffmpeg` (Apple Silicon) or `/usr/local/bin/ffmpeg` (Intel).

**Source**: Homebrew documentation.

**Impact**: Add a helpful error message in `processor.rs` when FFmpeg is not found, suggesting `brew install ffmpeg`.
