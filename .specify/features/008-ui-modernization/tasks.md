# Tasks: 008-ui-modernization

**Feature**: UI Modernization & UX Improvement
**Branch**: `008-ui-modernization`
**Total Tasks**: 26
**Estimated Effort**: XL

---

## Phase 0: Foundation — Sidebar Layout & App Shell

- [ ] **T01** [P0] [Story?] Create `Section` type and `SECTION_ITEMS` constant in a new `src/components/Layout/types.ts` file. Define `Section = "dashboard" | "transcriptions" | "export" | "audio" | "settings" | "overlay" | "shortcuts"` and the nav items array (label, icon, section id). No dependencies. **Effort: S**

- [ ] **T02** [P0] [Story?] Create `src/components/Layout/Sidebar.tsx` with props `{ activeSection: Section; onNavigate: (section: Section) => void; isCollapsed?: boolean }`. Render a vertical nav with logo ("S" badge), 7 nav items using lucide-react icons, active state highlighting (`bg-accent-subtle text-accent`), and collapse-to-icon mode when `isCollapsed` is true. Use `.sidebar` / `.sidebar-item` CSS classes. Depends on: T01. **Effort: M**

- [ ] **T03** [P0] [Story?] Add `.sidebar` and `.sidebar-item` CSS classes to `src/styles/globals.css`. Sidebar: fixed left, `w-52` when expanded, `w-14` when collapsed, `bg-bg-raised`, `border-r border-border-subtle`. Sidebar items: `flex items-center gap-3 px-3 py-2 rounded-md`, hover/active states. Add transition for collapse animation. Depends on: none. **Effort: S**

- [ ] **T04** [P0] [Story?] Create `src/components/Layout/SectionRouter.tsx` with props `{ section: Section; isCapturing: boolean; overlayVisible: boolean; translationEnabled: boolean; loadedModel: string | null; audioDevice: string | null; onToggleOverlay: () => void; onNavigate: (section: Section) => void }`. Maps `section` to the correct component (dashboard → Dashboard placeholder, transcriptions → HistoryList, export → placeholder, audio → placeholder, settings → SettingsPanel, overlay → placeholder, shortcuts → placeholder). Depends on: none. **Effort: S**

- [ ] **T05** [P0] [Story?] Create `src/components/Layout/AppShell.tsx` with props `{ isCapturing: boolean; overlayVisible: boolean; translationEnabled: boolean; loadedModel: string | null; audioDevice: string | null; onToggleCapture: () => void; onToggleOverlay: () => void; onToggleTranslation: () => void }`. Renders: Sidebar (left) + content area (right) + header bar (top of content) with capture/overlay/translate buttons + status bar (bottom of content). Manages `activeSection` state internally. Depends on: T02, T03. **Effort: M**

- [ ] **T06** [P0] [Story?] Refactor `src/App.tsx` to be a thin wrapper around `<AppShell />`. Strip out tab navigation (`activeTab`, tab bar, inline tab switching). Keep all state (`isCapturing`, `overlayVisible`, `translationEnabled`, `loadedModel`, `audioDevice`) and all toggle callbacks (`toggleCapture`, `toggleOverlay`, `toggleTranslation`). Pass everything as props to `<AppShell />`. Keep keyboard shortcut listener (`Ctrl+Shift+S`). Depends on: T05. **Effort: M**

- [ ] **T07** [P0] [Story?] Verify Phase 0: run `bun run typecheck`, `bun run lint`, `bun run build`. Visual check: sidebar renders all 7 items, clicking each shows correct content area (placeholder text for unimplemented sections), `Ctrl+Shift+S` still toggles capture. All existing settings/history/models accessible. **Effort: S**

---

## Phase 1: Dashboard — Status Cards & Live Feed

- [ ] **T08** [P1] [US1] Create `src/components/Dashboard/StatusHeroCard.tsx` with props `{ icon: LucideIcon; title: string; value: string; status: "active" | "inactive" | "warning"; onClick?: () => void }`. Renders a `.card` with icon (colored by status), title, value, and a `.status-dot`. Card uses `.card p-4` layout. Depends on: none (standalone component). **Effort: S**

