# Data Model: macOS Compatibility

**Feature**: `010-macos-compatibility`
**Date**: 2026-07-17

---

## Platform Detection

The app detects the current platform at compile time via `cfg(target_os)` and at runtime for UI display.

### Compile-Time Platform Detection

Used in Rust code for conditional compilation:

```rust
// Audio device naming
#[cfg(target_os = "macos")]
let is_monitor = name.contains("BlackHole") || name.contains("Soundflower");

#[cfg(target_os = "linux")]
let is_monitor = name.contains("Monitor") || name.contains("sink");

// Shortcut modifiers
#[cfg(target_os = "macos")]
let modifier = Modifiers::META;  // Cmd

#[cfg(target_os = "linux")]
let modifier = Modifiers::CONTROL;  // Ctrl

// FFmpeg error hints
#[cfg(target_os = "macos")]
let hint = "\n\nHint: brew install ffmpeg";

#[cfg(target_os = "linux")]
let hint = "";
```

### Runtime Platform Detection

Used in frontend for conditional UI:

```rust
// Tauri command
#[tauri::command]
pub fn get_platform() -> String {
    #[cfg(target_os = "macos")]
    { "macos".to_string() }
    #[cfg(target_os = "linux")]
    { "linux".to_string() }
    #[cfg(target_os = "windows")]
    { "windows".to_string() }
}
```

```typescript
// Frontend usage
const platform = await invoke<string>("get_platform");
if (platform === "macos") {
    // Show BlackHole guide
}
```

---

## Audio Device Classification

Extended to support macOS virtual audio drivers:

| Platform | System Audio Names | Microphone Names |
|----------|-------------------|------------------|
| Linux | "Monitor of ...", "...sink" | Everything else |
| macOS | "BlackHole", "Soundflower", "Aggregate Device" | "Built-in Microphone", everything else |
| Windows | "Stereo Mix" | Everything else |

---

## Keyboard Shortcuts

Platform-aware modifier mapping:

| Platform | Primary Modifier | Shortcut Example |
|----------|-----------------|------------------|
| macOS | `Cmd` (META) | `Cmd+Shift+S` |
| Linux | `Ctrl` (CONTROL) | `Ctrl+Shift+S` |
| Windows | `Ctrl` (CONTROL) | `Ctrl+Shift+S` |

Config storage: The TOML file stores `Ctrl`-based shortcuts. On macOS, the runtime overrides display strings to show `Cmd`.

---

## File Paths

Platform-specific paths (handled by `dirs` crate):

| Platform | Data Dir | Config Dir |
|----------|----------|------------|
| Linux | `~/.local/share/subtitledss/` | `~/.config/subtitledss/` |
| macOS | `~/Library/Application Support/subtitledss/` | `~/Library/Application Support/subtitledss/` |
| Windows | `%APPDATA%/subtitledss/` | `%APPDATA%/subtitledss/` |

---

## Build Targets

| Platform | Audio Backend | GPU Backend | Package Format |
|----------|--------------|-------------|----------------|
| Linux | PipeWire (CPAL) | CUDA / Vulkan | AppImage, .deb |
| macOS | CoreAudio (CPAL) | Metal | .dmg |
| Windows | WASAPI (CPAL) | CUDA | .msi, .exe |

---

## State Transitions

### Audio Device Discovery Flow

```
App Start
  → cpal::default_host()
    → Linux: PipeWire host
    → macOS: CoreAudio host
  → host.input_devices()
    → For each device:
      → #[cfg] classify as "mic" or "system"
      → Add to device list
  → Emit to frontend
```

### Shortcut Registration Flow

```
App Start
  → Load config (TOML with Ctrl shortcuts)
  → #[cfg] Select modifier:
      → macOS: Modifiers::META
      → Linux: Modifiers::CONTROL
  → Register shortcuts with modifier | SHIFT
  → #[cfg] Override display strings on macOS
  → Shortcuts Settings shows correct platform keys
```
