# Feature Specification: macOS Compatibility

**Feature Branch**: `010-macos-compatibility`

**Created**: 2026-07-17

**Status**: Draft

**Input**: User description: "Add full macOS compatibility to subtitledss. The app currently targets Linux (Arch/Wayland/Hyprland) with PipeWire audio. Extend to macOS with CoreAudio, Metal GPU acceleration, platform-aware keyboard shortcuts, DMG packaging, and a first-run guide for system audio capture via BlackHole."

## User Scenarios & Testing

### User Story 1 - App Builds and Runs on macOS (Priority: P1)

As a macOS user, I want to install and run subtitledss on my Mac so that I can use real-time subtitle transcription on macOS.

**Why this priority**: This is the foundation. Without a compilable, runnable app on macOS, no other feature matters. The app must build without errors and launch successfully on macOS 11+.

**Independent Test**: Can be fully tested by running `cargo build` on macOS, launching the app, and verifying the main window appears with the tray icon in the menu bar.

**Acceptance Scenarios**:

1. **Given** a macOS machine with Rust and Bun installed, **When** `cargo build --release` runs in `src-tauri/`, **Then** it compiles without errors (no PipeWire, no Linux-only deps).
2. **Given** the app is built, **When** launched on macOS, **Then** the main window appears and the system tray icon shows in the menu bar.
3. **Given** the app is running on macOS, **When** the user opens Settings, **Then** all tabs (Audio, Whisper, Translation, Theme, Shortcuts) render correctly.
4. **Given** the app is running on macOS, **When** the user closes the main window, **Then** the app continues running in the menu bar (not killed).

---

### User Story 2 - Audio Capture Works on macOS (Priority: P1)

As a macOS user, I want to capture microphone audio for transcription so that the core feature works on my Mac.

**Why this priority**: Audio capture is the core input mechanism. Without it, transcription cannot happen. CoreAudio is the native macOS audio backend and CPAL supports it natively.

**Independent Test**: Can be tested by selecting a microphone device, starting capture, speaking, and verifying audio level meter responds and transcription segments appear.

**Acceptance Scenarios**:

1. **Given** the app is running on macOS, **When** the user opens Audio Settings, **Then** the device list shows macOS audio devices (Built-in Microphone, etc.) with correct channel/rate info.
2. **Given** a microphone is selected, **When** the user starts capture, **Then** the audio level meter responds to speech in real time.
3. **Given** capture is running, **When** the user speaks, **Then** transcription segments appear in the overlay (assuming a Whisper model is loaded).
4. **Given** capture is running, **When** the user stops capture, **Then** the audio stream is cleanly released.

---

### User Story 3 - System Audio Capture Guide (Priority: P1)

As a macOS user, I want a clear guide on how to set up system audio capture (BlackHole) so that I can transcribe system audio, not just microphone.

**Why this priority**: macOS does not expose system audio to apps directly — a virtual audio driver (BlackHole) is required. Without a guide, users will be confused about why "system audio" doesn't appear in the device list.

**Independent Test**: Can be tested by opening the app for the first time on macOS and verifying a setup guide or tooltip explains the BlackHole requirement.

**Acceptance Scenarios**:

1. **Given** the app is running on macOS for the first time, **When** the user views Audio Settings and selects "System" as audio source, **Then** a message explains that macOS requires BlackHole for system audio capture with a link to the installation guide.
2. **Given** BlackHole is installed, **When** the user refreshes the device list, **Then** BlackHole devices appear and can be selected.
3. **Given** BlackHole is configured as both input and output, **When** the user starts system audio capture, **Then** audio from other apps is captured and transcribed.

---

### User Story 4 - GPU Acceleration via Metal (Priority: P2)

As a macOS user with Apple Silicon (M1/M2/M3/M4), I want whisper.cpp to use Metal GPU acceleration so that transcription is fast and efficient.

**Why this priority**: Apple Silicon Metal acceleration provides significant speedup over CPU-only inference. This is a key performance differentiator on macOS.

