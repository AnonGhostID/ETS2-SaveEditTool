# UI Rebuild Status


## Current phase
Complete — awaiting user build/tests per docs/qa.md.

## Done
- Created `ui-rebuild` branch from `main`
- Created `docs/` and `docs/screenshots/`
- Initialized tracking documents
- Phase 0: `docs/ui-audit.md` (404 lines; 159 grouped backend-call rows, ~188 call sites, all 47 `block_on` sites mapped); handoff note appended; backend gate clean; commit `121ddf4`
- Phase 1 Part A: `docs/ui-concepts.md` (398 lines, three concepts); commit `be14465`; gate clean
- Layout decision: **Concept 2 — Orbit Hub** (orchestrator's pick; see decision note below)
- Phase 1 concept checkpoint: user delegated the choice to the orchestrator
- Phase 1 Part B: `docs/ui-spec.md` (475 lines; 30 routes, 209 mapping rows, zero NO HOME); commit `95b2df6`; gate clean
- Phase 2 split into two disjoint slices: FoundationShell (shell/components/router/docs/components.md) and FoundationData (all of gui/data/)
- Phase 2: FoundationData (14 data files, 165 wrappers, 159 audit rows covered, commit `eb1dca0`) and FoundationShell (shell, components, 30-route/77-subview contract, docs/components.md, commits `8693beb`, `fd97f12`); gate clean
- Phase 3 pilot: `gui/screens/profile.rs` (four Safety tiles, 18 parity rows, commit `679ed05`) adapted to shell-owned route context (commit `0a5c887`)
- Foundation follow-up: ScreenCtx gained `route`/`sub_view`; screen-owned confirm focus trapping added (commit `53f967f`)
- Phase 3 fan-out: six area screens (economy, fleet, garages, career, mods, settings+launcher); 159/159 audit IDs ticked in docs/parity.md
- Phase 4: entry point switched to `gui::shell::AppShell::new`, old tab UI deleted, 30/30 routes dispatched; commit `d49965d`
- Phase 5: docs/qa.md checklist (commands, copy-only fixture, flow walk, UI checks, parity); commit `711ac98`
- Static review found 5 compile errors and 6 save-safety defects; all fixed and re-verified by focused re-review (no remaining findings in the reviewed set)
- Backend gate empty at every phase; no backend requests logged

## Blockers
None.

## User decision (interjection during Phase 0)
The user will perform all build and testing themselves. No orchestrator or sub-agent build/test/clippy/fmt runs. Verification artifacts (`docs/qa.md`) will be prepared as a ready-to-run checklist with exact commands, fixture setup, and parity rows for the user to execute. Pilot checkpoint will present the implemented screen and exact run commands instead of an orchestrator-produced screenshot.

Risk noted: without compile checks between phases, errors can accumulate until the user's final build; agent output will be reviewed statically and the user's build results will drive fixes.

## User decisions (later interjections)
- Layout choice delegated to the orchestrator; chosen **Concept 2 — Orbit Hub** (seven sectors, detail pane, Safety Spine, Issues drawer). Reason: collapses 21 flat destinations into seven, gives each capability its own detail pane for parallel screen work, and keeps backup/staged/error state always visible without wizard-step or long-scroll complexity.
- **No further confirmations.** All remaining checkpoints are waived; the orchestrator picks and proceeds. The user receives the final result only. This overrides the Phase 3 pilot-approval and Phase 4 deletion-approval checkpoints in the original plan.

## Remaining
- User runs build, lint, tests and the copy-save walk exactly as specified in docs/qa.md; this is the authoritative compile/runtime verification.
- No backend requests are outstanding; docs/backend-requests.md contains "No requests yet."

## Checkpoints
| # | Checkpoint | Status |
|---|---|---|
| 1 | Phase 0 audit summary | Passed — proceed to design |
| 2 | Phase 1 layout choice | Delegated to orchestrator — Concept 2 (Orbit Hub) |
| 3 | Phase 3 pilot approval | Waived by user — no confirmations |
| 4 | Phase 4 old-UI deletion approval | Waived by user — no confirmations |
