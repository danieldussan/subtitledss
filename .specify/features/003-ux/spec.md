# Feature: Phase 2 — UX Improvements

## Goal
Make the app feel native and polished with system tray, global shortcuts, and better model state management.

## User Stories

### US-1: System Tray Integration
**As a** user
**I want** the app to minimize to system tray
**So that** it runs unobtrusively in the background

**Acceptance Criteria:**
- [ ] App shows system tray icon (use existing `icons/` or generate SVG)
- [ ] Tray menu: Start/Stop Capture, Show/Hide Window, Quit
- [ ] Click tray icon → show main window
- [ ] Close button minimizes to tray (not quit)
- [ ] Tray tooltip shows "subtitledss — Idle" or "subtitledss — Capturing"
- [ ] Works on Hyprland/Wayland with SNI tray

### US-2: Global Keyboard Shortcuts
**As a** user
**I want** keyboard shortcuts that work even when the app isn't focused
**So that** I can control transcription without switching windows

**Acceptance Criteria:**
- [ ] `tauri-plugin-global-shortcut` added to Cargo.toml
- [ ] Ctrl+Shift+S: Toggle capture (global)
- [ ] Ctrl+Shift+O: Toggle overlay visibility (global)
- [ ] Shortcuts configurable in Settings → Shortcuts tab
- [ ] Shortcut conflicts detected and warned
- [ ] Shortcuts disabled when settings panel is open (avoid conflicts)

### US-3: Model State Visibility
**As a** user
**I want** to see if Whisper is loaded and ready
**So that** I know if transcription will work

**Acceptance Criteria:**
- [ ] Status bar shows: "Whisper: tiny (loaded)" or "Whisper: not loaded"
- [ ] If no model loaded, Start button shows warning tooltip
- [ ] Auto-load configured model on app start (already implemented)
- [ ] Model Manager shows which model is currently active
- [ ] Switching models: unload old → load new (with loading spinner)

### US-4: VAD Threshold Configuration
**As a** user
**I want** to adjust voice detection sensitivity
**So that** it works in different noise environments

**Acceptance Criteria:**
- [ ] Settings → Audio → VAD Threshold slider (0.001 — 0.1)
- [ ] Default: 0.01
- [ ] Live preview: show energy level indicator
- [ ] Changes apply on next capture start (not mid-capture)
- [ ] "Test VAD" button: shows real-time energy bar

### US-5: Better Status Feedback
**As a** user
**I want** to understand what's happening at all times
**So that** I'm not confused about app state

**Acceptance Criteria:**
- [ ] Status bar shows: capture state, model state, audio device
- [ ] Capture button shows spinner while starting (async)
- [ ] Overlay shows "Waiting for speech..." when capture active but no speech
- [ ] History shows entry count in tab badge

## Technical Notes

### System Tray (Tauri 2)
```rust
// In lib.rs setup
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState};
use tauri::menu::{Menu, MenuItem};

let tray = TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .tooltip("subtitledss — Idle")
    .menu(&build_tray_menu())
    .on_tray_icon_event(|tray, event| {
        if let MouseButtonState::Up = event.button_state() {
            match event.button() {
                MouseButton::Left => { /* show main window */ }
                _ => {}
            }
        }
    })
    .build(app)?;
```

### Global Shortcuts (Tauri 2)
```rust
// In Cargo.toml
tauri-plugin-global-shortcut = "2"

// In lib.rs
.plugin(tauri_plugin_global_shortcut::Builder::new().build())

// Register shortcuts
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Code, Modifiers};
app.global_shortcut().on_shortcut("CmdOrCtrl+Shift+S", move |app, shortcut, event| {
    // toggle capture
});
```

### Energy Level Indicator
- Ring buffer → calculate RMS energy over last 100ms
- Display as horizontal bar in Settings → Audio
- Color: green (voice), yellow (borderline), red (clipping)
- Updates every 50ms via requestAnimationFrame

### Tray Menu Structure
```
subtitledss
├── ▶ Start Capture / ⏹ Stop Capture
├── ─────────────
├── 👁 Show Overlay
├── 📋 Show Window
├── ─────────────
└── ❌ Quit
```

## File Structure
```
src-tauri/
├── Cargo.toml          (add tauri-plugin-global-shortcut)
├── src/
│   └── lib.rs          (tray + shortcuts setup)

src/
├── components/
│   ├── ui/
│   │   ├── Toast.tsx
│   │   └── EnergyMeter.tsx
│   └── Settings/
│       └── ShortcutsSettings.tsx
└── hooks/
    ├── useToast.ts
    └── useEnergy.ts
```