**Independent Test**: Can be tested by building with the `metal` feature, loading a Whisper model, and verifying GPU-accelerated transcription with lower CPU usage than CPU-only mode.

**Acceptance Scenarios**:

1. **Given** the app is built with `--features metal`, **When** a Whisper model is loaded, **Then** the engine initializes with Metal backend (logged in tracing).
2. **Given** Metal acceleration is active, **When** transcription runs, **Then** CPU usage is lower than CPU-only mode and latency is within the <500ms target.
3. **Given** the app is built without the `metal` feature, **When** a Whisper model is loaded, **Then** it falls back to CPU-only inference without errors.

---

### User Story 5 - Platform-Aware Keyboard Shortcuts (Priority: P2)

As a macOS user, I want keyboard shortcuts to use `Cmd` (⌘) instead of `Ctrl` so that shortcuts follow macOS conventions.

**Why this priority**: macOS users expect `Cmd+Shift+S` not `Ctrl+Shift+S`. Using `Ctrl` on macOS feels foreign and may conflict with system shortcuts.

**Independent Test**: Can be tested by pressing `Cmd+Shift+S` on macOS and verifying capture toggles, and verifying the Shortcuts Settings tab shows `Cmd` modifiers.

**Acceptance Scenarios**:

1. **Given** the app is running on macOS, **When** the user opens Shortcuts Settings, **Then** shortcuts display as `Cmd+Shift+S`, `Cmd+Shift+O`, `Cmd+Shift+T`.
2. **Given** shortcuts use `Cmd` on macOS, **When** the user presses `Cmd+Shift+S`, **Then** capture toggles on/off.
3. **Given** shortcuts use `Cmd` on macOS, **When** the user presses `Cmd+Shift+O`, **Then** the overlay toggles on/off.
4. **Given** the app is running on Linux, **When** the user opens Shortcuts Settings, **Then** shortcuts still display as `Ctrl+Shift+S` (no regression).

---

### User Story 6 - DMG Packaging and Distribution (Priority: P2)

As a macOS user, I want to install subtitledss via a `.dmg` file so that installation is a standard macOS experience (drag to Applications).

**Why this priority**: DMG is the standard macOS distribution format. Without it, users must use terminal commands to run the app.

**Independent Test**: Can be tested by building the DMG, opening it, dragging the app to Applications, and launching from there.

**Acceptance Scenarios**:

1. **Given** the app is built with `cargo tauri build`, **When** the build completes, **Then** a `.dmg` file is produced in `src-tauri/target/release/bundle/dmg/`.
2. **Given** the DMG is opened, **When** the user drags the app to Applications, **Then** the app is installed and appears in Launchpad.
3. **Given** the app is installed in Applications, **When** launched, **Then** it runs correctly with menu bar icon and all features functional.

---

### User Story 7 - Overlay Works on macOS (Priority: P3)

As a macOS user, I want the subtitle overlay to appear as a transparent always-on-top window so that subtitles are visible over other applications.

**Why this priority**: The overlay is a core UX element. Tauri's transparent window feature is cross-platform, so this should work with minimal changes, but needs verification.

**Independent Test**: Can be tested by starting transcription and verifying the overlay window appears on top of other apps with transparent background.

**Acceptance Scenarios**:

1. **Given** transcription is running, **When** text is produced, **Then** the overlay window appears as a transparent always-on-top window.
2. **Given** the overlay is visible, **When** the user drags it, **Then** the overlay moves to the new position.
3. **Given** the overlay is visible, **When** the user toggles it off, **Then** the overlay hides and does not appear in the Dock or Taskbar.

---

### User Story 8 - CI/CD Release Builds for macOS (Priority: P2)

As a maintainer, I want the GitHub Actions release workflow to build macOS artifacts (DMG + tar.gz) alongside Linux artifacts so that macOS users can download pre-built binaries from GitHub Releases.

**Why this priority**: Without CI/CD macOS builds, users must compile from source on macOS. Automated DMG builds ensure consistent, reproducible releases and lower the barrier to adoption.

**Independent Test**: Can be tested by pushing a `v*` tag, verifying the release workflow produces Linux + macOS artifacts, and downloading the DMG from the GitHub Release.

