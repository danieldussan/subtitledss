# Implementation Plan: 008-ui-modernization

**Feature**: UI Modernization & UX Improvement
**Branch**: `008-ui-modernization`
**Created**: 2026-07-13
**Status**: Ready

---

## Technical Context

| Item | Value |
|------|-------|
| Framework | Tauri 2 + React 19 + TypeScript |
| Styling | Tailwind CSS v4 (utility-first, design tokens in globals.css) |
| Animation | framer-motion (already installed) |
| Icons | lucide-react (already installed) |
| State | React hooks + `invoke()` IPC to Rust backend |
| Build | Vite 7, oxlint, oxfmt |
| Window | Fixed size (Tauri desktop, no responsive breakpoints) |
| Theme | Dark-only, slate-based neutrals, blue-500 accent |

---

## Constitution Compliance

| Principle | Status | Notes |
|-----------|--------|-------|
| Offline-First | ✅ PASS | No new network calls; all UI-only changes |
| Real-Time Performance | ✅ PASS | 60fps animations via framer-motion; EnergyMeter already 60fps |
| Modular Architecture | ✅ PASS | Each new component is self-contained; hooks encapsulate state |
| Linux-Native | ✅ PASS | No platform-specific changes; sidebar layout is desktop-optimized |
| Test-First | ⚠️ CONDITIONAL | Plan includes component-level verification; no new Rust modules |

---

## Phase 0: Foundation — Sidebar Layout & App Shell

**Goal**: Replace top tab navigation with persistent sidebar. Refactor `App.tsx` from tab-based to section-based routing.

### Files to Create

| File | Purpose |
|------|---------|
| `src/components/Layout/Sidebar.tsx` | Persistent left nav with icons + labels, active state, collapse logic |
| `src/components/Layout/AppShell.tsx` | New root layout: sidebar + header + content area + status bar |
| `src/components/Layout/SectionRouter.tsx` | Maps active section → component, replaces inline tab logic |

### Files to Modify

| File | Changes |
|------|---------|
| `src/App.tsx` | Strip down to thin wrapper around `<AppShell />`. Move all state to a new `useAppState` hook or keep inline but pass via props. |
| `src/styles/globals.css` | Add `.sidebar` and `.sidebar-item` classes; deprecate `.tab-bar` if no longer used (keep for sub-nav) |

### Component APIs

```tsx
// Sidebar.tsx
interface SidebarProps {
  activeSection: Section;
  onNavigate: (section: Section) => void;
  isCollapsed?: boolean;
}

type Section = "dashboard" | "transcriptions" | "export" | "audio" | "settings" | "overlay" | "shortcuts";

// AppShell.tsx
interface AppShellProps {
  isCapturing: boolean;
  overlayVisible: boolean;
  translationEnabled: boolean;
  loadedModel: string | null;
  audioDevice: string | null;
  onToggleCapture: () => void;
  onToggleOverlay: () => void;
  onToggleTranslation: () => void;
}

// SectionRouter.tsx
interface SectionRouterProps {
  section: Section;
  isCapturing: boolean;
  overlayVisible: boolean;
  translationEnabled: boolean;
  loadedModel: string | null;
  audioDevice: string | null;
  onToggleOverlay: () => void;
}
```

### State Management

- `activeSection` state lives in `App.tsx` (lifted from old `activeTab`)
- All toggle functions remain in `App.tsx` via existing `useCallback` patterns
- No new context providers needed — props drilling is fine for this tree depth

### Sidebar Navigation Items

```
┌─────────────────────┐
│ S (logo)            │
├─────────────────────┤
│ ◉ Dashboard         │
│ 📋 Transcriptions   │
│ 📤 Export           │
│ 🎤 Audio            │
│ ⚙️ Settings         │
│ 🖥️ Overlay          │
│ ⌨️ Shortcuts        │
└─────────────────────┘
```

### Keyboard Shortcuts

