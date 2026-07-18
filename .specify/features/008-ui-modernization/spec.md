# Feature Specification: UI Modernization & UX Improvement

**Feature Branch**: `008-ui-modernization`

**Created**: 2026-07-13

**Status**: Draft

**Input**: User description: "Modernize the UI/UX with dashboard overview, settings reorganization, onboarding wizard, and micro-interactions polish."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Dashboard Overview (Priority: P1)

**As a** user, **I want** a central dashboard showing system status and quick actions, **so that** I can immediately understand app state and access common features without navigating tabs.

**Why this priority**: The dashboard is the primary navigation paradigm shift. All other features depend on the new layout structure. This delivers the highest UX improvement by replacing flat tabs with an information-rich overview.

**Independent Test**: Can be fully tested by launching the app and verifying the sidebar renders with Dashboard, Transcriptions, Export, Audio, Settings, Overlay, and Shortcuts nav items. Dashboard shows status hero cards and quick action grid. Delivers a complete navigation overhaul.

**Acceptance Scenarios**:

1. **Given** the app is open, **When** the user views the dashboard, **Then** status hero cards display Whisper model (name + status), Translation state (enabled/disabled + direction), and Audio source (device name).
2. **Given** the dashboard is visible, **When** the user views quick actions, **Then** a grid shows Toggle Overlay, View History, Export, and Settings buttons that navigate to the corresponding section.
3. **Given** the overlay is OFF, **When** the user views the Live Transcription panel, **Then** it shows a real-time feed of transcribed text with auto-scroll.
4. **Given** the overlay is ON, **When** the user views the Live Transcription panel, **Then** it shows a status message "Overlay is active — transcriptions display on screen" with a link to toggle overlay off.
5. **Given** audio capture is active, **When** the user views the dashboard, **Then** an audio level meter shows real-time RMS energy with color coding (green/yellow/red).
6. **Given** models are loaded, **When** the user views the Models Status panel, **Then** it shows loaded model name, size, and memory usage.

---

### User Story 2 - Settings Reorganization (Priority: P2)

**As a** user, **I want** settings organized in flat, scannable sections with inline controls, **so that** I can configure the app without digging through nested tabs.

**Why this priority**: Settings are the second most-used area after the dashboard. Flat sections reduce cognitive load and eliminate the "find the right tab" problem. Directly improves daily usage experience.

**Independent Test**: Can be tested by opening Settings and verifying flat sections render with inline controls (sliders, toggles, dropdowns). Changes persist without explicit save buttons. Delivers immediate UX improvement for configuration.

**Acceptance Scenarios**:

1. **Given** the user opens Settings, **When** the settings panel loads, **Then** it shows context-aware sub-nav (General, Appearance, Advanced) with all sections visible in a scrollable view.
2. **Given** the user adjusts a slider, **When** the slider value changes, **Then** the setting is saved immediately without requiring a save button.
3. **Given** the user views model selection, **When** models are listed, **Then** each model shows a visual tier badge (Fast/Balanced/Precise) with size and recommended use case.
4. **Given** the user scrolls through Settings, **When** advanced options are visible, **Then** they are grouped under collapsible sections that default to collapsed.
5. **Given** the user changes any setting, **When** the change is made, **Then** the UI provides immediate visual feedback (toggle animation, slider thumb movement).

---

### User Story 3 - Quick Start Onboarding (Priority: P3)

**As a** first-time user, **I want** a guided setup wizard, **so that** I can configure the app quickly without reading documentation.

**Why this priority**: Onboarding reduces time-to-value for new users. Important for adoption but existing users have already configured their settings. Can be skipped by power users.

**Independent Test**: Can be tested by clearing app config and launching — wizard appears with 4 steps. Each step has clear options and a skip button. Delivers a complete first-run experience.

**Acceptance Scenarios**:

1. **Given** the app is launched for the first time, **When** the main window opens, **Then** the onboarding wizard overlays the dashboard with step 1 (Choose AI Model).
2. **Given** the user is on step 1, **When** they view model options, **Then** each model shows name, size on disk, and a recommendation badge (e.g., "Recommended for most users").
3. **Given** the user completes step 1, **When** they advance to step 2, **Then** they see a grid of language flags with language names, with the most common languages prominently placed.
4. **Given** the user is on step 3, **When** they view translation options, **Then** they see a toggle to enable/disable translation and a direction selector (source → target).
5. **Given** the user reaches step 4, **When** they view the summary, **Then** it shows their selected configuration with a keyboard shortcut reference and a "Get Started" button.
6. **Given** the wizard is visible, **When** the user clicks "Skip", **Then** the wizard dismisses and the dashboard loads with default settings.
7. **Given** the wizard is active, **When** the user views progress, **Then** dots indicate current step (1 of 4) with completed steps filled.