- [ ] **T09** [P1] [US1] Create `src/components/Dashboard/QuickActions.tsx` with props `{ onToggleOverlay: () => void; overlayVisible: boolean; onNavigate: (section: Section) => void }`. Renders a 2×2 grid of action buttons: Toggle Overlay (icon toggles MonitorPlay/MonitorOff), View History (→ transcriptions), Export (→ export), Settings (→ settings). Each button is a `.card` with icon + label, onClick navigates or triggers action. Depends on: T01 (Section type). **Effort: S**

- [ ] **T10** [P1] [US1] Create `src/components/Dashboard/LiveTranscriptionPanel.tsx` with props `{ isCapturing: boolean; overlayVisible: boolean; onToggleOverlay: () => void }`. Adaptive panel: when `overlayVisible=false` and `isCapturing=true`, show a scrollable text area with recent transcription segments (mock empty state initially); when `overlayVisible=true`, show "Overlay is active — transcriptions display on screen" with a link to toggle overlay off. Add a `useRef` for auto-scroll container. Depends on: none. **Effort: M**

- [ ] **T11** [P1] [US1] Create `src/components/Dashboard/AudioMeterCard.tsx` with props `{ isCapturing: boolean }`. Wraps existing `EnergyMeter` component in a `.card` container with a "Audio Level" title. Shows "Audio unavailable" error state when not capturing. Depends on: none (reuses existing EnergyMeter). **Effort: S**

- [ ] **T12** [P1] [US1] Create `src/components/Dashboard/ModelStatusCard.tsx` with props `{ loadedModel: string | null }`. Shows loaded model name + tier badge (placeholder) + "No model loaded" when null with a link to open Model Manager. Depends on: none. **Effort: S**

- [ ] **T13** [P1] [US1] Create `src/components/Dashboard/Dashboard.tsx` with props matching the dashboard needs. Layout: 3-column hero cards row (Model, Translation, Audio) → 2-column row (LiveTranscriptionPanel left, QuickActions right) → full-width AudioMeterCard. Depends on: T08, T09, T10, T11, T12. **Effort: M**

- [ ] **T14** [P1] [US1] Update `src/components/Layout/SectionRouter.tsx` to import and render `<Dashboard />` for the `"dashboard"` section (replacing placeholder). Wire all required props. Depends on: T04, T13. **Effort: S**

- [ ] **T15** [P1] [US1] Verify Phase 1: run `bun run typecheck`, `bun run lint`, `bun run build`. Visual check: dashboard shows 3 hero cards with correct values, quick actions grid navigates to sections, live transcription panel shows adaptive content, audio meter renders. Depends on: T07. **Effort: S**

---

## Phase 2: Settings — Flat Sections & Model Picker

- [ ] **T16** [P2] [US2] Create `src/components/Settings/InlineControl.tsx` with props `{ label: string; description?: string; children: React.ReactNode }`. Renders label + optional description on the left, control (slot) on the right, in a flex row with vertical centering. Depends on: none. **Effort: S**

- [ ] **T17** [P2] [US2] Create `src/components/Settings/SettingsSection.tsx` with props `{ title: string; description?: string; defaultCollapsed?: boolean; children: React.ReactNode }`. Renders section title + description + collapsible children. Uses `<details>`/`<summary>` or state-based collapse. Default collapsed when `defaultCollapsed=true`. Depends on: none. **Effort: S**

- [ ] **T18** [P2] [US2] Create `src/components/Settings/ModelPicker.tsx` with props `{ selectedModel: string; onSelect: (model: string) => void }`. Renders a card grid of model options. Each card shows: model name, size, tier badge (Fast/Balanced/Precise with color coding), recommendation text. Selected card has accent border. Depends on: T08 (reuses tier badge pattern). **Effort: M**

