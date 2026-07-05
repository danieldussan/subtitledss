# Feature: Phase 3 — Core Functionality

## Goal
Complete the core feature set: export transcriptions, scrollable history, draggable overlay, and local translation.

## User Stories

### US-1: Export Transcriptions
**As a** user
**I want** to export my transcription history
**So that** I can use it in other tools or archive it

**Acceptance Criteria:**
- [ ] Export button in History panel
- [ ] Formats: .srt (SubRip), .vtt (WebVTT), .txt (plain text), .json (raw)
- [ ] SRT format: sequence number, timestamps (HH:MM:SS,mmm --> HH:MM:SS,mmm), text
- [ ] VTT format: "WEBVTT" header, timestamps (HH:MM:SS.mmm --> HH:MM:SS.mmm), text
- [ ] JSON format: array of {id, timestamp, language, text, translation}
- [ ] File save dialog (native OS dialog)
- [ ] Export filtered results if search is active, otherwise all
- [ ] Bulk export: select multiple entries with checkboxes

### US-2: Scrollable History with Pagination
**As a** user
**I want** to browse through all my past transcriptions
**So that** I can find specific conversations

**Acceptance Criteria:**
- [ ] Virtualized list (react-window or similar) for performance
- [ ] Infinite scroll: load 50 entries at a time
- [ ] "Load more" button at bottom if scroll not feasible
- [ ] Entry count shown in History tab badge
- [ ] Entries grouped by date (Today, Yesterday, This Week, Older)
- [ ] Click entry → expand to show full text + timestamp
- [ ] Delete individual entries (swipe or button)

### US-3: Draggable Overlay
**As a** user
**I want** to reposition the subtitle overlay
**So that** it doesn't block important content

**Acceptance Criteria:**
- [ ] Overlay window is draggable by default
- [ ] Drag with left mouse button
- [ ] Position remembered across restarts (saved to config)
- [ ] Double-click → reset to default position
- [ ] Right-click → context menu: Lock Position, Reset, Settings
- [ ] Position shown in Settings → Overlay tab (X, Y inputs)

### US-4: Local Translation (NLLB-200)
**As a** user
**I want** real-time translation of transcribed text
**So that** I can understand foreign language content

**Acceptance Criteria:**
- [ ] NLLB-200 model integration (facebook/nllb-200-distilled-600M)
- [ ] Translation runs after transcription in pipeline
- [ ] Source language auto-detected by Whisper
- [ ] Target language configurable in Settings → Translation
- [ ] Supported: en, es, fr, de, it, pt, ja, zh, ko, ar, ru, hi
- [ ] Overlay shows translated text (original below in smaller font)
- [ ] Translation toggle in Settings (can disable for performance)
- [ ] Model downloaded on first use (like Whisper models)
- [ ] Latency target: <1s additional after transcription

### US-5: History Search Improvements
**As a** user
**I want** better search capabilities
**So that** I can find specific phrases quickly

**Acceptance Criteria:**
- [ ] Real-time search (debounced 300ms, no Enter required)
- [ ] Search highlights matching text in results
- [ ] Filter by language
- [ ] Filter by date range
- [ ] Search results show context (surrounding entries)

## Technical Notes

### SRT Format Example
```
1
00:00:01,000 --> 00:00:04,500
Hello, this is a test transcription.

2
00:00:05,200 --> 00:00:08,800
The second segment of speech.
```

### VTT Format Example
```
WEBVTT

00:00:01.000 --> 00:00:04.500
Hello, this is a test transcription.

00:00:05.200 --> 00:00:08.800
The second segment of speech.
```

### NLLB Integration
- Use `candle` or `ort` crate for ONNX inference
- Model: `facebook/nllb-200-distilled-600M` (~1.2GB)
- Tokenizer: sentencepiece (NLLB-specific)
- Batch translation for better throughput
- Cache translated segments to avoid re-translation

### Virtualized List
```tsx
import { useVirtualizer } from '@tanstack/react-virtual';

// Or simpler:
import { FixedSizeList } from 'react-window';
```

### Overlay Drag (Tauri 2)
```rust
// Overlay window config
"decorations": false,
"transparent": true,
"alwaysOnTop": true,
"resizable": false

// Frontend: draggable div
<div
  data-tauri-drag-region
  className="cursor-move"
>
  {text}
</div>
```

### Date Grouping Logic
```typescript
function getDateGroup(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days === 0) return 'Today';
  if (days === 1) return 'Yesterday';
  if (days < 7) return 'This Week';
  if (days < 30) return 'This Month';
  return 'Older';
}
```

## File Structure
```
src-tauri/src/
├── commands/
│   └── export.rs         (new: SRT/VTT/TXT/JSON export)
├── translation/
│   ├── mod.rs
│   ├── nllb.rs           (NLLB-200 inference)
│   └── tokenizer.rs      (sentencepiece tokenizer)

src/
├── components/
│   ├── History/
│   │   └── HistoryList.tsx  (virtualized, grouped, searchable)
│   ├── Export/
│   │   └── ExportDialog.tsx (format selection, file save)
│   └── Settings/
│       └── TranslationSettings.tsx
└── utils/
    └── export.ts         (SRT/VTT/TXT formatters)
```

## Dependencies (New)
```toml
# Rust
candle-core = "0.8"         # or ort for ONNX
candle-nn = "0.8"
tokenizers = "0.21"         # HuggingFace tokenizers

# Frontend
@tanstack/react-virtual = "^3"  # or react-window
```
