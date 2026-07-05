# subtitledss — Task Board

## Phase 1: Stability

### Tests
- [ ] Write tests for `audio/buffer.rs` (push, drain, peek, capacity, overflow)
- [ ] Write tests for `vad/detector.rs` (silence, voice, threshold)
- [ ] Write tests for `settings/config.rs` (load, save, defaults, missing file)
- [ ] Write tests for `history/db.rs` (insert, get, search, clear, FTS5)
- [ ] Write tests for `whisper/engine.rs` (new, load_model, is_loaded)
- [ ] Verify all tests pass with `cargo test`

### CI/CD
- [ ] Create `.github/workflows/ci.yml`
- [ ] Add Rust toolchain setup
- [ ] Add Bun setup
- [ ] Add cargo check + cargo test steps
- [ ] Add bun install + typecheck + build steps
- [ ] Add caching for cargo and bun
- [ ] Test CI on first push

### Error Handling
- [ ] Create `src/components/ui/Toast.tsx`
- [ ] Create `src/hooks/useToast.ts`
- [ ] Add toast to `App.tsx` provider
- [ ] Wrap all `invoke()` calls in try/catch with toast
- [ ] Add specific error messages per command
- [ ] Add success toasts for: save, load, start, stop

### Documentation
- [ ] Create `AGENTS.md` with project conventions
- [ ] Update `README.md` with install/usage/contributing
- [ ] Create `.github/CONTRIBUTING.md`

---

## Phase 2: UX

### System Tray
- [ ] Add tray icon (SVG/PNG)
- [ ] Build tray menu (Start/Stop, Show, Quit)
- [ ] Handle tray click → show window
- [ ] Handle close → minimize to tray
- [ ] Update tray tooltip with state
- [ ] Test on Hyprland/Wayland

### Global Shortcuts
- [ ] Add `tauri-plugin-global-shortcut` to Cargo.toml
- [ ] Register Ctrl+Shift+S (toggle capture)
- [ ] Register Ctrl+Shift+O (toggle overlay)
- [ ] Add shortcut config to TOML
- [ ] Add ShortcutsSettings.tsx panel
- [ ] Handle conflict detection

### Model State
- [ ] Add model state to status bar
- [ ] Show "loaded" badge on active model
- [ ] Auto-load on startup (verify)
- [ ] Warning if no model when starting capture
- [ ] Loading spinner during model switch

### VAD Configuration
- [ ] Add VAD threshold to config
- [ ] Add slider in AudioSettings
- [ ] Create EnergyMeter.tsx component
- [ ] Show real-time energy in settings
- [ ] "Test VAD" button

### Status Feedback
- [ ] Capture button spinner during async
- [ ] Overlay "Waiting for speech..." state
- [ ] History tab badge with count
- [ ] Status bar: capture + model + device

---

## Phase 3: Functionality

### Export
- [ ] Create `src-tauri/src/commands/export.rs`
- [ ] Implement SRT formatter
- [ ] Implement VTT formatter
- [ ] Implement TXT formatter
- [ ] Implement JSON formatter
- [ ] Create ExportDialog.tsx
- [ ] Wire export button in HistoryList
- [ ] File save dialog integration

### History
- [ ] Install @tanstack/react-virtual
- [ ] Virtualize HistoryList
- [ ] Implement infinite scroll (load 50 at a time)
- [ ] Group entries by date
- [ ] Expandable entry detail view
- [ ] Individual entry delete
- [ ] Entry count in tab badge

### Overlay
- [ ] Add `data-tauri-drag-region` to overlay
- [ ] Save position to config on drag end
- [ ] Double-click → reset position
- [ ] Right-click context menu
- [ ] Position inputs in Settings → Overlay

### Translation
- [ ] Research candle/ort for NLLB
- [ ] Download NLLB-200 model
- [ ] Implement tokenizer
- [ ] Implement translation pipeline
- [ ] Add to transcription pipeline
- [ ] Create TranslationSettings.tsx
- [ ] Overlay shows translated + original

### Search
- [ ] Debounced search (300ms)
- [ ] Highlight matching text
- [ ] Language filter dropdown
- [ ] Date range filter
- [ ] Context around matches
