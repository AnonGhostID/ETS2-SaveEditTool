# Foundation requests

Screen agents append shared-component requests here. Only the foundation agent edits shared UI code.

## ProfileScreen (Phase 3 pilot)

1. **Route visibility in `ScreenCtx`:** `gui/shell/detail_pane.rs` dispatches all four `safety_*` routes to one `profile::render` call, and `ScreenCtx` carries only `data` + `commands`. The screen therefore cannot tell which capability tile the Sector Map selected; it currently renders a local four-way tile selector and pushes `Navigate` when the user switches tabs. Please add the current `route` and `sub_view` to `ScreenCtx` (or pass them to `render`) so screen content follows deep links from the Sector Map / Safety Spine. The pilot will adopt the change without changing its own state contract.
2. **Focus trapping for screen-owned confirm dialogs:** `AppShell::handle_tabs` only counts shell overlays (`self.confirm`, unsaved prompt, Issues drawer). Confirm dialogs opened from a screen via `components::confirm_dialog` use `modal_focus_id(0/1)` but are not part of that Tab cycle. Please expose an `AppShell` flag/API (or make `confirm_dialog` register its focus ids globally) so Tab/Shift+Tab stay inside screen confirmations and Escape cancels them.