---

### User Story 4 - Micro-Interactions & Visual Polish (Priority: P4)

**As a** user, **I want** the app to feel responsive and polished with subtle animations, **so that** interactions feel intentional and the app feels premium.

**Why this priority**: Micro-interactions are polish that elevates the experience but aren't core functionality. They layer on top of the structural changes from P1-P3.

**Independent Test**: Can be tested by hovering cards, clicking buttons, toggling switches, and triggering toasts. Each interaction has appropriate animation. Delivers visual polish across all components.

**Acceptance Scenarios**:

1. **Given** the user hovers over a card, **When** the cursor enters the card bounds, **Then** a subtle radial gradient glow effect follows the cursor position.
2. **Given** the user clicks a toggle switch, **When** the toggle is activated, **Then** the knob animates with spring physics (overshoot + settle).
3. **Given** the user clicks a button, **When** the click registers, **Then** a ripple effect emanates from the click point and fades out.
4. **Given** a toast notification appears, **When** it slides in, **Then** it animates from the edge with a timer bar that decreases over the dismiss duration.
5. **Given** data is loading, **When** the content is not yet available, **Then** skeleton loading states display with a shimmer animation matching the content shape.
6. **Given** the user focuses an interactive element, **When** the element receives focus, **Then** a 2px accent-colored focus ring with 2px offset appears.
7. **Given** keyboard shortcuts exist, **When** the user views UI elements with shortcuts, **Then** `<kbd>` elements display the shortcut in monospace with subtle styling.

---

### Edge Cases

- What happens when the user has no model loaded? Dashboard shows "No model loaded" with a quick action to open Model Manager.
- What happens when audio capture fails? Dashboard audio meter shows "Audio unavailable" with an error indicator.
- What happens when the user resizes the window? Layout adjusts within the fixed Tauri window constraints; sidebar collapses to icons if width < 600px.
- What happens during onboarding if the user selects an invalid configuration? Inline validation prevents advancement with a clear error message.
- What happens if the user skips onboarding? Default settings are applied and the wizard never appears again (config flag `onboarding_completed = true`).
- What happens if micro-interaction performance drops below 60fps? Animations degrade gracefully — reduce to simple opacity transitions.

## Requirements *(mandatory)*

### Functional Requirements

**Dashboard Overview**

- **FR-001**: System MUST display a persistent sidebar with navigation items: Dashboard, Transcriptions, Export, Audio, Settings, Overlay, Shortcuts.
- **FR-002**: Sidebar MUST highlight the active section and support click-to-navigate.
- **FR-003**: Dashboard MUST display status hero cards for Whisper model, Translation state, and Audio source.
- **FR-004**: Dashboard MUST display a quick actions grid with Toggle Overlay, View History, Export, and Settings buttons.
- **FR-005**: Dashboard MUST display a Live Transcription panel that adapts based on overlay state (feed when OFF, status when ON).
- **FR-006**: Dashboard MUST display an audio level meter showing real-time RMS energy during capture.
- **FR-007**: Dashboard MUST display a Models Status panel showing loaded model info.

**Settings Reorganization**

- **FR-008**: Settings MUST replace nested tabs with flat, scrollable sections.
- **FR-009**: Settings MUST provide context-aware sub-nav (General, Appearance, Advanced) that filters visible sections.
- **FR-010**: Model selection MUST display cards with visual tier badges (Fast/Balanced/Precise).
- **FR-011**: All settings controls (sliders, toggles, dropdowns) MUST persist changes inline without explicit save buttons.
- **FR-012**: Advanced settings MUST be grouped under collapsible sections that default to collapsed.

**Quick Start Onboarding**

- **FR-013**: System MUST detect first launch and display the onboarding wizard before showing the dashboard.
- **FR-014**: Wizard MUST present 4 steps: Model Selection, Language Selection, Translation Setup, Ready Summary.
- **FR-015**: Wizard MUST include a "Skip" option that applies defaults and dismisses permanently.
- **FR-016**: Wizard MUST display progress dots indicating current step and completed steps.
- **FR-017**: Wizard MUST validate selections before allowing advancement to the next step.

