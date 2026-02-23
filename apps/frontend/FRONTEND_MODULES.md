# Frontend Module Plan (AirDrop-like UI)

This plan is intentionally incremental: implement and test one frontend module at a time.

## Module FE-1: App Shell + AirDrop-like Layout (Now)
- Build the base desktop-like window UI.
- Left panel: user/device profile and quick actions.
- Main panel: nearby devices grid/cards.
- Bottom section: transfer queue placeholder.
- Add responsive behavior and design tokens.

## Module FE-2: Device Discovery UI State Binding
- Bind nearby-device cards to dynamic state.
- Online/busy/offline badges.
- Empty/loading/error states.

## Module FE-3: Drag & Drop Send Flow
- Drag area + file picker.
- Receiver selection (multi-select).
- Pre-send validation and confirmation.

## Module FE-4: Incoming Request Modal
- Accept / Decline interaction.
- Display file metadata and sender details.
- Timeout/auto-dismiss states.

## Module FE-5: Transfer Dashboard
- Per-transfer progress bars and speed.
- Pause/resume/cancel buttons.
- Completed/failed status timeline.

## Module FE-6: Security/Trust UI
- Device fingerprint display and trust actions.
- First-time trust prompt.
- Session security status indicator.

## Module FE-7: Settings & Network Mode Controls
- Offline LAN-only toggle.
- Relay/Internet mode preferences.
- Update channel + diagnostics panel.

## Module FE-8: Accessibility + Polish
- Keyboard navigation, focus states, screen-reader labels.
- Motion-reduced mode.
- Visual polish and micro-interactions.

---

## Current implementation status
- ✅ FE-1 implemented
- ✅ FE-2 implemented
- ✅ FE-3 implemented
- ✅ FE-4 implemented
- ✅ FE-5 implemented
- ✅ FE-6 implemented
- ✅ FE-7 implemented
- ✅ FE-8 implemented
- 🎯 Frontend modules complete
