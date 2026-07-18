# Tasks: macOS Compatibility

**Feature**: `010-macos-compatibility`
**Generated**: 2026-07-17
**Spec**: `spec.md` | **Plan**: `plan.md`

---

## Phase 1: Build System & Dependencies

> Goal: Make `cargo build` compile on macOS without PipeWire link errors.
> Dependencies: None — foundation layer.

### 1A. Platform-Conditional cpal

- [ ] [T-1A.1] Replace unconditional `cpal` dependency with platform-conditional deps in `src-tauri/Cargo.toml`
  - **File**: `src-tauri/Cargo.toml`
  - **Spec Ref**: FR-001, FR-005
  - **Details**: Move `cpal = { version = "0.18", features = ["pipewire"] }` to `[target.'cfg(target_os = "linux")'.dependencies]`. Add `cpal = { version = "0.18" }` under `[target.'cfg(target_os = "macos")'.dependencies]` and `[target.'cfg(target_os = "windows")'.dependencies]`.
  - **Verification**: `cargo check` on macOS passes (no PipeWire link errors)

### 1B. Metal Feature Flag

- [ ] [T-1B.1] Add `metal` feature flag to `src-tauri/Cargo.toml`
  - **File**: `src-tauri/Cargo.toml`
  - **Spec Ref**: FR-003, FR-014
  - **Details**: Add `metal = ["whisper-rs/metal"]` to `[features]` section.
  - **Verification**: `cargo check` passes

- [ ] [T-1B.2] Make `whisper-rs` platform-conditional in `src-tauri/Cargo.toml`
  - **File**: `src-tauri/Cargo.toml`
  - **Spec Ref**: FR-002, FR-014
  - **Details**: Move `whisper-rs` to platform-conditional sections. macOS gets `features = ["tracing_backend", "metal"]`. Linux keeps `features = ["tracing_backend"]`.
  - **Verification**: `cargo check` passes on both platforms

### 1C. Update Description

- [ ] [T-1C.1] Update package description in `src-tauri/Cargo.toml`
  - **File**: `src-tauri/Cargo.toml`
  - **Spec Ref**: —
  - **Details**: Change `description = "Real-time subtitle overlay for Linux"` to `"Real-time subtitle overlay for Linux and macOS"`.
  - **Verification**: `cargo check` passes

### Phase 1 Verification Gate

```bash
cargo check          # macOS
cargo check          # Linux — no regression
```

---

## Phase 2: Audio & Platform Detection

> Goal: Audio capture works on macOS, FFmpeg error is helpful, BlackHole guide shows.
> Dependencies: Phase 1 complete.

### 2A. macOS Device Detection

- [ ] [T-2A.1] Add `cfg(target_os)` device classification in `src-tauri/src/audio/capture.rs`
  - **File**: `src-tauri/src/audio/capture.rs`
  - **Spec Ref**: FR-007, FR-008
  - **Details**: Replace the single `is_monitor` check (lines 77-81) with platform-conditional blocks. macOS checks for "BlackHole", "Soundflower", "Aggregate Device", "System Audio". Linux keeps existing "Monitor"/"sink" checks. Add Windows "Stereo Mix" check.
  - **Verification**: `cargo check` passes on all platforms; `cargo test` passes

### 2B. FFmpeg Error Message

- [ ] [T-2B.1] Add macOS-specific FFmpeg hint in `src-tauri/src/video/processor.rs`
  - **File**: `src-tauri/src/video/processor.rs`
  - **Spec Ref**: FR-028
  - **Details**: Add `#[cfg(target_os = "macos")]` hint string `"\n\nHint: Install FFmpeg on macOS with: brew install ffmpeg"`. Append to error messages in `extract_audio()` and `get_duration()`.
  - **Verification**: `cargo check` passes

### 2C. BlackHole Guide in UI

- [ ] [T-2C.1] Add platform detection command `get_platform` in `src-tauri/src/commands/settings.rs`
  - **File**: `src-tauri/src/commands/settings.rs`
  - **Spec Ref**: FR-011
  - **Details**: Add `#[tauri::command] pub fn get_platform() -> String` that returns `"macos"`, `"linux"`, or `"windows"` based on `cfg!(target_os)`.
  - **Verification**: `cargo check` passes