- [ ] **T19** [P2] [US2] Create `src/components/Settings/SettingsLayout.tsx` with props `{ config: AppConfig; onSave: (config: AppConfig) => void; isCapturing: boolean; activeFilter?: "general" | "appearance" | "advanced" }`. Renders: sub-nav tabs (General | Appearance | Advanced) at top, scrollable area with `SettingsSection` groups. General: AudioSettings, WhisperSettings (with ModelPicker), TranslationSettings. Appearance: ThemeSettings. Advanced: ShortcutsSettings + collapsible advanced options. Depends on: T16, T17, T18. **Effort: M**

- [ ] **T20** [P2] [US2] Refactor `src/components/Settings/SettingsPanel.tsx` to replace nested tab logic with `<SettingsLayout />`. Remove `activeTab` state and tab bar. Pass `config`, `saveConfig`, `isCapturing` to `SettingsLayout`. Depends on: T19. **Effort: S**

- [ ] **T21** [P2] [US2] Verify Phase 2: run `bun run typecheck`, `bun run lint`, `bun run build`. Visual check: settings shows flat sections with inline controls, model picker has tier badges, collapsible sections default to collapsed, sub-nav filters correct sections. All settings persist on change (check TOML). Depends on: T15. **Effort: S**

---

## Phase 3: Onboarding — First-Launch Wizard

- [ ] **T22** [P3] [US3] Create `src/hooks/useOnboarding.ts` hook. Returns `{ shouldShow: boolean; currentStep: number; selections: OnboardingSelections; setStep: (n: number) => void; updateSelection: (partial) => void; complete: () => Promise<void>; skip: () => Promise<void> }`. On mount: reads `config.onboarding_completed` via `invoke("get_config")`. If `false` or missing, `shouldShow=true`. `complete()` merges selections into config and saves `onboarding_completed: true`. `skip()` saves defaults + `onboarding_completed: true`. Depends on: none (uses existing `useSettings` pattern). **Effort: M**

- [ ] **T23** [P3] [US3] Create `src/components/Onboarding/ProgressDots.tsx` with props `{ currentStep: number; totalSteps: number }`. Renders a row of dots: completed = filled accent, current = ring accent, upcoming = muted. Shows "Step N of M" text. Depends on: none. **Effort: S**

- [ ] **T24** [P3] [US3] Create `src/components/Onboarding/OnboardingWizard.tsx` with props `{ onComplete: (config: Partial<AppConfig>) => void; onSkip: () => void }`. Full-screen overlay (`fixed inset-0 z-50 bg-bg-base/90 backdrop-blur-sm`). Renders ProgressDots + current step component + Back/Next/Skip buttons. Manages `currentStep` and `selections` via `useOnboarding`. Validates before allowing "Next". Depends on: T22, T23. **Effort: L**

- [ ] **T25** [P3] [US3] Create 4 step components in `src/components/Onboarding/steps/`:
  - `StepModelSelection.tsx` — model cards with recommendation badges (reuses ModelPicker pattern from T18)
  - `StepLanguageSelection.tsx` — grid of language options with flag emojis + names
  - `StepTranslationSetup.tsx` — toggle enable/disable + source→target language selectors
  - `StepReadySummary.tsx` — summary of selections + keyboard shortcut reference + "Get Started" button
  Each has props matching the plan APIs. Depends on: T18 (ModelPicker pattern). **Effort: M**

- [ ] **T26** [P3] [US3] Update `src/App.tsx` to add onboarding check: import `useOnboarding`, render `<OnboardingWizard />` when `shouldShow=true`, otherwise render `<AppShell />`. Depends on: T06, T24. **Effort: S**

- [ ] **T27** [P3] [US3] Verify Phase 3: run `bun run typecheck`, `bun run lint`, `bun run build`. Visual check: wizard appears when `onboarding_completed=false`, does not appear when `true`. Each step validates before advancing. Skip applies defaults and dismisses. Progress dots update correctly. Depends on: T21. **Effort: S**