- `Ctrl+Shift+S` — toggle capture (existing, preserved)
- `Ctrl+Shift+O` — toggle overlay (existing, preserved)
- `Ctrl+Shift+T` — toggle translation (existing, preserved)
- `Ctrl+1..7` — navigate to section by index (new, optional)

### Testing Strategy

- Visual: sidebar renders all 7 items, active item highlights
- Navigation: clicking each item shows correct content area
- Keyboard: `Ctrl+Shift+S` still works after refactor
- Regression: all existing settings, history, models still accessible

---

## Phase 1: Dashboard — Status Cards & Live Feed

**Goal**: Build the dashboard view with hero status cards, quick actions, live transcription panel, and audio meter.

### Files to Create

| File | Purpose |
|------|---------|
| `src/components/Dashboard/Dashboard.tsx` | Main dashboard layout container |
| `src/components/Dashboard/StatusHeroCard.tsx` | Reusable card: icon + title + value + status dot |
| `src/components/Dashboard/QuickActions.tsx` | Grid of action buttons (toggle overlay, history, export, settings) |
| `src/components/Dashboard/LiveTranscriptionPanel.tsx` | Adaptive panel: live feed (overlay OFF) or status message (overlay ON) |
| `src/components/Dashboard/AudioMeterCard.tsx` | Wraps EnergyMeter in a dashboard card with label |
| `src/components/Dashboard/ModelStatusCard.tsx` | Shows loaded model name, tier badge, memory indicator |

### Component APIs

```tsx
// Dashboard.tsx
interface DashboardProps {
  isCapturing: boolean;
  overlayVisible: boolean;
  translationEnabled: boolean;
  loadedModel: string | null;
  audioDevice: string | null;
  onToggleOverlay: () => void;
  onNavigate: (section: Section) => void;
}

// StatusHeroCard.tsx
interface StatusHeroCardProps {
  icon: LucideIcon;
  title: string;
  value: string;
  status: "active" | "inactive" | "warning";
  onClick?: () => void;
}

// QuickActions.tsx
interface QuickActionsProps {
  onToggleOverlay: () => void;
  overlayVisible: boolean;
  onNavigate: (section: Section) => void;
}

// LiveTranscriptionPanel.tsx
interface LiveTranscriptionProps {
  isCapturing: boolean;
  overlayVisible: boolean;
  onToggleOverlay: () => void;
}

// AudioMeterCard.tsx
interface AudioMeterCardProps {
  isCapturing: boolean;
}

// ModelStatusCard.tsx
interface ModelStatusCardProps {
  loadedModel: string | null;
}
```

### Dashboard Layout

