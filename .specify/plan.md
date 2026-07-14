# subtitledss — Implementation Plan

## Overview
Three-phase roadmap to take subtitledss from current state (~70% functional) to production-ready release.

---

## Phase 1: Stability (1-2 days)
**Goal:** Ship with confidence — tests, CI, error handling.

| Task | Priority | Effort | Files |
|------|----------|--------|-------|
| Unit tests for buffer, VAD, config, history | High | 4h | `src-tauri/src/**/*_test.rs` |
| CI/CD pipeline (GitHub Actions) | High | 1h | `.github/workflows/ci.yml` |
| Toast notification system | High | 3h | `src/components/ui/Toast.tsx`, `src/hooks/useToast.ts` |
| Error handling in all invoke() calls | High | 2h | All frontend components |
| AGENTS.md | Medium | 30m | `AGENTS.md` |
| README.md with screenshots | Medium | 1h | `README.md` |
| CONTRIBUTING.md | Low | 30m | `.github/CONTRIBUTING.md` |

**Exit Criteria:**
- `cargo test` — all pass
- `cargo clippy` — no warnings
- `bun run typecheck` — clean
- CI green on first push
- Toast errors visible in UI

---

## Phase 2: UX (2-3 days)
**Goal:** Feel like a native Linux app — tray, shortcuts, feedback.

| Task | Priority | Effort | Files |
|------|----------|--------|-------|
| System tray (minimize, menu, quit) | High | 3h | `src-tauri/src/lib.rs` |
| Global shortcuts (Ctrl+Shift+S/O) | High | 2h | `src-tauri/Cargo.toml`, `lib.rs` |
| Model state indicator in status bar | Medium | 1h | `src/App.tsx` |
| VAD threshold slider + energy meter | Medium | 2h | `src/components/Settings/AudioSettings.tsx` |
| Configurable shortcuts in Settings | Low | 2h | `src/components/Settings/ShortcutsSettings.tsx` |
| Tab badges (history count) | Low | 30m | `src/App.tsx` |

**Exit Criteria:**
- Tray icon visible on Hyprland
- Ctrl+Shift+S works from any window
- Status bar shows model + capture state
- VAD slider adjusts sensitivity

---

## Phase 3: Functionality (5-7 days)
**Goal:** Complete feature set — export, translation, history browsing.

| Task | Priority | Effort | Files |
|------|----------|--------|-------|
| Export SRT/VTT/TXT/JSON | High | 4h | `src-tauri/src/commands/export.rs`, `src/utils/export.ts` |
| Export dialog with format picker | High | 2h | `src/components/Export/ExportDialog.tsx` |
| Virtualized history (infinite scroll) | Medium | 4h | `src/components/History/HistoryList.tsx` |
| History grouping by date | Medium | 1h | `src/components/History/HistoryList.tsx` |
| Draggable overlay | Medium | 2h | `public/overlay.html`, `src-tauri/tauri.conf.json` |
| NLLB-200 translation | Low | 8h | `src-tauri/src/translation/` |
| **Whisper translate + LibreTranslate** | **High** | **5h** | `src-tauri/src/translation/`, `pipeline/transcriber.rs` |
| **Accumulative overlay with fade** | **High** | **4h** | `public/overlay.html`, `tauri.conf.json` |
| Translation settings UI | Low | 2h | `src/components/Settings/TranslationSettings.tsx` |
| History search improvements | Low | 2h | `src/components/History/HistoryList.tsx` |

**Exit Criteria:**
- Export .srt from History → opens file dialog → file created
- History scrolls smoothly with 1000+ entries
- Overlay draggable, position saved
- Translation works (if model downloaded)

---

## Dependencies

### New Rust Crates (Phase 3)
```toml
# NLLB approach (alternative)
candle-core = "0.8"
candle-nn = "0.8"
tokenizers = "0.21"

# Whisper/LibreTranslate approach (feature 005) — NO NEW DEPS
# Uses existing: whisper-rs 0.16 (set_translate), reqwest 0.12 (LibreTranslate API)
```

### New npm Packages (Phase 2-3)
```json
{
  "@tauri-plugin-global-shortcut": "^2",
  "@tanstack/react-virtual": "^3"
}
```

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| NLLB inference too slow on CPU | High | Defer to Phase 4, use smaller model |
| Overlay transparent window broken on Hyprland | High | Test early, fallback to CSS-only overlay |
| Global shortcuts conflict with DE | Medium | Make configurable, provide defaults |
| SQLite FTS5 performance with 10k+ entries | Low | Add index, paginate |
| **Whisper translate latency** | **Low** | **Translate is fast (<500ms for short chunks)** |
| **LibreTranslate service unavailable** | **Medium** | **Graceful fallback to original text, log warning** |

---

## Timeline

```
Week 1: Phase 1 (Stability)
  Mon-Tue: Tests + CI
  Wed: Toast system + error handling
  Thu: Docs + README

Week 2: Phase 2 (UX)
  Mon: System tray
  Tue: Global shortcuts
  Wed: Model state + VAD slider
  Thu: Polish + testing

Week 3-4: Phase 3 (Functionality)
  Mon-Wed: Export system
  Thu-Fri: Virtualized history
  Week 3: Overlay drag + history
  Week 4: Translation (Whisper/LibreTranslate approach)
    - Feature spec: .specify/features/005-overlay-translation/
    - Tasks: 39 tasks across 8 phases
    - Estimated: 3-4 days (Whisper translate + accumulative overlay)
```