---

## Phase 4: Polish — Micro-Interactions & Animations

- [ ] **T28** [P4] [US4] Create `src/components/ui/GlowCard.tsx` with props `{ children: React.ReactNode; className?: string; glowColor?: string }`. Uses `useRef` + `onMouseMove` to track cursor position, sets CSS custom properties (`--glow-x`, `--glow-y`). Renders radial gradient overlay at cursor position via CSS. Performance: uses CSS `transform` and `will-change: transform`. Add `.glow-card` CSS class to globals.css with `@keyframes` and `::before` pseudo-element for the gradient. Depends on: T03 (globals.css edits). **Effort: M**

- [ ] **T29** [P4] [US4] Create `src/components/ui/AnimatedToggle.tsx` with props `{ checked: boolean; onChange: (checked: boolean) => void; label?: string; disabled?: boolean }`. Replaces existing `.toggle-switch` CSS approach. Uses framer-motion `motion.div` with spring animation (`stiffness: 500, damping: 30, mass: 0.8`) for the knob translateX. Background color transitions via CSS. Depends on: none. **Effort: M**

- [ ] **T30** [P4] [US4] Create `src/components/ui/RippleButton.tsx` extending `ButtonHTMLAttributes<HTMLButtonElement>` with props `{ children: React.ReactNode; variant?: "primary" | "ghost" | "danger"; size?: "sm" | "md" }`. On click: spawns a circle at click coordinates (`e.nativeEvent.offsetX/Y`), animates scale 0→2.5 + opacity 0.4→0 over 600ms using framer-motion `AnimatePresence`. Depends on: none. **Effort: M**

- [ ] **T31** [P4] [US4] Create `src/components/ui/Skeleton.tsx` with props `{ width?: string | number; height?: string | number; variant?: "text" | "rect" | "circle"; className?: string }`. Uses existing `.skeleton` CSS class (shimmer animation already in globals.css). Variants: text = `h-4 w-full rounded`, rect = `rounded-lg`, circle = `rounded-full`. Depends on: none (CSS already exists). **Effort: S**

- [ ] **T32** [P4] [US4] Create `src/components/ui/KbdBadge.tsx` with props `{ keys: string[] }`. Renders `<kbd>` elements with `font-mono text-xs bg-bg-surface border border-border-default rounded px-1.5 py-0.5` styling. Joins keys with "+" separator. Depends on: none. **Effort: S**

- [ ] **T33** [P4] [US4] Create `src/components/ui/ToastTimer.tsx` with props `{ toast: Toast; onRemove: (id: string) => void }`. Entry animation: slide from right + fade in (300ms). Timer bar: width animates from 100%→0% over the dismiss duration using framer-motion `useAnimate`. Exit: slide out right + fade out (200ms). Depends on: none. **Effort: M**

- [ ] **T34** [P4] [US4] Update `src/components/ui/Toast.tsx` to replace static toast rendering with `<ToastTimer />`. Wrap each toast in `<AnimatePresence>`. Depends on: T33. **Effort: S**

- [ ] **T35** [P4] [US4] Replace existing `.toggle-switch` CSS usage in settings components with `<AnimatedToggle />`. Update `AudioSettings.tsx`, `WhisperSettings.tsx`, `TranslationSettings.tsx`, `ThemeSettings.tsx` to import and use `AnimatedToggle`. Depends on: T29, T20. **Effort: S**

- [ ] **T36** [P4] [US4] Add `.glow-card` and `.ripple` CSS classes to `src/styles/globals.css`. Add shimmer keyframe refinements if needed. Add `kbd` styling utility class. Depends on: T03. **Effort: S**

- [ ] **T37** [P4] [US4] Verify Phase 4: run `bun run typecheck`, `bun run lint`, `bun run build`. Visual check: hover glow follows cursor on cards, toggle spring animates smoothly, ripple appears at click point, toast slides in with timer bar, skeleton shimmer runs at 60fps, focus ring visible, kbd badges render correctly. Depends on: T27. **Effort: S**

