# Foundation requests

Screen agents append shared-component requests here. Only the foundation agent edits shared UI code.

## ProfileScreen (Phase 3 pilot)

1. **Route visibility in `ScreenCtx`:** `gui/shell/detail_pane.rs` dispatches all four `safety_*` routes to one `profile::render` call, and `ScreenCtx` carries only `data` + `commands`. The screen therefore cannot tell which capability tile the Sector Map selected; it currently renders a local four-way tile selector and pushes `Navigate` when the user switches tabs. Please add the current `route` and `sub_view` to `ScreenCtx` (or pass them to `render`) so screen content follows deep links from the Sector Map / Safety Spine. The pilot will adopt the change without changing its own state contract.
2. **Focus trapping for screen-owned confirm dialogs:** `AppShell::handle_tabs` only counts shell overlays (`self.confirm`, unsaved prompt, Issues drawer). Confirm dialogs opened from a screen via `components::confirm_dialog` use `modal_focus_id(0/1)` but are not part of that Tab cycle. Please expose an `AppShell` flag/API (or make `confirm_dialog` register its focus ids globally) so Tab/Shift+Tab stay inside screen confirmations and Escape cancels them.

## FoundationFix — residual component work (follow-up)

1. **Explicit modal button ids and Escape cancel in the shared root are still
   needed for complete focus fidelity.** `components::confirm_dialog`
   (`gui/components.rs:248-275`) wraps its buttons in
   `ui.push_id(modal_focus_id(0/1), ...)`; egui derives the real widget id from
   that salt plus the enclosing Ui stack, so `AppShell`'s
   `memory.request_focus(modal_focus_id(index))` cannot target the actual
   buttons (this pre-existing mismatch also affects the shell's own overlays).
   Required component change: give the confirm/cancel buttons explicit widget
   ids equal to `modal_focus_id(0)` / `modal_focus_id(1)` (e.g. via
   `Button::new(...).id(modal_focus_id(index))`) and treat `Escape` as the
   cancel button (`Some(false)`) inside `confirm_dialog`. The shell-side part
   is implemented already in `gui/shell/mod.rs`: screen modals are detected
   from visible `Order::Middle` layers, Tab/Shift+Tab are consumed and cycled
   across two slots, and focus is restored to the captured opener on close.
2. **Opener capture is best-effort without screen/component cooperation.**
   egui does not focus a button on click and `Memory` does not expose the last
   interested widget publicly, so `screen_modal_focus_return` captures
   `memory.focused()` at detection time. Screens that open a confirm dialog
   should expose/announce the opener id (or `confirm_dialog` should return it)
   so restoration targets the exact opener rather than any previously focused
   widget.

## CareerScreen (Phase 3 fan-out)

1. **Typed `CreateCompanyInput` re-export from `gui/data/vtc.rs`.** The
   `vtc::create_company` wrapper takes
   `crate::features::vtc::models::CreateCompanyInput`, but screens may not
   import `crate::features`. The career screen currently constructs the input
   through `serde_json::from_value(json!({...}))`, which is type-inferred at
   the wrapper call and works with the existing derive, but a
   `pub use crate::features::vtc::models::CreateCompanyInput;` in the data
   module (matching the garages fix) would let the screen build it directly.
2. **Data wrapper for the selected game.** Company creation uses the active
   game from `career::status().active_game` with an `"ets2"` fallback because
   `gui/data/context.rs` has no wrapper for `AppProfileState::selected_game`.
   Please add `context::selected_game(&DataContext) -> Result<String, UiError>`
   so the form can match the Context Column selector exactly.
3. **Data wrapper for the selected profile path.** Analytics history scanning
   passes `None` to `career::scan_profile_job_history`, which the backend
   resolves to the active profile; the old screen passed
   `app.selected_profile_path`. A `context::selected_profile_path` wrapper
   would allow exact parity without state access in the screen.