- [ ] [T-2C.2] Register `get_platform` command in `src-tauri/src/lib.rs`
  - **File**: `src-tauri/src/lib.rs`
  - **Spec Ref**: FR-011
  - **Details**: Add `commands::settings::get_platform` to the `invoke_handler` list.
  - **Verification**: `cargo check` passes

- [ ] [T-2C.3] Add BlackHole setup guide in `src/components/Settings/AudioSettings.tsx`
  - **File**: `src/components/Settings/AudioSettings.tsx`
  - **Spec Ref**: FR-011, FR-012, FR-013
  - **Details**: When audio source is "System" and platform is "darwin", show a yellow info box explaining BlackHole requirement with a link to https://github.com/ExistentialAudio/BlackHole. Use `invoke("get_platform")` to detect OS.
  - **Verification**: `bun run typecheck` passes; manual: macOS shows guide

### Phase 2 Verification Gate

```bash
cargo check
cargo test
bun run typecheck
bun run lint
```

---

## Phase 3: Keyboard Shortcuts

> Goal: `Cmd` on macOS, `Ctrl` on Linux for global shortcuts.
> Dependencies: Phase 1 complete (parallel with Phase 2).

### 3A. Platform-Aware Shortcuts in lib.rs

- [ ] [T-3A.1] Add platform-conditional modifier key in `src-tauri/src/lib.rs`
  - **File**: `src-tauri/src/lib.rs`
  - **Spec Ref**: FR-018
  - **Details**: Replace hardcoded `Modifiers::CONTROL` with platform-conditional `modifier` variable. macOS: `Modifiers::META`. Linux/Windows: `Modifiers::CONTROL`. Use `#[cfg(target_os)]` blocks. Apply to all three shortcut definitions (Ctrl+Shift+S/O/T).
  - **Verification**: `cargo check` passes on both platforms

- [ ] [T-3A.2] Override shortcut display strings on macOS at runtime in `src-tauri/src/lib.rs`
  - **File**: `src-tauri/src/lib.rs`
  - **Spec Ref**: FR-019, FR-020
  - **Details**: After loading config, `#[cfg(target_os = "macos")]` override `config.shortcuts.toggle_*` strings to use "Cmd" prefix for display purposes.
  - **Verification**: `cargo check` passes

### Phase 3 Verification Gate

```bash
cargo check
cargo test
bun run typecheck
```

---

## Phase 4: Packaging & Distribution

> Goal: DMG builds on macOS, microphone entitlement is configured.
> Dependencies: Phase 1-3 complete.

### 4A. macOS Bundle Config

- [ ] [T-4A.1] Add `macOS` section to `bundle` in `src-tauri/tauri.conf.json`
  - **File**: `src-tauri/tauri.conf.json`
  - **Spec Ref**: FR-024, FR-025
  - **Details**: Add `macOS.minimumSystemVersion: "11.0"`, `macOS.dmg` with app/folder positions and window size. Set `signingIdentity: null` for unsigned builds.
  - **Verification**: `cargo tauri build` produces `.dmg` on macOS

### 4B. Microphone Entitlement

- [ ] [T-4B.1] Create `src-tauri/macOS.entitlements` with microphone entitlement
  - **File**: `src-tauri/macOS.entitlements` (NEW)
  - **Spec Ref**: FR-010
  - **Details**: XML plist with `com.apple.security.device.audio-input` set to `true`.
  - **Verification**: File exists and is valid XML

- [ ] [T-4B.2] Reference entitlements file in `src-tauri/tauri.conf.json`
  - **File**: `src-tauri/tauri.conf.json`
  - **Spec Ref**: FR-010
  - **Details**: Set `macOS.entitlements` to `"./macOS.entitlements"`.
  - **Verification**: `cargo tauri build` includes entitlements in app bundle

### Phase 4 Verification Gate

```bash
cargo tauri build     # On macOS — produces .dmg
```