**Acceptance Scenarios**:

1. **Given** a `v*` tag is pushed, **When** the release workflow runs, **Then** both Linux and macOS build jobs run in parallel.
2. **Given** the macOS build job completes, **When** artifacts are collected, **Then** a `.dmg` file and a `.tar.gz` (raw binary) for macOS are produced.
3. **Given** all builds complete, **When** the GitHub Release is created, **Then** it contains 5 artifacts: Linux AppImage, Linux .deb, Linux .tar.gz, macOS .dmg, macOS .tar.gz.
4. **Given** the GitHub Release body is generated, **When** the user reads it, **Then** installation instructions for both Linux and macOS are present.
5. **Given** the macOS build uses `macos-latest` runner, **When** the build runs, **Then** it compiles natively on Apple Silicon with Metal support.

---

### User Story 9 - FFmpeg Availability Check (Priority: P3)

As a macOS user, I want the app to check for FFmpeg on startup and show a helpful message if it's missing, so that video transcription works or I know how to fix it.

**Why this priority**: FFmpeg is not pre-installed on macOS. Video transcription depends on it. A clear error message prevents user confusion.

**Independent Test**: Can be tested by removing FFmpeg from PATH, attempting video transcription, and verifying a user-friendly error message appears.

**Acceptance Scenarios**:

1. **Given** FFmpeg is not installed, **When** the user attempts video transcription, **Then** an error toast says "FFmpeg not found. Install with: brew install ffmpeg".
2. **Given** FFmpeg is installed via Homebrew, **When** the user attempts video transcription, **Then** the pipeline runs successfully.
3. **Given** the app starts on macOS, **When** FFmpeg is missing, **Then** a non-blocking warning appears in the Audio/Video settings tab.

---

### Edge Cases

- What happens if the user denies microphone permission? The app shows an error toast explaining that microphone access is required in System Preferences > Privacy & Security > Microphone.
- What happens if BlackHole is installed but not configured as the output device? The device list shows BlackHole but capture produces silence — a tooltip warns the user to set BlackHole as the system output.
- What happens if the Mac has no Metal-capable GPU (older Intel Macs)? The app falls back to CPU-only inference with no errors. The GPU settings toggle is disabled or shows "Not available".
- What happens if the user has both BlackHole and Soundflower installed? Both appear in the device list. The user can select either. No conflict.
- What happens on macOS 10.x (Catalina and earlier)? The app does not support macOS < 11.0 (Big Sur). The DMG includes a minimum OS check.
- What happens if the user tries to build on macOS without Xcode command line tools? `cargo build` fails with a clear error about missing clang/cc. The quickstart guide documents this.
- What happens if the overlay window is behind fullscreen apps? macOS handles always-on-top differently with Spaces. The overlay may not appear over fullscreen apps — this is a known macOS limitation documented in the README.
- What happens if the macOS CI build fails but Linux succeeds? The release job waits for ALL build jobs — if any fails, no release is created. The developer must fix the macOS build before releasing.
- What happens if the macOS runner is slow? GitHub's `macos-latest` (Apple Silicon) typically builds in 5-10 minutes. If it exceeds the 6-hour job timeout, the workflow fails and must be investigated.

## Requirements

### Functional Requirements

**Build System & Dependencies**

- **FR-001**: `src-tauri/Cargo.toml` MUST use platform-conditional dependencies: `cpal` with `pipewire` feature on Linux only, plain `cpal` on macOS.
- **FR-002**: `src-tauri/Cargo.toml` MUST add `whisper-rs` with `metal` feature on macOS via `[target.'cfg(target_os = "macos")'.dependencies]`.
- **FR-003**: `src-tauri/Cargo.toml` MUST define a `metal` feature flag: `metal = ["whisper-rs/metal"]`.
- **FR-004**: The `gpu` feature flag MUST default to `cuda` on Linux and `metal` on macOS (or remain Linux-only with `metal` as explicit opt-in).
- **FR-005**: `cargo build` on macOS MUST compile without errors when `pipewire` feature is not activated.