```
┌──────────────────────────────────────────────────────┐
│  Dashboard                                            │
├──────────────────────────────────────────────────────┤
│ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│ │ 🧠 Model │ │ 🌐 Trans │ │ 🎤 Audio │  (hero cards) │
│ └──────────┘ └──────────┘ └──────────┘              │
│                                                       │
│ ┌─────────────────────────┐ ┌────────────────────┐   │
│ │ Live Transcription      │ │ Quick Actions      │   │
│ │ ─────────────────────── │ │ ┌────┐ ┌────┐     │   │
│ │ [real-time text feed]   │ │ │OVR │ │HIST│     │   │
│ │ [auto-scroll]           │ │ └────┘ └────┘     │   │
│ │                         │ │ ┌────┐ ┌────┐     │   │
│ │                         │ │ │EXP │ │ SET│     │   │
│ │                         │ │ └────┘ └────┘     │   │
│ └─────────────────────────┘ └────────────────────┘   │
│                                                       │
│ ┌──────────────────────────────────────────────────┐ │
│ │ Audio Level Meter                                │ │
│ │ [████████░░░░░░░░░░] 45.2%                       │ │
│ └──────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

### State Management

- Dashboard reads existing props from `AppShell` (no new state)
- `LiveTranscriptionPanel` needs a new `useLiveTranscription` hook or extends `useTranscription` with a `recentSegments` buffer
- `AudioMeterCard` reuses existing `EnergyMeter` component directly

### Testing Strategy

- Hero cards display correct values based on props
- Quick actions navigate to correct sections
- Live panel shows feed when `overlayVisible=false`, status when `true`
- Audio meter renders and updates at 60fps
- Model card shows "No model loaded" when `loadedModel=null`

---

## Phase 2: Settings — Flat Sections & Model Picker

**Goal**: Replace nested tab settings with flat scrollable sections. Add model picker cards with tier badges.

### Files to Create

| File | Purpose |
|------|---------|
| `src/components/Settings/SettingsLayout.tsx` | New settings shell: sub-nav + scrollable sections |
| `src/components/Settings/SettingsSection.tsx` | Reusable section wrapper: title + description + collapsible |
| `src/components/Settings/ModelPicker.tsx` | Card grid for model selection with tier badges |
| `src/components/Settings/InlineControl.tsx` | Wrapper for inline label + control (toggle, slider, dropdown) |

### Files to Modify

| File | Changes |
|------|---------|
| `src/components/Settings/SettingsPanel.tsx` | Gut and rewrite to use `SettingsLayout` + flat sections instead of tab switching |
| `src/components/Settings/AudioSettings.tsx` | Extract into flat sections, use `InlineControl` wrappers |
| `src/components/Settings/WhisperSettings.tsx` | Replace model `<select>` with `ModelPicker` cards |
| `src/components/Settings/TranslationSettings.tsx` | Flatten, use inline controls |
| `src/components/Settings/ThemeSettings.tsx` | Flatten, use inline controls |
| `src/components/Settings/ShortcutsSettings.tsx` | Flatten, use inline controls |

### Component APIs

```tsx
// SettingsLayout.tsx
interface SettingsLayoutProps {
  config: AppConfig;
  onSave: (config: AppConfig) => void;
  isCapturing: boolean;
  activeFilter?: "general" | "appearance" | "advanced";
}

// SettingsSection.tsx
interface SettingsSectionProps {
  title: string;
  description?: string;
  defaultCollapsed?: boolean;
  children: React.ReactNode;
}

// ModelPicker.tsx
interface ModelPickerProps {
  selectedModel: string;
  onSelect: (model: string) => void;
}

interface ModelOption {
  id: string;
  name: string;
  size: string;
  tier: "fast" | "balanced" | "precise";
  description: string;
}

// InlineControl.tsx
interface InlineControlProps {
  label: string;
  description?: string;
  children: React.ReactNode; // toggle, slider, select, etc.
}
```

### Settings Sub-Nav Filters

```
[General] [Appearance] [Advanced]
```

- **General**: Audio device, whisper model, language, threads
- **Appearance**: Overlay font, colors, opacity, position
- **Advanced**: GPU acceleration, VAD threshold, auto-hide delay, export format

### Model Tier Badges

| Tier | Color | Label |
|------|-------|-------|
| fast | `bg-success-subtle text-success` | ⚡ Fast |
| balanced | `bg-accent-subtle text-accent` | ⚖️ Balanced |
| precise | `bg-warning-subtle text-warning` | 🎯 Precise |

### State Management

- Settings continue using `useSettings` hook with `config` + `saveConfig`
- No new state management needed — all controls call `saveConfig` inline on change

### Testing Strategy

- All 5 settings subsections render in flat layout
- Model picker shows tier badges and selection works
- Inline controls persist changes without save button
- Collapsible sections default to collapsed
- Sub-nav filters correct sections

---

## Phase 3: Onboarding — First-Launch Wizard

**Goal**: 4-step wizard for first-time users. Detects first launch, stores completion flag in TOML config.

### Files to Create

| File | Purpose |
|------|---------|
| `src/components/Onboarding/OnboardingWizard.tsx` | Wizard overlay container with step routing |
| `src/components/Onboarding/StepModelSelection.tsx` | Step 1: Model cards with recommendation badges |
| `src/components/Onboarding/StepLanguageSelection.tsx` | Step 2: Language grid with flags |
| `src/components/Onboarding/StepTranslationSetup.tsx` | Step 3: Translation toggle + direction selector |
| `src/components/Onboarding/StepReadySummary.tsx` | Step 4: Config summary + shortcuts reference + "Get Started" |
| `src/components/Onboarding/ProgressDots.tsx` | Step indicator dots (current + completed) |
| `src/hooks/useOnboarding.ts` | First-launch detection, step state, skip/complete logic |

### Files to Modify

| File | Changes |
|------|---------|
| `src/App.tsx` | Add `onboarding_completed` check; render `OnboardingWizard` before `AppShell` when not completed |
| `src-tauri/src/settings/config.rs` | Add `onboarding_completed: bool` field to `AppConfig` struct (default: `false`) |

### Component APIs

```tsx
// OnboardingWizard.tsx
interface OnboardingWizardProps {
  onComplete: (config: Partial<AppConfig>) => void;
  onSkip: () => void;
}