**Micro-Interactions & Polish**

- **FR-018**: Card hover MUST display a radial gradient glow effect following the cursor position.
- **FR-019**: Toggle switches MUST animate with spring physics (overshoot + settle).
- **FR-020**: Button clicks MUST trigger a ripple effect emanating from the click point.
- **FR-021**: Toast notifications MUST slide in with a timer bar that decreases over the dismiss duration.
- **FR-022**: Loading states MUST display skeleton placeholders with shimmer animation.
- **FR-023**: Focused elements MUST show a 2px accent-colored focus ring with 2px offset.
- **FR-024**: Keyboard shortcut badges MUST render as `<kbd>` elements with monospace styling.

**Cross-Cutting**

- **FR-025**: System MUST maintain existing keyboard shortcuts (Ctrl+Shift+S for capture, etc.) throughout the UI reorganization.
- **FR-026**: System MUST preserve all existing functionality — no feature regressions allowed.
- **FR-027**: System MUST maintain the existing dark theme and design tokens (slate-based neutrals, blue-500 accent).
- **FR-028**: All animations MUST maintain 60fps; degrade gracefully if performance drops.
- **FR-029**: System MUST work within Tauri's fixed window constraints (no web-responsive layout).
- **FR-030**: No new npm dependencies — use existing framer-motion and lucide-react.

### Key Entities

- **Sidebar Navigation**: Persistent left panel with section icons and labels. Active state highlights current section. Collapsible to icon-only mode at narrow widths.
- **Status Hero Card**: Compact card displaying a subsystem's current state (model name + status, translation enabled/disabled, audio device name). Used on the dashboard.
- **Quick Action Button**: Tappable card with icon and label that navigates to a section or triggers an action (toggle overlay). Used in the dashboard grid.
- **Live Transcription Panel**: Adaptive panel showing real-time text feed (overlay OFF) or status message (overlay ON). Auto-scrolls to latest text.
- **Settings Section**: Flat, grouped container for related settings. Each section has a title, description, and inline controls.
- **Tier Badge**: Visual indicator on model cards showing classification (Fast, Balanced, Precise) with color coding.
- **Onboarding Wizard**: Overlay component with 4 steps, progress indicator, skip option, and validation logic.
- **Animation Effect**: Reusable interaction pattern (glow, ripple, spring toggle, toast slide) implemented via framer-motion.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can reach any settings section in ≤2 clicks from the dashboard (down from 3+ clicks in the current tab system).
- **SC-002**: First-time users complete onboarding in under 90 seconds.
- **SC-003**: All micro-interactions animate at 60fps (measured via Chrome DevTools Performance panel during testing).
- **SC-004**: 100% of existing functionality remains accessible and functional after the UI reorganization (regression test: all existing user stories pass).
- **SC-005**: Settings changes persist immediately without user-initiated save action (verified by checking TOML config file after each change).
- **SC-006**: Dashboard loads within 200ms of navigation (perceived instant).
- **SC-007**: No visual regressions in dark theme contrast ratios (maintain WCAG AA 4.5:1 for text).
- **SC-008**: Keyboard shortcuts function identically before and after the UI reorganization.

## Assumptions

- Users are on Linux (Arch, Wayland, Hyprland) — layout is optimized for desktop, not mobile/tablet.
- The Tauri window has a fixed size (typically 1024×768 or similar) — no responsive breakpoints needed.
- Existing design system classes (`.card`, `.btn`, `.input`, `.select`, `.tab-bar`, `.toggle-switch`) will be extended, not replaced.
- The current 7 feature directories (001–007) represent prior work — this spec focuses on UI/UX overlay without modifying backend Rust modules.
- Constitution v1.1.0 principles (offline-first, real-time performance, modular architecture, Linux-native, test-first) apply to all implementations.
- framer-motion and lucide-react are already installed and available — no dependency additions required.
- The sidebar replaces the top tab bar — the `.tab-bar` CSS class may be deprecated or repurposed for sub-navigation.
- Onboarding wizard state is stored in the TOML config file (`onboarding_completed = true`) — no new persistence layer needed.
- Micro-interactions are optional visual enhancements that degrade gracefully — if performance is impacted, they can be disabled via a settings toggle.