**Audio Capture**

- **FR-006**: `src-tauri/src/audio/capture.rs` MUST use `cpal::default_host()` which returns CoreAudio on macOS — no code changes needed for basic capture.
- **FR-007**: `src-tauri/src/audio/capture.rs` MUST detect macOS audio devices using platform-appropriate naming (CoreAudio device names, not PipeWire "Monitor"/"sink" names).
- **FR-008**: `src-tauri/src/audio/capture.rs` MUST classify audio devices on macOS: built-in microphone as "mic", BlackHole/Soundflower as "system".
- **FR-009**: The `to_mono()` and `simple_resample()` functions in `capture.rs` MUST remain unchanged (pure math, cross-platform).
- **FR-010**: The app MUST request microphone permission on first launch via macOS entitlements or Tauri's permission system.

**System Audio Capture**

- **FR-011**: The Audio Settings UI MUST display a platform-specific message on macOS explaining that system audio capture requires BlackHole.
- **FR-012**: The message MUST include a link to the BlackHole GitHub repository (https://github.com/ExistentialAudio/BlackHole).
- **FR-013**: The device list MUST include BlackHole devices when they are installed and available in CoreAudio.

**GPU Acceleration**

- **FR-014**: `whisper-rs` with `metal` feature MUST be available on macOS builds.
- **FR-015**: The `WhisperEngine` in `src-tauri/src/whisper/engine.rs` MUST work with Metal backend without code changes (whisper-rs handles backend selection internally).
- **FR-016**: The GPU toggle in Whisper Settings MUST be functional on macOS when Metal is available.
- **FR-017**: On Intel Macs without Metal support, the GPU toggle MUST be disabled or hidden with an explanatory tooltip.

**Keyboard Shortcuts**

- **FR-018**: `src-tauri/src/lib.rs` MUST use `Modifiers::META` (Cmd) on macOS and `Modifiers::CONTROL` (Ctrl) on Linux for global shortcuts.
- **FR-019**: `config/default.toml` MUST have platform-specific default shortcut strings: `Cmd+Shift+S/O/T` on macOS, `Ctrl+Shift+S/O/T` on Linux.
- **FR-020**: The Shortcuts Settings UI MUST display the correct platform modifier key.

**Window Management & Overlay**

- **FR-021**: The overlay window (`transparent: true`, `alwaysOnTop: true`, `decorations: false`) MUST work on macOS via Tauri's cross-platform window management.
- **FR-022**: The overlay MUST appear in all macOS Spaces by default (or respect user's Space settings).
- **FR-023**: The system tray icon MUST appear in the macOS menu bar (top-right area).

**Packaging & Distribution**

- **FR-024**: `tauri.conf.json` MUST include `macOS` bundle configuration with `minimumSystemVersion: "11.0"`.
- **FR-025**: The DMG configuration MUST define app and Applications folder positions.
- **FR-026**: The app bundle MUST include an `Info.plist` with proper `CFBundleIdentifier`, `LSMinimumSystemVersion`, and microphone `NSMicrophoneUsageDescription`.
- **FR-027**: The icon set MUST include `.icns` format for macOS (already present in `src-tauri/icons/icon.icns`).

**FFmpeg**

- **FR-028**: `src-tauri/src/video/processor.rs` MUST provide a user-friendly error message on macOS when FFmpeg is not found, suggesting `brew install ffmpeg`.
- **FR-029**: The video transcription UI MUST show a warning if FFmpeg is not available.

**Configuration & Paths**

- **FR-030**: `dirs::data_dir()` on macOS returns `~/Library/Application Support/subtitledss/` — this is correct behavior, no changes needed.
- **FR-031**: The TOML config file on macOS MUST be at `~/Library/Application Support/subtitledss/config.toml` (handled by `dirs` crate).
- **FR-032**: The SQLite database on macOS MUST be at `~/Library/Application Support/subtitledss/history.db` (handled by `dirs` crate).

**CI/CD Release Workflow**

- **FR-033**: `.github/workflows/release.yml` MUST add a `build-macos` job running on `macos-latest`.
- **FR-034**: The `build-macos` job MUST install Bun, Rust stable, and build the frontend.
- **FR-035**: The `build-macos` job MUST build the Rust backend with `--release` and `--features metal` for Apple Silicon GPU acceleration.
- **FR-036**: The `build-macos` job MUST produce a `.dmg` file via `cargo tauri build`.
- **FR-037**: The `build-macos` job MUST produce a `.tar.gz` raw binary archive for users who prefer manual installation.
- **FR-038**: The `create-release` job MUST depend on both `build-linux` AND `build-macos` — release is not created until both succeed.
- **FR-039**: The `create-release` job MUST download artifacts from both build jobs and attach all 5 artifacts to the GitHub Release.
- **FR-040**: The release body MUST include macOS installation instructions (DMG drag-to-install + quarantine removal command).
- **FR-041**: The release body MUST mention that macOS builds include Metal GPU acceleration for Apple Silicon.

### Key Entities

- **PlatformConfig**: Runtime detection of OS platform (Linux/macOS) affecting audio backend, shortcuts, and device naming.
- **AudioDevice** (extended): Now includes macOS-specific device classification (CoreAudio device names, BlackHole detection).
- **ShortcutModifier**: Platform-aware modifier key — `Cmd` on macOS, `Ctrl` on Linux.

## Success Criteria

### Measurable Outcomes

- **SC-001**: `cargo build --release` compiles successfully on macOS 11+ with zero errors.
- **SC-002**: `cargo build --release --features metal` compiles successfully on macOS with Metal support.
- **SC-003**: The app launches on macOS and displays the main window + menu bar icon within 3 seconds.
- **SC-004**: Microphone audio capture works with <500ms latency on macOS (CoreAudio).
- **SC-005**: With Metal enabled, Whisper Base model transcription achieves <500ms latency on Apple Silicon.
- **SC-006**: `Cmd+Shift+S` toggles capture on macOS; `Ctrl+Shift+S` still works on Linux (no regression).
- **SC-007**: The DMG builds and installs correctly — app appears in Launchpad and runs from /Applications.
- **SC-008**: The overlay window appears as transparent and always-on-top on macOS.
- **SC-009**: `cargo test` passes on macOS with no regressions.
- **SC-010**: `bun run typecheck` passes with no TypeScript errors.
- **SC-011**: The BlackHole setup guide is visible in Audio Settings when "System" source is selected on macOS.
- **SC-012**: Video transcription shows a helpful error when FFmpeg is not installed.
- **SC-013**: Pushing a `v*` tag produces 5 release artifacts: Linux AppImage, .deb, .tar.gz + macOS DMG, .tar.gz.
- **SC-014**: The release workflow completes in under 15 minutes (Linux + macOS in parallel).
- **SC-015**: macOS DMG is downloadable from GitHub Releases and installs correctly on a fresh macOS machine.

## Assumptions

- CPAL natively supports CoreAudio on macOS — no additional audio backend configuration is needed.
- `whisper-rs` 0.16 supports the `metal` feature for Apple Silicon GPU acceleration.
- `dirs` crate version 5 correctly resolves macOS Application Support paths.
- Tauri 2 handles macOS menu bar tray icons natively without additional configuration.
- Tauri 2's transparent window feature works on macOS (it uses WKWebView which supports transparent backgrounds).
- BlackHole 2ch is the recommended virtual audio driver for system audio capture on macOS.
- Apple Silicon Macs (M1+) have Metal support. Intel Macs may or may not have Metal (depends on GPU model).
- macOS 11.0 (Big Sur) is a reasonable minimum — it's the oldest version with reliable Metal support and Tauri 2 compatibility.
- FFmpeg can be installed on macOS via Homebrew (`brew install ffmpeg`).
- The `.icns` icon file already exists in `src-tauri/icons/` — no new icon generation needed.
- The existing overlay implementation (Tauri transparent window) is fully cross-platform and requires no macOS-specific changes.
- Code signing and notarization are out of scope for v1 — the app can run unsigned on macOS with user override.