// ProgressDots.tsx
interface ProgressDotsProps {
  currentStep: number;  // 0-indexed
  totalSteps: number;
}

// StepModelSelection.tsx
interface StepModelSelectionProps {
  selectedModel: string;
  onSelect: (model: string) => void;
}

// StepLanguageSelection.tsx
interface StepLanguageSelectionProps {
  selectedLanguage: string;
  onSelect: (language: string) => void;
}

// StepTranslationSetup.tsx
interface StepTranslationSetupProps {
  enabled: boolean;
  sourceLang: string;
  targetLang: string;
  onUpdate: (update: { enabled?: boolean; sourceLang?: string; targetLang?: string }) => void;
}

// StepReadySummary.tsx
interface StepReadySummaryProps {
  model: string;
  language: string;
  translation: { enabled: boolean; source: string; target: string };
  onStart: () => void;
}
```

### Wizard Flow

```
┌─────────────────────────────────────┐
│  Welcome to subtitledss             │
│                                     │
│  ● ○ ○ ○    (1 of 4)              │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ Choose your AI model        │   │
│  │                             │   │
│  │ ┌────────┐ ┌────────┐      │   │
│  │ │ Tiny   │ │ Base   │      │   │
│  │ │ ⚡ Fast │ │ ⚖️ Rec  │      │   │
│  │ └────────┘ └────────┘      │   │
│  │ ┌────────┐                  │   │
│  │ │ Small  │                  │   │
│  │ │ 🎯 Acc │                  │   │
│  │ └────────┘                  │   │
│  └─────────────────────────────┘   │
│                                     │
│  [Skip]              [Next →]       │
└─────────────────────────────────────┘
```

### First-Launch Detection

1. On app mount, `useOnboarding` checks `config.onboarding_completed`
2. If `false` (or field missing from old configs), show wizard
3. Wizard completion sets `onboarding_completed = true` via `save_config`
4. Skip also sets `onboarding_completed = true` with default values

### State Management

- `useOnboarding` hook manages: `currentStep`, `selections` (model, language, translation)
- Wizard is a controlled component — parent (`App.tsx`) owns completion state
- On complete/skip, calls `saveConfig` with merged selections + `onboarding_completed: true`

### Testing Strategy

- Wizard appears when `onboarding_completed=false`
- Wizard does not appear when `onboarding_completed=true`
- Each step validates before allowing advancement
- Skip applies defaults and dismisses
- Progress dots update correctly
- Completed wizard persists flag to config

---

## Phase 4: Polish — Micro-Interactions & Animations

**Goal**: Add hover glow, animated toggles, button ripple, toast animations, skeleton states, focus rings, keyboard badges.

### Files to Create

| File | Purpose |
|------|---------|
| `src/components/ui/GlowCard.tsx` | Card with radial gradient glow following cursor |
| `src/components/ui/AnimatedToggle.tsx` | Toggle switch with spring physics animation |
| `src/components/ui/RippleButton.tsx` | Button with click ripple effect |
| `src/components/ui/Skeleton.tsx` | Shimmer loading placeholder |
| `src/components/ui/KbdBadge.tsx` | `<kbd>` styled shortcut badge |
| `src/components/ui/ToastTimer.tsx` | Toast with animated timer bar |

### Files to Modify

| File | Changes |
|------|---------|
| `src/components/ui/Toast.tsx` | Replace static toast with animated `ToastTimer` version |
| `src/styles/globals.css` | Add shimmer keyframe, glow utility, ripple styles |
| All card-containing components | Wrap cards with `GlowCard` where hover effect is desired |

### Component APIs

```tsx
// GlowCard.tsx
interface GlowCardProps {
  children: React.ReactNode;
  className?: string;
  glowColor?: string; // default: accent
}