---

## Phase 5: CI/CD Release Workflow

> Goal: Add macOS build job to release workflow and CI check.
> Dependencies: Phase 1-4 complete.

### 5A. Update release.yml

- [ ] [T-6A.1] Add `build-macos` job to `.github/workflows/release.yml`
  - **File**: `.github/workflows/release.yml`
  - **Spec Ref**: FR-033, FR-034, FR-035, FR-036, FR-037
  - **Details**: New job on `macos-latest`. Steps: checkout, setup Bun 1.3.12, setup Rust stable, rust-cache, `bun install --frozen-lockfile`, `bun run build`, `cargo build --release --features metal` in `src-tauri/`, `cargo tauri build` in `src-tauri/`, tar.gz of raw binary, upload artifacts as `macos-builds`.
  - **Verification**: YAML is valid; job appears in workflow

- [ ] [T-6A.2] Update `create-release` job to depend on both build jobs in `.github/workflows/release.yml`
  - **File**: `.github/workflows/release.yml`
  - **Spec Ref**: FR-038
  - **Details**: Change `needs: build-linux` to `needs: [build-linux, build-macos]`. Add step to download `macos-builds` artifact. Add macOS DMG and .tar.gz to the `files:` list.
  - **Verification**: YAML is valid; release waits for both builds

- [ ] [T-6A.3] Update release body with macOS installation instructions in `.github/workflows/release.yml`
  - **File**: `.github/workflows/release.yml`
  - **Spec Ref**: FR-040, FR-041
  - **Details**: Add macOS DMG section (drag to Applications, right-click Open for unsigned, microphone permission). Add macOS manual install section. Add BlackHole note. Add Metal GPU acceleration note. Update title to "Linux and macOS".
  - **Verification**: YAML is valid; release body renders correctly

### 5B. Update CI Workflow

- [ ] [T-6B.1] Add `rust-check-macos` job to `.github/workflows/ci.yml`
  - **File**: `.github/workflows/ci.yml`
  - **Spec Ref**: —
  - **Details**: New job on `macos-latest`. Steps: checkout, setup Bun, setup Rust, rust-cache, `bun install`, `bun run build`, `cargo check --all-targets` in `src-tauri/`. This catches macOS compilation errors on every push/PR.
  - **Verification**: YAML is valid; job runs on next push

### Phase 5 Verification Gate

```bash
# Validate YAML
cat .github/workflows/release.yml
cat .github/workflows/ci.yml

# On macOS:
cargo build --release --features metal
cargo tauri build
```

---

## Phase 6: Polish & Verification

> Goal: End-to-end verification, documentation, edge cases.
> Dependencies: Phase 1-5 complete.

### 6A. End-to-End Verification

- [ ] [T-6A.1] Verify audio capture works on macOS with built-in microphone
  - **Verification**: Select mic → start capture → audio level responds → transcription appears

- [ ] [T-6A.2] Verify BlackHole detection when installed
  - **Verification**: Install BlackHole → refresh device list → BlackHole appears as "system" device

- [ ] [T-6A.3] Verify Metal GPU acceleration on Apple Silicon
  - **Verification**: Build with `--features metal` → load model → transcription uses GPU (check Activity Monitor)

- [ ] [T-6A.4] Verify overlay window on macOS
  - **Verification**: Start transcription → overlay appears transparent and always-on-top → can be dragged

- [ ] [T-6A.5] Verify `Cmd+Shift+S` toggles capture on macOS
  - **Verification**: Press shortcut → capture starts/stops

- [ ] [T-6A.6] Verify DMG installs correctly
  - **Verification**: Open DMG → drag to Applications → launch from Launchpad → app works

- [ ] [T-6A.7] Verify FFmpeg error message when missing
  - **Verification**: Uninstall FFmpeg → try video transcription → error suggests `brew install ffmpeg`

### 5B. Regression Verification (Linux)

- [ ] [T-6B.1] Verify Linux build is unchanged
  - **Verification**: `cargo check` on Linux passes; `cargo test` passes; shortcuts use `Ctrl`

