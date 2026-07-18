# Contract: Platform Detection Interface

**Feature**: `010-macos-compatibility`
**Date**: 2026-07-17

---

## Tauri Command: `get_platform`

### Signature

```rust
#[tauri::command]
pub fn get_platform() -> String
```

### Preconditions

- App is running
- Tauri IPC is available

### Postconditions

- Returns one of: `"macos"`, `"linux"`, `"windows"`
- Return value is a lowercase string
- The return value matches the compilation target

### Error Cases

- None — this function always succeeds

### Usage

```typescript
const platform = await invoke<string>("get_platform");
// platform === "macos" | "linux" | "windows"
```

### Consumers

- `src/components/Settings/AudioSettings.tsx` — Show BlackHole guide on macOS
- `src/components/Settings/ShortcutsSettings.tsx` — Display correct modifier key

---

## Rust Interface: Platform-Conditional Audio Detection

### Signature

```rust
// In src-tauri/src/audio/capture.rs

// Internal classification logic (not a public API, but a contract):
// On macOS: is_monitor = name.contains("BlackHole") || name.contains("Soundflower") || ...
// On Linux: is_monitor = name.contains("Monitor") || name.contains("sink") || ...
// On Windows: is_monitor = name.contains("Stereo Mix") || ...
```

### Preconditions

- `list_devices()` is called
- CPAL host is initialized

### Postconditions

- Each device is classified as `"mic"` or `"system"`
- Classification is deterministic based on device name
- No devices are dropped or duplicated

### Error Cases

- If `host.input_devices()` returns an error, an empty list is returned (existing behavior)

---

## Rust Interface: Platform-Conditional Shortcuts

### Signature

```rust
// In src-tauri/src/lib.rs

// Platform modifier selection:
// macOS: Modifiers::META (Cmd)
// Linux: Modifiers::CONTROL (Ctrl)
// Windows: Modifiers::CONTROL (Ctrl)

// Shortcut registration:
// Shortcut::new(Some(modifier | Modifiers::SHIFT), Code::KeyS)
// Shortcut::new(Some(modifier | Modifiers::SHIFT), Code::KeyO)
// Shortcut::new(Some(modifier | Modifiers::SHIFT), Code::KeyT)
```

### Preconditions

- App is starting up
- `tauri-plugin-global-shortcut` is initialized

### Postconditions

- Shortcuts are registered with the correct platform modifier
- On macOS: `Cmd+Shift+S/O/T`
- On Linux: `Ctrl+Shift+S/O/T`
- Shortcut display strings are updated on macOS

### Error Cases

- If shortcut registration fails, the error is logged but the app continues (existing behavior)