// AnimatedToggle.tsx
interface AnimatedToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
}

// RippleButton.tsx
interface RippleButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode;
  variant?: "primary" | "ghost" | "danger";
  size?: "sm" | "md";
}

// Skeleton.tsx
interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  variant?: "text" | "rect" | "circle";
  className?: string;
}

// KbdBadge.tsx
interface KbdBadgeProps {
  keys: string[]; // e.g. ["Ctrl", "Shift", "S"]
}

// ToastTimer.tsx
interface ToastTimerProps {
  toast: Toast;
  onRemove: (id: string) => void;
}
```

### Animation Specifications

#### Hover Glow (`GlowCard`)
```
- On mousemove: calculate cursor position relative to card center
- Render radial gradient at cursor position with 50% opacity
- Gradient: accent color → transparent (150px radius)
- Performance: use CSS custom properties + transform, not re-renders
```

#### Spring Toggle (`AnimatedToggle`)
```
- Knob animation: framer-motion spring
  - stiffness: 500, damping: 30, mass: 0.8
  - translateX from 0 → 18px on activate
- Background: color transition 0.2s (border-default → accent)
```

#### Button Ripple (`RippleButton`)
```
- On click: spawn circle at click coordinates
- Animate: scale 0 → 2.5, opacity 0.4 → 0 over 600ms
- Use framer-motion AnimatePresence for cleanup
```

#### Toast Slide (`ToastTimer`)
```
- Entry: slide from right + fade in (300ms)
- Timer bar: width animates from 100% → 0% over duration
- Exit: slide out right + fade out (200ms)
```

#### Skeleton Shimmer (`Skeleton`)
```
- Background: linear-gradient 90deg
  - bg-surface → bg-hover → bg-surface (2s infinite)