---

## Phase 5: Integration — Wire Everything & Regression

- [ ] **T38** [P5] [All] Final integration of `src/App.tsx`: ensure `AppShell` + `OnboardingWizard` work together, all state flows correctly, header buttons (overlay/translate/capture) are in the right place. Clean up any unused imports from old tab-based layout. Depends on: T26, T34. **Effort: S**

- [ ] **T39** [P5] [All] Run full regression test matrix (from plan):
  - Start/Stop capture via sidebar Audio section and Ctrl+Shift+S
  - Toggle overlay via header button
  - Toggle translation via header button
  - Settings: all 5 subsections accessible via flat layout
  - History list accessible via sidebar Transcriptions
  - Model manager accessible via Settings Model Picker
  - All keyboard shortcuts functional
  - Check TOML config file after settings changes for persistence
  - Verify dark theme contrast (WCAG AA)
  - Depends on: T38. **Effort: M**

- [ ] **T40** [P5] [All] Performance verification: open Chrome DevTools Performance panel, run through all micro-interactions (glow, toggle, ripple, toast, skeleton), verify 60fps. If any drop below, add graceful degradation (reduce to opacity transitions). Depends on: T39. **Effort: S**

- [ ] **T41** [P5] [All] Cleanup: remove deprecated `.tab-bar` usage from main nav (keep for sub-nav if still used in SettingsLayout). Remove any dead code from old tab system. Final `bun run fmt` + `bun run lint:fix`. Depends on: T39. **Effort: S**

---

## Summary

| Phase | Tasks | Effort | Parallelizable |
|-------|-------|--------|----------------|
| Phase 0: Foundation | T01–T07 | M | No (sequential) |
| Phase 1: Dashboard | T08–T15 | L | After Phase 0 |
| Phase 2: Settings | T16–T21 | L | After Phase 0 (parallel with Phase 1) |
| Phase 3: Onboarding | T22–T27 | M | After Phase 0 (parallel with Phase 1 & 2) |
| Phase 4: Polish | T28–T37 | M | After Phase 0 (parallel with 1–3) |
| Phase 5: Integration | T38–T41 | M | After all phases |
| **Total** | **41 tasks** | **XL** | |

## Dependency Graph

```
Phase 0 (T01-T07)
    │
    ├──► Phase 1 (T08-T15)  ──┐
    ├──► Phase 2 (T16-T21)  ──┼──► Phase 5 (T38-T41)
    ├──► Phase 3 (T22-T27)  ──┤
    └──► Phase 4 (T28-T37)  ──┘
```

## Quick Reference: Task IDs by Component

| Component | Task | Phase |
|-----------|------|-------|
| Sidebar | T02 | 0 |
| AppShell | T05 | 0 |
| SectionRouter | T04 | 0 |
| StatusHeroCard | T08 | 1 |
| QuickActions | T09 | 1 |
| LiveTranscriptionPanel | T10 | 1 |
| AudioMeterCard | T11 | 1 |
| ModelStatusCard | T12 | 1 |
| Dashboard | T13 | 1 |
| InlineControl | T16 | 2 |
| SettingsSection | T17 | 2 |
| ModelPicker | T18 | 2 |
| SettingsLayout | T19 | 2 |
| SettingsPanel refactor | T20 | 2 |
| useOnboarding | T22 | 3 |
| ProgressDots | T23 | 3 |
| OnboardingWizard | T24 | 3 |
| Step components (×4) | T25 | 3 |
| App.tsx onboarding | T26 | 3 |
| GlowCard | T28 | 4 |
| AnimatedToggle | T29 | 4 |
| RippleButton | T30 | 4 |
| Skeleton | T31 | 4 |
| KbdBadge | T32 | 4 |
| ToastTimer | T33 | 4 |
| Toast refactor | T34 | 4 |
| Toggle replacement | T35 | 4 |
| CSS additions | T36 | 4 |