- [ ] [T-6B.2] Verify Linux audio capture still works
  - **Verification**: PipeWire devices appear correctly; system audio capture works

### 5C. Documentation

- [ ] [T-6C.1] Update `README.md` with macOS installation instructions
  - **File**: `README.md`
  - **Details**: Add "macOS" section with: prerequisites (Rust, Bun, Xcode CLI tools, FFmpeg optional), build instructions, DMG installation, BlackHole setup for system audio.
  - **Verification**: README renders correctly

### Phase 5 Verification Gate

```bash
# Full verification
cargo check
cargo test
bun run typecheck
bun run lint
bun run build
```

---

## Dependency Graph

```
Phase 1 (Build System)
  ├─ T-1A.1: cpal platform-conditional    ─── standalone
  ├─ T-1B.1: metal feature flag           ─── standalone
  ├─ T-1B.2: whisper-rs platform-conditional ─── depends on T-1B.1
  └─ T-1C.1: description update           ─── standalone

Phase 2 (Audio & Platform)
  ├─ T-2A.1: capture.rs detection         ─── depends on T-1A.1
  ├─ T-2B.1: processor.rs FFmpeg hint     ─── standalone
  ├─ T-2C.1: get_platform command         ─── standalone
  ├─ T-2C.2: register get_platform        ─── depends on T-2C.1
  └─ T-2C.3: AudioSettings.tsx guide      ─── depends on T-2C.2

Phase 3 (Shortcuts)
  ├─ T-3A.1: lib.rs modifier key          ─── depends on T-1A.1
  └─ T-3A.2: lib.rs display override      ─── depends on T-3A.1

Phase 4 (Packaging)
  ├─ T-4A.1: tauri.conf.json macOS bundle ─── depends on T-1A.1, T-1B.1
  ├─ T-4B.1: macOS.entitlements file      ─── standalone
  └─ T-4B.2: reference entitlements       ─── depends on T-4B.1, T-4A.1

Phase 5 (CI/CD)
  ├─ T-5A.1: release.yml macOS job        ─── depends on T-1A.1, T-1B.1, T-4A.1
  ├─ T-5A.2: release.yml update needs     ─── depends on T-5A.1
  ├─ T-5A.3: release.yml macOS instructions ─── depends on T-5A.2
  └─ T-5B.1: ci.yml macOS check           ─── depends on T-1A.1, T-1B.1

Phase 6 (Polish)
  ├─ T-6A.1-6A.7: e2e verification       ─── depends on all above
  ├─ T-6B.1-6B.2: Linux regression       ─── depends on all above
  └─ T-6C.1: README update               ─── depends on all above
```

---

## Task Summary

| Phase | Tasks | Key Files |
|-------|-------|-----------|
| **Phase 1** | 4 tasks | `Cargo.toml` |
| **Phase 2** | 5 tasks | `capture.rs`, `processor.rs`, `settings.rs`, `lib.rs`, `AudioSettings.tsx` |
| **Phase 3** | 2 tasks | `lib.rs` |
| **Phase 4** | 3 tasks | `tauri.conf.json`, `macOS.entitlements` |
| **Phase 5** | 4 tasks | `release.yml`, `ci.yml` |
| **Phase 6** | 9 tasks | `README.md`, verification |
| **Total** | **27 tasks** | |

---

## Verification Commands (Final)

```bash
# Rust — both platforms
cargo check
cargo test
cargo clippy

# Frontend
bun run typecheck
bun run lint
bun run build

# macOS-specific
cargo tauri build     # Produces .dmg
xattr -cr target/release/bundle/dmg/*.dmg  # Remove quarantine

# Linux-specific (regression check)
cargo check           # pipewire feature still active
cargo test

# CI/CD validation
cat .github/workflows/release.yml   # Valid YAML with macOS job
cat .github/workflows/ci.yml        # Valid YAML with macOS check

# Release test (on macOS)
git tag v1.2.0-test
git push --tags
# → Wait for both build-linux and build-macos to complete
# → Verify 5 artifacts in GitHub Release
```
