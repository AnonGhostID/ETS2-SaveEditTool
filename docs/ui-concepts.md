# UI Concepts — Phase 1, Part A (Information Architecture)

Three **mutually exclusive** information architectures for the native ETS2/ATS save editor. Each one replaces the current top-bar + left-sidebar + central-panel shell with a different navigation model and page hierarchy; none is a recoloring or respacing of the existing screens. They are presented without a recommendation — the user picks one at the Phase 1 checkpoint (or supplies a hybrid, which will be recorded verbatim).

## Reading guide and hard constraints

- **Capability source of truth:** `docs/ui-audit.md` — nine top-level destinations, four sub-tab groups, launcher flows, three modal/toast surfaces, and 159 grouped backend-call rows. Every capability listed there appears in every concept's sitemap below.
- **Frozen backend:** all three concepts call only the audited commands and `save_session` helpers. No new command, state type, crate, or data source is assumed. Features the backend does not expose (for example a language selector — the current LANG control is a status-only indicator) are not invented.
- **egui 0.29 implementation envelope:** `ScrollArea`, `CollapsingHeader`, `Grid`, `Window`, `Area`, `ui.columns()`, and the existing shared `confirm_dialog` primitive are enough to build each concept. No WebView, no new dependency.
- **Safety invariants common to all three:** edits remain staged until explicit Save; a backup is created before a write (and on switch/clone where the backend exposes the toggle); every destructive action goes through the shared confirm dialog; validation, error and log surfaces are always reachable; backup/restore access is reachable from every screen.
- **PP#1 … PP#12** refer to the 12 IA pain points in `docs/ui-audit.md` §7.
- **Deep link** means one router call sets a destination *and* its step / tile / station. This replaces the current scattered mutation of `active_tab` plus four sub-tab enums from the sidebar, the stat tiles and the Save Health jump (PP#3).

---

## Concept 1 — Task Cockpit

**One-sentence rationale:** Make the unit of navigation a *task* rather than a destination: a stepper drives one workflow at a time, a Task Switcher replaces the flat rail, and a permanent Save Ledger makes the staged/saved model impossible to miss.

### Navigation model

There is no global destination rail and no command bar. The window has four fixed regions:

1. **Context breadcrumb** across the top: `GAME › PROFILE › SAVE › Workflow › Step`, with `Tasks ▾`, `Reload` and `Undo`.
2. **Stepper** on the left: numbered steps *of the current workflow only* (never a list of every feature).
3. **Step pane** in the center plus a narrow **Context facts** strip on the right.
4. **Save Ledger** docked along the bottom of every step, holding staged count, backup count/outcome, `Backups`, `Save staged edits` and `Errors & Logs`.

All workflows are reached through the **Task Switcher** overlay (button in the breadcrumb, keyboard shortcut, or a quick tile on the Overview workflow). The Stepper is the only in-workflow navigation; the Task Switcher is the only cross-workflow navigation. This is a deliberate break from the existing 21-row rail (PP#2).

### Complete sitemap

- **Task Switcher (global overlay, grouped workflow index)**
  - Overview; Save Data; Vehicle; Operations; Library; Safety; System.
  - Each entry deep-links to a workflow **and** step; it is the single navigation authority (PP#3).
- **Workflow: Overview** — steps:
  1. Select context — GAME / PROFILE / SAVE combos, `Reload`, language status indicator (status-only).
  2. Review snapshot — profile/save header card; Current Setup card (truck, fuel & wear bars, trailer, HQ); eight clickable stat tiles deep-linking into Save Editor / Vehicles / Garages workflows; ETS2 dataset counts where available.
  3. Quick actions — Repair + Refuel; Max All Skills; Scan Save Health (deep-links to Safety › Save Health step 2); Generate Market Jobs.
  4. Open a workflow — action cards for every workflow in the Task Switcher index.
  5. Recent activity — last five background events; staging reminder footer.
- **Workflow: Economy & Progression** — steps:
  1. Money, XP, level with level preview.
  2. Six skill sliders with backend range validation (ADR 0–63, others 0–6).
  3. Custom reset values (level / XP / money).
  4. Game config values — Traffic, Developer, Console, Convoy, Parking Doubles.
  5. Custom key/value rows for base config and save config; `Refresh Save Data`.
  6. Review — all staged changes for this workflow listed before returning to the ledger.
- **Workflow: Bus Career (Coaches)** — steps:
  1. Bus level / XP editor with level preview; bus statistics.
  2. Job history with formatted stop and bus names.
  3. Discovered bus stops — search and filter; formatted stop labels (the old internal segmented tabs become step headers).
  4. Apply bus XP; link back to Economy step 2.
- **Workflow: Player Vehicle & Trailer** — steps:
  1. Repair / refuel player truck.
  2. Truck license plate and odometer.
  3. Truck fuel and wear sliders.
  4. Player trailer repair, license plate, cargo mass.
  5. Review staged vehicle edits.
- **Workflow: Truck Switcher** — steps:
  1. Initialize change session.
  2. Searchable owned-truck table.
  3. Preview with warnings and `can_apply` / expected hash.
  4. Apply, with the `create_backup_on_switch` toggle surfaced here.
  5. Result and backup outcome.
- **Workflow: Trailer Switcher** — steps: initialize session → searchable owned-trailer table → preview (warnings/hash) → apply with backup toggle → result.
- **Workflow: Powertrain** — steps: load official catalog (game/version) → select engine and transmission → preview → apply with confirmation.
- **Workflow: Fleet Inventory** — single step: searchable, sortable truck and trailer tables.
- **Workflow: Garages** — steps:
  1. Load garage list; filters, sorting and per-garage warnings.
  2. Garage detail — buy, upgrade, set HQ, random resource assignment.
  3. Sell garage / sell empty / buy all — each with destructive confirm.
  4. Review results.
- **Workflow: AI Drivers** — steps: generate drivers (count, assign trucks) → assign/reassign → dismiss with confirm.
- **Workflow: Live Telemetry** — steps: plugin status and live telemetry stream (500 ms repaint) → ETS2 snapshot refresh.
- **Workflow: Dispatcher** — steps:
  1. Generate standard / universal jobs.
  2. Market and active jobs with search and sorting; cleanup of expired jobs.
  3. Job detail — accept / cancel, risk and bonus notes, `last_error_message`.
  4. Assign + prepare + write to quicksave, with the staged-vs-written status label.
- **Workflow: Logbook & Analytics** — steps: trips / job log / job stats → profile job-history scan → analytics CSV export.
- **Workflow: VTC & Account** — steps: sign in / sign out → company overview and members → create company.
- **Workflow: Mod Manager** — steps: light/deep scan (background) → manual Workshop directory set/clear → installed mod table → Steam console and Workshop page links.
- **Workflow: Presets & Sandboxes** — steps: presets list / create / compare / export / import / delete (confirm) → sandboxes list / create / add mod / apply to active profile / delete (confirm).
- **Workflow: Conflict Analyzer** — steps: standard or deep conflict analysis → report view and report export.
- **Workflow: Save Health** — steps: scan active save → report → one-click fixes with confirm.
- **Workflow: Backups & Restore** — steps: list backups → preview backup → restore with confirm.
- **Workflow: Profile Clone & Rename** — steps: validate clone target → clone (pre-write backup + hex/text replacement options) → rename active profile → copy controls / mods to target profile.
- **Workflow: Profile Sharing** — steps: choose export directory and create ZIP export → pick import archive → inspect preview → import with confirm.
- **Workflow: Paths & Data** — steps: homedir/data-location status, custom path chooser, reset, mode → SQLite info and table counts → ETS2 dataset import.
- **Workflow: Logs & Diagnostics** — steps: log status and filterable user log list → diagnostic preview → export user logs / support bundle / log file → clear user logs with confirm.
- **Workflow: About** — single step: version footer and language status indicator (status-only).
- **Global overlays:** unsaved-changes prompt (Save / Discard / Cancel) for game/profile/save switch, Reload and Exit; destructive confirm dialog; toast stack.

### Text wireframe (main window)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ CONTEXT  ETS2 › QA Copy › QA Save › Truck Switcher › Step 3/5                │
│                                        [Tasks ▾]  [⟳ Reload]  [↶ Undo]       │
├──────────────────┬────────────────────────────────────────────┬──────────────┤
│ STEPS            │ STEP 3 — Review switch preview             │ CONTEXT      │
│ 1 Session    ✓   │ ┌ search owned trucks ───────────────────┐ │ Game: ETS2   │
│ 2 Select     ✓   │ │ | Truck | Garage | Status | Select |    │ │ Save: QA     │
│ 3 Preview   ◀    │ | ...                                 |    │ │ Staged: 3    │
│ 4 Apply          │ └────────────────────────────────────────┘ │ │ Backup: 4 OK │
│ 5 Result         │ Preview: warnings, can_apply, expected hash │ │ Health: OK   │
│                  │ Inline validation: "Save changed since load"│ │ [Issues ⚠2]  │
│                  │                    [← Back]  [Continue →]   │ │              │
├──────────────────┴────────────────────────────────────────────┴──────────────┤
│ SAVE LEDGER  ● 3 staged files  |  ◷ last backup 14:32 OK  |  [Backups]       │
│              [Create backup]  [Save staged edits]  [Errors & Logs ▴]        │
└──────────────────────────────────────────────────────────────────────────────┘
   Overlays (centered): unsaved prompt, destructive confirm, Task Switcher.
```

### Safety placements

- **(a) Automatic backup + backup/restore access:** the bottom ledger shows a backup count and last backup outcome continuously; `Backups` opens the Backups & Restore workflow from any step. The pre-write backup performed by Save is described on the pinned `Save staged edits` control; `create_backup_on_switch` and clone-backup toggles appear at the apply/clone steps with their effect mirrored in the ledger.
- **(b) Staged/saved status + Save action:** staged-file count and the `Save staged edits` button live in the bottom ledger on every step; the Context strip repeats the count. Save is disabled when clean. Game/profile/save switch, `Reload` and Exit invoke the unsaved prompt via the same router; switching workflows without changing context never discards staging.
- **(c) Error, validation and log surface:** validation messages render inline under the field or step that produced them and as a badge in the Context strip; transient outcomes use the toast stack; `Errors & Logs` in the ledger opens a persistent drawer with filterable validation / error / user-log / recent-activity tabs so a dismissed failure can be revisited.

### Pros and cons (grounded in PP#1–PP#12)

**Pros**
- Removes the 21-peer rail and introduces a notion of "current task" (PP#2); the stepper is bounded by one workflow.
- One deep-link router replaces three sources of navigation mutation (PP#3).
- Save state and backups become permanently visible instead of a small label in a crowded bar (PP#4, PP#5).
- The fragmented vehicle task becomes one workflow with explicit steps; save-health jumps land on a specific step (PP#6).
- The ledger makes "staged until Save" self-evident rather than prose (PP#11).
- Confirmation is uniform: every destructive step ends in the shared dialog (PP#10).

**Cons**
- A coordinator still has to own step state for every workflow; without per-workflow state objects this concept can reproduce the god object (PP#1).
- The Issues drawer must genuinely merge toasts, recent activity and user logs or the three-way split remains (PP#7).
- Every step needs its own loading / empty / error contract; a wizard-shaped step can hide "nothing loaded yet" (PP#8).
- Language remains a status-only indicator (PP#9); Paths & Data sits late in the Task Switcher index alongside Logs (PP#12).
- At the 960 px minimum width the three vertical regions plus the ledger demand careful responsive collapse, and infrequent workflows require an extra click through the Task Switcher.

---

## Concept 2 — Orbit Hub

**One-sentence rationale:** Replace destination lists with a two-level sector map: a small set of capability constellations is always visible, and detail opens in a dedicated pane with a vertical safety spine covering backup, staging and issues.

### Navigation model

Three columns and a fixed edge:

1. **Context Column** (left): stacked Game / Profile / Save selector card, `Reload`, language status, and live context facts. It is not a top bar.
2. **Sector Map** (center): seven tiles — Overview, Save Data, Vehicles, Operations, Library, Safety, System. Selecting a tile swaps the grid to that sector's capability tiles (progressive disclosure), then selecting a capability loads the Detail Pane.
3. **Detail Pane** (right): fields, tables, previews, inline validation and the action buttons for exactly one capability; its header carries a breadcrumb.
4. **Safety Spine** (far right, always visible): three gauges — Backups, Staged, Issues — plus `Save staged edits` and `Errors & Logs`.

There is no global rail, no command bar, and no bottom dock. Context changes (Game/Profile/Save) are the only navigation that can be blocked by the unsaved prompt; moving between sectors never blocks.

### Complete sitemap

- **Context Column (global):** GAME / PROFILE / SAVE selection; `Reload`; language status indicator (status-only); current save facts and health summary.
- **Sector: Overview**
  - Profile and save header card.
  - Current Setup card — truck, fuel & wear bars, trailer, HQ.
  - Eight stat tiles deep-linking into Save Data / Vehicles / Operations capabilities.
  - Quick actions — Repair + Refuel; Max All Skills; Scan Save Health (deep-link to Safety › Save Health); Generate Market Jobs.
  - Open-a-screen cards for every sector tile; RECENT ACTIVITY (last five events); staging reminder.
- **Sector: Save Data**
  - Economy & Experience — money, XP, level with level preview, six skill sliders, custom reset.
  - Project Coaches / Bus Career — bus level/XP, bus statistics, job history, discovered bus stops with search/filter and formatted names, internal segmented tabs.
  - Config keys — Traffic, Developer, Console, Convoy, Parking Doubles.
  - Custom key/value rows — base config and save config; `Refresh Save Data`.
- **Sector: Vehicles**
  - Player Vehicle — repair/refuel, plate, odometer, fuel & wear sliders.
  - Player Trailer — repair, plate, cargo mass.
  - Truck Switcher — session init, searchable table, preview with warnings and `can_apply`, apply with `create_backup_on_switch` toggle.
  - Trailer Switcher — session init, searchable table, preview, apply with backup toggle.
  - Powertrain Swapper — official catalog, engine/transmission selection and preview.
  - Fleet Inventory — searchable, sortable truck and trailer tables.
- **Sector: Operations**
  - Garages — filterable/sortable garage table with warnings; buy, upgrade, set HQ; sell, sell-empty and buy-all with confirm; random resource assignment.
  - AI Drivers — generate (count, assign trucks), assign/reassign, dismiss with confirm.
  - Live Telemetry — plugin status, 500 ms telemetry stream, ETS2 snapshot refresh.
  - Dispatcher — generate standard/universal jobs; market and active jobs with search/sort; accept/cancel; assign + prepare + write to quicksave with staged-vs-written label; cleanup expired.
  - Logbook & Analytics — trips, job log, job stats, history scan, CSV export.
  - VTC & Account — sign in/out, company overview, members, create company.
- **Sector: Library**
  - Mod Manager — light/deep scan (background), manual Workshop directory set/clear, installed mods, Steam console and Workshop links.
  - Presets — create, compare, export, import, delete with confirm.
  - Sandboxes — create, add mod, apply to active profile, delete with confirm.
  - Conflict Analyzer — standard and deep diagnostics, report view and export.
- **Sector: Safety**
  - Save Health — scan, report, one-click fixes with confirm.
  - Backups & Restore — list, preview, restore with confirm.
  - Profile Clone & Rename — validate target, clone with pre-write backup and hex/text options, rename, copy controls and mods.
  - Profile Sharing — ZIP export, archive pick, preview and import with confirm.
- **Sector: System**
  - Paths & Data — homedir/data-location status, custom path chooser, reset, mode; SQLite info and table counts; ETS2 dataset import.
  - Logs & Diagnostics — log status, filterable user log list, diagnostic preview, export logs / support bundle / log file, clear logs with confirm, recent activity and event errors.
  - About — version footer and language status indicator (status-only).
- **Global overlays:** unsaved-changes prompt (Save / Discard / Cancel) for game/profile/save switch, Reload and Exit; destructive confirm dialog; toast stack; Issues drawer launched from the spine.

### Text wireframe (main window)

```
┌──────────────┬───────────────────────────────────────────────────┬────────────────┬──┐
│ CONTEXT      │ SECTOR: VEHICLES              [◀ All sectors]     │ DETAIL         │S │
│ ┌──────────┐ │ ┌───────────┐ ┌───────────┐ ┌───────────┐         │ Truck Switcher │A │
│ │ ETS2   ▾ │ │ │ Player    │ │ Trailer   │ │ Truck     │         │ ─────────────  │F │
│ │ QA Copy▾ │ │ │ Vehicle   │ │ Editor    │ │ Switcher  │◀        │ search ...     │E │
│ │ QA Save▾ │ │ └───────────┘ └───────────┘ └───────────┘         │ preview ...    │T │
│ └──────────┘ │ ┌───────────┐ ┌───────────┐ ┌───────────┐         │ warnings ...   │Y │
│ Reload       │ │ Trailer   │ │ Powertrain│ │ Fleet     │         │ backup toggle  │  │
│ Lang: EN     │ │ Switcher  │ │ Swapper   │ │ Inventory │         │ [Apply]        │  │
├──────────────┤ └───────────┘ └───────────┘ └───────────┘         │ inline errors  │  │
│ Context facts│ Breadcrumb: Vehicles › Truck Switcher             ├────────────────│  │
│ save health  │                                                   │ Validation: …  │  │
└──────────────┴───────────────────────────────────────────────────┴────────────────┴──┘
   Safety Spine (right edge): [◷ Backups 4 OK] [◍ Staged 3] [✓ Save staged]
                              [⚠ Issues 2 → drawer: errors | validation | log | activity]
```

### Safety placements

- **(a) Automatic backup + backup/restore access:** the `Backups N / last outcome` gauge sits in the safety spine on every screen; clicking it opens the Safety › Backups & Restore tile in the Detail Pane. Backup toggles for truck/trailer switches and profile clone are rendered in the Detail Pane beside the apply action and summarized in the spine.
- **(b) Staged/saved status + Save action:** the `Staged N files` gauge and `Save staged edits` button are in the spine; the Context Column repeats the staged count beneath the selected save. Save is disabled when clean. The unsaved prompt guards game/profile/save selection, `Reload` and Exit only.
- **(c) Error, validation and log surface:** field-level validation appears in the Detail Pane under the offending control; the `Issues N` gauge opens a drawer with tabs for validation failures, errors, user logs and recent activity; transient outcomes toast in the corner. Settings read failures must write into the Issues drawer, not toast only.

### Pros and cons (grounded in PP#1–PP#12)

**Pros**
- Seven sectors plus one level of tiles replace 21 flat peers; progressive disclosure makes "where am I" explicit in the Detail Pane breadcrumb (PP#2).
- Context, capability browsing and detail are separated, so sector tiles and stat tiles can share one deep-link router (PP#3).
- Safety is a dedicated always-visible edge rather than one label in a wrapped bar (PP#4, PP#5).
- All five vehicle sub-capabilities are co-located in one sector, and save health sits next to backups in Safety (PP#6).
- System splits Paths & Data from Logs & Diagnostics, making recovery tasks findable (PP#12).
- The Issues drawer gives a single place to revisit dismissed failures (PP#7).

**Cons**
- The Detail Pane can become a second god-object coordinator unless each tile owns its own state (PP#1).
- Sector tiles, stat tiles and quick actions all deep-link; without one router they desynchronize (PP#3).
- Every tile needs a defined loading / empty / error state, and a background scan needs visible progress in the Detail Pane (PP#8).
- Language is still status-only (PP#9); confirmations must be routed through the shared dialog even when invoked from a tile (PP#10).
- Three columns plus the spine are tight at the 960 px minimum and need explicit responsive rules (e.g., collapse Context Column into the spine); the sector map also has a fixed tile capacity.

---

## Concept 3 — Save Ledger

**One-sentence rationale:** Make the save itself the primary object: one chronological ledger shows staged edits, the commit boundary, written history and expandable work stations, so save safety and the audit trail are the interface rather than an add-on.

### Navigation model

There is no rail, no top bar of commands, and no sector map. The main window is a single scrollable ledger with fixed sections:

1. **HEADER** — Game/Profile/Save selection, `Reload`, `Undo`, language status, profile/save card, Current Setup, eight stat tiles, staging reminder.
2. **STAGED** — one card per pending edit, with inline preview and discard; validation failures appear here as cards.
3. **COMMIT boundary** — a pinned card holding backup state, `Backups`, and `Save staged edits`.
4. **WRITTEN** — chronological results: backups created, save results, failures, `Undo`; reversible outcomes live here.
5. **STATIONS** — a searchable index of collapsing panels, one expanded at a time; each station holds all fields/tables for one capability.
6. **ISSUES drawer** — validation / errors / user log / recent activity.

Navigation is scrolling plus the Station Index (collapsing headers with search). Deep links from stat tiles or quick actions scroll to and expand the target station.

### Complete sitemap

- **HEADER (global):** GAME / PROFILE / SAVE selection; `Reload`; `Undo`; language status indicator (status-only); profile and save header card; Current Setup (truck, fuel & wear bars, trailer, HQ); eight stat tiles; staging reminder; last save outcome.
- **QUICK station:** Repair + Refuel; Max All Skills; Scan Save Health (deep-link to Save Health station); Generate Market Jobs; open-a-screen cards; Recent activity (last five events).
- **STATION: Economy & Progression** — money, XP, level with level preview; six skill sliders; custom reset; config values (Traffic, Developer, Console, Convoy, Parking Doubles); custom key/value rows for base and save config; `Refresh Save Data`.
- **STATION: Bus Career** — bus level/XP editor, bus statistics, job history, discovered bus stops with search/filter and formatted names, internal segmented tabs carried over as station sub-headers.
- **STATION: Player Vehicle & Trailer** — repair/refuel, plate, odometer, fuel & wear sliders; trailer repair, plate, cargo mass.
- **STATION: Truck Switcher** — session init, searchable table, preview with warnings/`can_apply`, apply with `create_backup_on_switch` toggle.
- **STATION: Trailer Switcher** — session init, searchable table, preview, apply with backup toggle.
- **STATION: Powertrain Swapper** — official catalog, engine/transmission selection and preview.
- **STATION: Fleet Inventory** — searchable, sortable truck and trailer tables.
- **STATION: Garages** — filterable/sortable table with warnings; buy, upgrade, set HQ; sell, sell-empty and buy-all with confirm; random resource assignment.
- **STATION: AI Drivers** — generate (count, assign trucks), assign/reassign, dismiss with confirm.
- **STATION: Live Telemetry** — plugin status, 500 ms telemetry stream, ETS2 snapshot refresh.
- **STATION: Dispatcher** — generate standard/universal jobs; market and active jobs with search/sort; accept/cancel; assign + prepare + write to quicksave with staged-vs-written status; cleanup expired; risk/bonus notes and `last_error_message`.
- **STATION: Logbook & Analytics** — trips, job log, job stats, history scan, CSV export.
- **STATION: VTC & Account** — sign in/out, company overview, members, create company.
- **STATION: Mod Manager** — light/deep scan (background), manual Workshop directory set/clear, installed mods, Steam console and Workshop links.
- **STATION: Presets & Sandboxes** — create/compare/export/import/delete presets with confirm; create/add-mod/apply/delete sandboxes with confirm.
- **STATION: Conflict Analyzer** — standard and deep diagnostics, report view and export.
- **STATION: Save Health** — scan, report, one-click fixes with confirm.
- **STATION: Backups & Restore** — backup list, preview, restore with confirm.
- **STATION: Profile Clone & Rename** — validate target; clone with pre-write backup and hex/text options; rename; copy controls and mods to target.
- **STATION: Profile Sharing** — export directory and ZIP export; archive pick; preview; import with confirm.
- **STATION: Paths & Data** — homedir/data-location status, chooser, reset, mode; SQLite info and table counts; ETS2 dataset import.
- **STATION: Logs & Diagnostics** — log status and filterable user log list; diagnostic preview; export user logs / support bundle / log file; clear logs with confirm; recent activity and event errors.
- **STATION: About** — version footer and language status indicator (status-only).
- **Ledger dynamics:** every successful edit adds a STAGED card; Save moves the entries to WRITTEN and records a backup card; failures stay in STAGED with an inline error card; restores and undo append WRITTEN cards.
- **Global overlays:** unsaved-changes prompt (Save / Discard / Cancel) for game/profile/save switch, Reload and Exit; destructive confirm dialog; toast stack.

### Text wireframe (main window)

```
┌────────────────────────────────────────────────────────────────────────────┐
│ SAVE LEDGER  ETS2 › QA Copy › QA Save       [Stations ▾] [⟳] [↶ Undo]       │
├────────────────────────────────────────────────────────────────────────────┤
│ HEADER   Profile card | Current Setup | 8 stat tiles | staging note | EN   │
├────────────────────────────────────────────────────────────────────────────┤
│ STAGED (3)                                                                 │
│  • money       1,000,000 → 50,000,000        [preview] [discard]           │
│  • truck plate AB-123    → CD-456            [preview] [discard]           │
│  • ⚠ XP value out of range — fix inline                                   │
│ ───────────── COMMIT   Backup: ON (4)  [Backups]  [Save staged edits] ──── │
├────────────────────────────────────────────────────────────────────────────┤
│ WRITTEN  ● 14:32 backup before last save  [Restore this]  [Undo]           │
│  • 14:30 save_pending_changes → 3 files written OK                         │
│  • 14:12 dispatcher job written — staged-vs-written status recorded        │
├────────────────────────────────────────────────────────────────────────────┤
│ STATIONS  [search…]  (one expanded)                                        │
│  ▸ Economy & Progression  ▸ Bus Career    ▸ Vehicle & Trailer              │
│  ▸ Truck Switcher  ▸ Trailer Switcher  ▸ Powertrain  ▸ Fleet Inventory     │
│  ▸ Garages  ▸ AI Drivers  ▸ Live Telemetry  ▸ Dispatcher                   │
│  ▸ Logbook & Analytics  ▸ VTC & Account  ▸ Mod Manager                     │
│  ▸ Presets & Sandboxes  ▸ Conflict Analyzer  ▸ Save Health                 │
│  ▸ Backups & Restore  ▸ Profile Clone & Rename  ▸ Profile Sharing          │
│  ▸ Paths & Data  ▸ Logs & Diagnostics  ▸ About                             │
│  ┌ expanded station: fields / tables / inline validation / [Apply] ──────┐ │
│  └───────────────────────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────────────────────────┤
│ ISSUES ▴  validation | errors | user log | recent activity                 │
└────────────────────────────────────────────────────────────────────────────┘
```

### Safety placements

- **(a) Automatic backup + backup/restore access:** the COMMIT boundary is a permanent card showing backup-on-write state, backup count and the last backup outcome; `Backups` deep-links to the Backups & Restore station from anywhere. Switch/clone backup toggles appear in their apply cards and their result appears as a WRITTEN backup card.
- **(b) Staged/saved status + Save action:** the STAGED section and the pinned `Save staged edits` button *are* the primary view of save state — the staged count is in the section title and the last save outcome is in HEADER and WRITTEN. The unsaved prompt guards game/profile/save switch, `Reload` and Exit; Cancel leaves the ledger untouched.
- **(c) Error, validation and log surface:** validation failures are STAGED cards next to the edit they block; operation failures are WRITTEN cards; the ISSUES drawer aggregates validation, errors, user logs and recent activity for revisit; toasts acknowledge transient success only.

### Pros and cons (grounded in PP#1–PP#12)

**Pros**
- The staged-vs-written model becomes the visible structure of the app, not a footer sentence (PP#11, PP#4).
- One Station Index owns deep links from stat tiles, quick actions and health scans (PP#3).
- Errors and validation are colocated with the edit and also collected in one ISSUES drawer, replacing the three-way split (PP#7).
- The HEADER carries only context, so lifecycle actions no longer share a wrapping command row (PP#5).
- Loading and empty states are natural ledger cards ("no staged edits", "no backups yet", "station not loaded"), making missing data explicit (PP#8).
- All destructive station actions, including Clear User Logs and Undo, end in one confirm path (PP#10).

**Cons**
- A single ledger can drift toward one giant state list; per-station state objects are required to avoid the god object (PP#1).
- The Station Index is still a long list (about 24 entries); grouping and search mitigate but do not remove this (PP#2).
- Paths & Data and Logs & Diagnostics are far down the scroll; emergency recovery needs a deep link from the safety surface (PP#12).
- Language remains status-only (PP#9); one expanded station at a time limits side-by-side comparison of, for example, dispatcher results and save data (PP#6).
- Chronological order is not task order, and the 960 px width needs collapsible station columns to keep forms usable.

---

## Coverage index

Every audit capability is present in all three sitemaps. This table is an index only; the authoritative detail is in the sitemaps above.

| Audit capability area | Concept 1 node | Concept 2 node | Concept 3 node |
|---|---|---|---|
| Game / profile / save selection, Reload | Overview step 1; Context breadcrumb | Context Column | HEADER |
| Undo, undo-disabled-while-dirty rule | Context breadcrumb | Safety spine / Context Column | HEADER + WRITTEN |
| Unsaved changes prompt (switch / reload / exit) | Global overlay | Global overlay | Global overlay |
| Toast surface, background events, recent activity | Overview step 5 + Errors & Logs drawer | Issues drawer + Overview sector | ISSUES drawer + QUICK station |
| Language status indicator | Overview step 5 / About | Context Column / About | HEADER / About |
| Launcher profile/save header, Current Setup, 8 stat tiles | Overview step 2 | Overview sector | HEADER |
| Quick actions: repair+refuel, max skills, scan health, generate jobs | Overview step 3 | Overview sector | QUICK station |
| Open-a-screen cards, staging reminder | Overview steps 4–5; Save Ledger | Overview sector; spine | QUICK station; HEADER |
| Economy: money, XP, level, skills, custom reset, level preview | Economy & Progression steps 1–3 | Save Data › Economy | Economy & Progression station |
| Config values + custom key/value rows + Refresh Save Data | Economy steps 4–6 | Save Data › Config keys / Custom rows | Economy station |
| Coaches: bus XP, statistics, job history, stops, internal tabs | Bus Career steps 1–4 | Save Data › Coaches | Bus Career station |
| Player truck repair/refuel, plate, odometer, fuel/wear | Player Vehicle & Trailer steps 1–3 | Vehicles › Player Vehicle | Vehicle & Trailer station |
| Player trailer repair, plate, cargo mass | Player Vehicle & Trailer step 4 | Vehicles › Player Trailer | Vehicle & Trailer station |
| Truck switcher: session/search/preview/apply/backup toggle | Truck Switcher steps 1–5 | Vehicles › Truck Switcher | Truck Switcher station |
| Trailer switcher: session/search/preview/apply/backup toggle | Trailer Switcher steps 1–5 | Vehicles › Trailer Switcher | Trailer Switcher station |
| Powertrain: catalog, engine/transmission preview | Powertrain steps 1–4 | Vehicles › Powertrain | Powertrain station |
| Fleet inventory search / sort | Fleet Inventory | Vehicles › Fleet Inventory | Fleet Inventory station |
| Garages: table, filters, sorting, warnings, buy/upgrade/HQ | Garages steps 1–2 | Operations › Garages | Garages station |
| Sell / sell-empty / buy-all with confirm | Garages step 3 | Operations › Garages (confirm) | Garages station (confirm) |
| Random resource assignment | Garages step 2 | Operations › Garages | Garages station |
| AI drivers: generate, assign, dismiss with confirm | AI Drivers steps 1–3 | Operations › AI Drivers | AI Drivers station |
| Live telemetry, plugin status, snapshot refresh | Live Telemetry steps 1–2 | Operations › Live Telemetry | Live Telemetry station |
| Dispatcher: generate, market/active jobs, accept/cancel, write, cleanup | Dispatcher steps 1–4 | Operations › Dispatcher | Dispatcher station |
| Job risk/bonus notes, last_error_message, staged-vs-written label | Dispatcher steps 3–4 | Operations › Dispatcher | Dispatcher station |
| Logbook, job stats, history scan, CSV export | Logbook & Analytics steps 1–3 | Operations › Logbook & Analytics | Logbook & Analytics station |
| VTC sign in/out, company overview/members, create | VTC & Account steps 1–3 | Operations › VTC & Account | VTC & Account station |
| Mod Manager: light/deep scan, manual dir, Steam links | Mod Manager steps 1–4 | Library › Mod Manager | Mod Manager station |
| Presets: create/compare/export/import/delete + confirm | Presets & Sandboxes step 1 | Library › Presets | Presets & Sandboxes station |
| Sandboxes: create/add-mod/apply/delete + confirm | Presets & Sandboxes step 2 | Library › Sandboxes | Presets & Sandboxes station |
| Conflict analyzer: standard/deep, report export | Conflict Analyzer steps 1–2 | Library › Conflict Analyzer | Conflict Analyzer station |
| Save health: scan, report, one-click fixes + confirm | Save Health steps 1–3 | Safety › Save Health | Save Health station |
| Backups: list, preview, restore + confirm | Backups & Restore steps 1–3 | Safety › Backups & Restore | Backups & Restore station |
| Clone (backup, hex/text), validate, rename, copy controls/mods | Profile Clone & Rename steps 1–4 | Safety › Profile Clone & Rename | Profile Clone & Rename station |
| Profile sharing: ZIP export/import, archive preview + confirm | Profile Sharing steps 1–4 | Safety › Profile Sharing | Profile Sharing station |
| Homedir/status/chooser/reset/mode, SQLite info/counts, dataset import | Paths & Data steps 1–3 | System › Paths & Data | Paths & Data station |
| Log status/list/preview, exports, clear with confirm | Logs & Diagnostics steps 1–4 | System › Logs & Diagnostics | Logs & Diagnostics station |
| Version footer, language indicator | About | System › About | About station |
| Destructive confirm dialog | Global overlay + destructive steps | Global overlay + tile actions | Global overlay + station actions |

## Selection note

No concept is preferred here. At the checkpoint the user may pick Concept 1, 2 or 3, or describe a hybrid; the exact wording of the choice will be recorded in `docs/status.md` and used as the input to the Phase 1 spec.