- CSS animation, no JS needed
```

#### Focus Ring
```
- Already implemented in globals.css: `:focus-visible { outline: 2px solid accent; outline-offset: 2px }`
- Verify consistency across all interactive elements
```

### State Management

- No new state — all effects are purely visual
- `GlowCard` uses `useRef` + `onMouseMove` + CSS custom properties
- `RippleButton` uses `useState` for ripple array + `AnimatePresence`

### Performance Budget

| Effect | Target | Fallback |
|--------|--------|----------|
| Hover glow | 60fps via CSS vars | Disable on <30fps |
| Spring toggle | 60fps via framer-motion | CSS transition fallback |
| Button ripple | 60fps via framer-motion | Simple opacity change |
| Toast animation | 60fps via framer-motion | Instant appear/disappear |
| Skeleton shimmer | Pure CSS animation | Static placeholder |

### Testing Strategy

- Hover glow follows cursor without lag
- Toggle spring animates smoothly, settles without jitter
- Ripple appears at click point, fades cleanly
- Toast slides in, timer bar depletes, slides out
- Skeleton shimmer runs at 60fps (DevTools verified)
- Focus ring visible on all interactive elements
- Keyboard badges render correctly with monospace font

---

## Phase 5: Integration — Wire Everything Together

**Goal**: Final integration pass. Connect all phases, verify all flows, run regression checks.

### Files to Modify

| File | Changes |
|------|---------|
| `src/App.tsx` | Final integration: `AppShell` + `OnboardingWizard` + all state |
| `src/styles/globals.css` | Final cleanup: remove deprecated `.tab-bar` if unused, add any missing utilities |

### Integration Checklist

- [ ] Sidebar navigation works for all 7 sections
- [ ] Dashboard renders all cards with live data
- [ ] Settings flat sections render with inline controls
- [ ] Model picker appears in Whisper settings and onboarding
- [ ] Onboarding wizard appears on first launch only
- [ ] All micro-interactions work across all components
- [ ] Keyboard shortcuts function identically to pre-refactor
- [ ] Toast notifications animate correctly
- [ ] Skeleton states appear during loading
- [ ] All existing functionality preserved (no regressions)
- [ ] 60fps maintained across all animations
- [ ] Dark theme contrast ratios maintained (WCAG AA)

### Regression Test Matrix

| Feature | Pre-Refactor | Post-Refactor | Status |
|---------|-------------|---------------|--------|
| Start/Stop capture | Tab → Settings | Sidebar → Audio | ⬜ |
| Toggle overlay | Header button | Header button (unchanged) | ⬜ |
| Toggle translation | Header button | Header button (unchanged) | ⬜ |
| Settings: Audio | Settings → Audio tab | Settings → General section | ⬜ |
| Settings: Whisper | Settings → Whisper tab | Settings → General section | ⬜ |
| Settings: Translation | Settings → Translation tab | Settings → General section | ⬜ |
| Settings: Theme | Settings → Overlay tab | Settings → Appearance section | ⬜ |
| Settings: Shortcuts | Settings → Shortcuts tab | Settings → Advanced section | ⬜ |
| History list | History tab | Transcriptions section | ⬜ |
| Model manager | Models tab | Settings → Model Picker | ⬜ |
| Ctrl+Shift+S | Works | Works | ⬜ |
| Ctrl+Shift+O | Works | Works | ⬜ |
| Ctrl+Shift+T | Works | Works | ⬜ |

---

## Dependency Graph

```
Phase 0 (Foundation)
    │
    ├──► Phase 1 (Dashboard)
    │       │
    │       ├──► Phase 4 (Polish) — GlowCard, Skeletons
    │       │
    │       └──► Phase 5 (Integration)
    │
    ├──► Phase 2 (Settings)
    │       │
    │       └──► Phase 5 (Integration)
    │
    └──► Phase 3 (Onboarding)
            │
            └──► Phase 5 (Integration)
```

**Parallel execution possible**: Phase 1, 2, 3 can be developed in parallel after Phase 0. Phase 4 can be developed in parallel with 1-3 (effects are independent). Phase 5 is sequential after all others.

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Sidebar breaks fixed window layout | Medium | High | Test at 1024×768 early; use flex, not grid |
| Settings refactor breaks config persistence | Low | High | Test save/load cycle after each subsection |
| framer-motion performance on low-end GPU | Medium | Medium | Implement graceful degradation toggle |
| Onboarding config field missing from old installs | High | Low | Default to `false`; handle missing field in Rust |
| Tab-bar deprecation breaks sub-nav | Low | Low | Keep `.tab-bar` class; only deprecate usage in main nav |

---

## Success Criteria (from spec)

| Criterion | Verification |
|-----------|-------------|
| SC-001: ≤2 clicks to any setting | Manual test: dashboard → settings section |
| SC-002: Onboarding <90s | Timed test with realistic selections |
| SC-003: 60fps animations | Chrome DevTools Performance panel |
| SC-004: No regressions | Regression test matrix above |
| SC-005: Immediate persistence | Check TOML after each change |
| SC-006: Dashboard <200ms | Performance measurement |
| SC-007: WCAG AA contrast | Automated contrast checker |
| SC-008: Shortcuts preserved | Test each shortcut |

---

## Summary

| Phase | Files Created | Files Modified | Effort |
|-------|--------------|----------------|--------|
| Phase 0: Foundation | 3 | 2 | M |
| Phase 1: Dashboard | 6 | 0 | L |
| Phase 2: Settings | 4 | 6 | L |
| Phase 3: Onboarding | 7 | 2 | M |
| Phase 4: Polish | 6 | 3 | M |
| Phase 5: Integration | 0 | 2 | S |
| **Total** | **26** | **15** | **XL** |
