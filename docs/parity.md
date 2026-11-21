# Feature parity checklist

Every row of `docs/ui-audit.md` must appear here and be ticked by the owning screen agent.

| Area | Feature | Old location | New location | Done |
|---|---|---|---|---|

## Profile / Safety sector (Phase 3 pilot — `gui/screens/profile.rs`)

| Area | Feature | Old location | New location | Done |
|---|---|---|---|---|
| Profile Clone & Rename | Clone & Rename sub-view map | Profile Tools sub-tabs | Safety › Profile Clone & Rename › Clone / Rename / Copy controls / Copy mods | ✓ |
| Profile Clone & Rename | C-119 validate clone target | Profile Clone | Safety › Profile Clone & Rename › Clone (Validate target) | ✓ |
| Profile Clone & Rename | C-120 clone with backup + hex/text options, confirmed | Profile Clone | Safety › Profile Clone & Rename › Clone (confirm dialog) | ✓ |
| Profile Clone & Rename | C-121 rename active profile, confirmed | Profile Clone | Safety › Profile Clone & Rename › Rename (confirm dialog) | ✓ |
| Profile Clone & Rename | C-122 copy profile controls, confirmed | Profile Clone | Safety › Profile Clone & Rename › Copy controls (confirm dialog) | ✓ |
| Profile Clone & Rename | C-123 copy mods to profile, confirmed | Profile Clone | Safety › Profile Clone & Rename › Copy mods (confirm dialog) | ✓ |
| Profile Sharing | C-124 load profile share context | Profile Sharing | Safety › Profile Sharing › Export (context card) | ✓ |
| Profile Sharing | C-125 pick ZIP export directory | Profile Sharing | Safety › Profile Sharing › Export (Browse) | ✓ |
| Profile Sharing | C-126 export shared profile ZIP | Profile Sharing | Safety › Profile Sharing › Export | ✓ |
| Profile Sharing | C-127 pick import archive | Profile Sharing | Safety › Profile Sharing › Import (Browse) | ✓ |
| Profile Sharing | C-128 inspect archive preview | Profile Sharing | Safety › Profile Sharing › Preview | ✓ |
| Profile Sharing | C-129 import shared profile, confirmed | Profile Sharing | Safety › Profile Sharing › Import / Preview (confirm dialog) | ✓ |
| Save Health | C-031 / C-130 scan active save health | Dashboard quick action / Profile Tools Health | Safety › Save Health › Scan | ✓ |
| Save Health | C-131 apply save health fix, confirmed | Profile Tools Health | Safety › Save Health › Fix (confirm dialog) | ✓ |
| Backups & Restore | C-132 list active save backups | Profile Tools Backups | Safety › Backups & Restore › List | ✓ |
| Backups & Restore | C-133 preview backup restore | Profile Tools Backups | Safety › Backups & Restore › Preview | ✓ |
| Backups & Restore | C-134 restore backup, confirmed | Backup confirm window | Safety › Backups & Restore › Confirm (confirm dialog) | ✓ |
| Safety spine linkage | Backup count + last outcome, staged-edit warning on restore | Absent (audit gap a) | Safety › Backups & Restore › status card + Restore | ✓ |

## Operations sector (Phase 3 fan-out — `gui/screens/career.rs`)
## Library sector (Phase 3 fan-out — `gui/screens/mods.rs`)

| Area | Feature | Old location | New location | Done |
|---|---|---|---|---|
| Live Telemetry | Four-tile sub-view dispatch | Career sub-tabs | Operations › Live Telemetry / Dispatcher / Logbook & Analytics / VTC & Account | ✓ |
| Live Telemetry | C-065 career status | Career refresh + Live Telemetry | Operations › Live Telemetry › Live (bridge status card) | ✓ |
| Live Telemetry | C-066 plugin status | Career refresh | Operations › Live Telemetry › Live (SDK plugin installed) | ✓ |
| Live Telemetry | C-067 career overview (company, bank, reputation, fleet) | Live Telemetry | Operations › Live Telemetry › Live (Company economy & bank card) | ✓ |
| Live Telemetry | Live telemetry sample with 500 ms repaint | Career tab repaint loop | Operations › Live Telemetry › Live (500 ms refresh + sample card) | ✓ |
| Live Telemetry | Active trip summary | Live Telemetry overview | Operations › Live Telemetry › Live (active trip line) | ✓ |
| Live Telemetry | C-081 ETS2 save snapshot refresh | Live Telemetry `Refresh Active Save Snapshot` | Operations › Live Telemetry › Save snapshot | ✓ |
| Live Telemetry | Loading/empty/error/validation + Retry | Status toast only | `loading_state` / `empty_state` / `error_state` + Retry / inline validation + Issues | ✓ |
| Dispatcher | C-068 dispatcher overview | Career refresh | Operations › Dispatcher › Generation (overview card) | ✓ |
| Dispatcher | C-069 dispatcher generation status | Career refresh | Operations › Dispatcher › Generation (status card) | ✓ |
| Dispatcher | C-079 generate standard market jobs | Dispatcher `Generate Market Jobs` | Operations › Dispatcher › Generation (Generate market jobs) | ✓ |
| Dispatcher | C-080 generate save-synced universal jobs | Dispatcher `Generate Universal Jobs` | Operations › Dispatcher › Generation (force toggle + Generate universal jobs) | ✓ |
| Dispatcher | C-082 cleanup expired jobs, confirmed | Dispatcher `Cleanup Expired Jobs` | Operations › Dispatcher › Generation (confirm dialog) | ✓ |
| Dispatcher | C-070 market jobs table | Dispatcher `Available Market Jobs` | Operations › Dispatcher › Jobs (Market jobs) | ✓ |
| Dispatcher | C-071 active jobs table | Dispatcher refresh | Operations › Dispatcher › Jobs (Active jobs) | ✓ |
| Dispatcher | Job search and sort | Dispatcher search + sort headers | Operations › Dispatcher › Jobs (search, sort select, ascending toggle) | ✓ |
| Dispatcher | C-083 accept dispatcher job | Job detail `Accept Job` | Operations › Dispatcher › Job detail (Accept job) | ✓ |
| Dispatcher | C-085 cancel dispatcher job, confirmed | Job detail `Cancel Job` | Operations › Dispatcher › Job detail (confirm dialog) | ✓ |
| Dispatcher | C-084 assign + prepare + write to quicksave, confirmed | Job detail `Write to Quicksave` | Operations › Dispatcher › Job detail (confirm dialog) | ✓ |
| Dispatcher | C-091 staged-vs-written label and Save action | Job detail staging label | Operations › Dispatcher › Job detail (staged message + `Save staged edits` command) | ✓ |
| Dispatcher | Job detail risk/bonus notes and last_error_message | Job detail card | Operations › Dispatcher › Job detail (risk, bonus and last error rows) | ✓ |
| Dispatcher | Loading/empty/error/validation + Retry | Status toast / silent empty | `loading_state` / `empty_state` / `error_state` + Retry / selection validation + Issues | ✓ |
| Logbook & Analytics | C-072 recorded trips table | Logbook `Recorded Trips` | Operations › Logbook & Analytics › Trips | ✓ |
| Logbook & Analytics | C-073 job log table | Logbook refresh | Operations › Logbook & Analytics › Job log | ✓ |
| Logbook & Analytics | C-074 job statistics | Logbook refresh | Operations › Logbook & Analytics › Stats | ✓ |
| Logbook & Analytics | C-075 analytics summary | Analytics refresh | Operations › Logbook & Analytics › Analytics | ✓ |
| Logbook & Analytics | C-086 profile job history scan | Analytics `Scan Profile Save Job History` | Operations › Logbook & Analytics › Analytics (Scan profile save job history) | ✓ |
| Logbook & Analytics | C-087 analytics CSV export | Analytics `Export Analytics CSV` | Operations › Logbook & Analytics › Analytics (Export analytics CSV) | ✓ |
| Logbook & Analytics | Loading/empty/error/validation + Retry | Status toast / silent empty | `loading_state` / `empty_state` / `error_state` + Retry / inline validation + Issues | ✓ |
| VTC & Account | C-076 current user profile | VTC account card | Operations › VTC & Account › Sign in | ✓ |
| VTC & Account | C-089 auth sign in with validation | VTC `Sign In` | Operations › VTC & Account › Sign in (required-field validation) | ✓ |
| VTC & Account | C-088 auth sign out, confirmed | VTC `Sign Out` | Operations › VTC & Account › Sign in (confirm dialog) | ✓ |
| VTC & Account | C-077 company overview | VTC company card | Operations › VTC & Account › Company | ✓ |
| VTC & Account | C-078 company members | VTC member list | Operations › VTC & Account › Members | ✓ |
| VTC & Account | C-090 create company with validation | VTC `Create VTC Company` | Operations › VTC & Account › Company (name/HQ validation) | ✓ |
| VTC & Account | Loading/empty/error/validation + Retry | Status toast / silent empty | `loading_state` / `empty_state` / `error_state` + Retry / form validation + Issues | ✓ |
| Operations save-safety | Backups/staged note, Issues drawer and Save staging status | Top-bar status only | Every Operations sub-view: staging note + `Open Issues` + Safety Spine Save | ✓ |
| Mod Manager | Four-tile sub-view map | Mods sub-tabs (Mod Manager / Presets / Conflict Analyzer) | Library › Mod Manager (Scan & Summary / Installed Mods / Workshop & Links), Presets, Sandboxes, Conflict Analyzer | ✓ |
| Mod Manager | C-098 load mod manager state | Mods Manager `Load State` | Library › Mod Manager › Scan & Summary (Load State) | ✓ |
| Mod Manager | C-099 open Steam console | Mods Manager `Open Steam Console` | Library › Mod Manager (Open Steam Console, also Workshop & Links) | ✓ |
| Mod Manager | C-100 deep mod scan with background progress | Mods Manager `Deep Scan Mods` | Library › Mod Manager (Deep Scan Mods + progress row) | ✓ |
| Mod Manager | C-101 light mod scan with background progress | Mods Manager `Quick Scan Mods` | Library › Mod Manager (Quick Scan Mods + progress row) | ✓ |
| Mod Manager | C-105 choose manual Workshop directory | Presets & Sandboxes `Set Manual Workshop Folder` | Library › Mod Manager › Workshop & Links (Set Manual Workshop Folder) | ✓ |
| Mod Manager | C-106 clear manual Workshop directory | Presets & Sandboxes `Clear Manual Workshop Folder` | Library › Mod Manager › Workshop & Links (Clear Manual Workshop Folder) | ✓ |
| Mod Manager | C-107 open Workshop page | Installed mod row `Steam Workshop` | Library › Mod Manager › Installed Mods (selected row) / Workshop & Links (first detected) | ✓ |
| Mod Manager | Installed mod table with order/source/status/workshop id | Mods Manager discovered-mods grid | Library › Mod Manager › Installed Mods (`components::table`) | ✓ |
| Mod Manager | Scan loading/empty/error/validation + Retry | Spinner / status toast | progress row + `loading_state` / `empty_state` / `error_state` with Retry / inline validation + Issues | ✓ |
| Presets | C-104 list mod presets | Presets & Sandboxes `Load Presets` | Library › Presets › Preset list (Load Presets + one-time load) | ✓ |
| Presets | C-108 import preset JSON (cross-game notice) | Presets & Sandboxes `Import Preset JSON` | Library › Presets › Preset list (Import Preset JSON) | ✓ |
| Presets | C-109 create preset from current mods | Presets & Sandboxes `Save Current as Preset` | Library › Presets › Preset list (name/notes form) | ✓ |
| Presets | C-110 compare preset with active profile | Presets & Sandboxes `Compare` | Library › Presets › Comparison (summary tiles, warnings) | ✓ |
| Presets | C-111 export preset JSON | Presets & Sandboxes `Export` | Library › Presets › Preset list (Export JSON) | ✓ |
| Presets | C-102 delete preset, confirmed | Mods confirm dialog | Library › Presets › Preset list (Delete preset → shared confirm dialog) | ✓ |
| Presets | Preset loading/empty/error/validation + Retry | Status toast / silent empty | `loading_state` / `empty_state` / `error_state` with Retry / name + selection validation + Issues | ✓ |
| Sandboxes | C-112 load mod sandboxes | Presets & Sandboxes `Load Sandboxes` | Library › Sandboxes (Load Sandboxes + one-time load) | ✓ |
| Sandboxes | C-113 create sandbox | Presets & Sandboxes `Create Sandbox` | Library › Sandboxes (Create sandbox form) | ✓ |
| Sandboxes | C-115 add mod to sandbox | Presets & Sandboxes `+ Add Mod` | Library › Sandboxes (target select + Workshop ID/URL) | ✓ |
| Sandboxes | C-114 apply sandbox to active profile, confirmed | Presets & Sandboxes `Apply to Active Profile` | Library › Sandboxes (Apply to Active Profile → shared confirm dialog with staged-edit warning and backup path) | ✓ |
| Sandboxes | C-103 delete sandbox, confirmed | Mods confirm dialog | Library › Sandboxes (Delete sandbox → shared confirm dialog) | ✓ |
| Sandboxes | Sandbox loading/empty/error/validation + Retry | Status toast / silent empty | `loading_state` / `empty_state` / `error_state` with Retry / title + target + input validation + Issues | ✓ |
| Conflict Analyzer | C-116 standard conflict diagnostics | Mods Analyzer `Run Standard Conflict Diagnostics` | Library › Conflict Analyzer (Run Standard Conflict Diagnostics) | ✓ |
| Conflict Analyzer | C-117 deep conflict diagnostics | Mods Analyzer `Run Deep Conflict Diagnostics` | Library › Conflict Analyzer (Run Deep Conflict Diagnostics) | ✓ |
| Conflict Analyzer | Report view (status, confidence, crash counts, suspects, missing refs) | Mods Analyzer report card + grid | Library › Conflict Analyzer (report card + suspected-mods table + detail lists) | ✓ |
| Conflict Analyzer | C-118 export conflict report | Mods Analyzer `Export Diagnostic Report` | Library › Conflict Analyzer (Export Diagnostic Report, requires completed report) | ✓ |
| Conflict Analyzer | Analysis loading/empty/error/validation + Retry | Blocking `block_on` + status toast | `loading_state` / `empty_state` / `error_state` with Retry / export validation + Issues | ✓ |
| Library save-safety | Issues surface and backup/staged visibility | Status toast only | All four tiles: `Open Issues` command + Safety Spine (shell-owned), backup path shown after sandbox apply, staged warning in confirm | ✓ |

## Save Data sector (Phase 3 fan-out — `gui/screens/economy.rs`)

| Area | Feature | Old location | New location | Done |
|---|---|---|---|---|
| Economy & Experience | C-136 edit player money + quick amount buttons | Save Editor › Economy | Save Data › Economy & Experience › Fields (Apply Money, +100k/+1M/Set 10M/Set 100M) | ✓ |
| Economy & Experience | C-137 edit truck XP from the true game.sii value | Save Editor › Economy | Save Data › Economy & Experience › Fields (Apply XP, truck XP from `try_read_truck_xp`) | ✓ |
| Economy & Experience | C-138 edit target level | Save Editor › Economy | Save Data › Economy & Experience › Fields (Apply Level) | ✓ |
| Economy & Experience | C-148 level preview with progress and capped next level | Save Editor › level card | Save Data › Economy & Experience › Level preview | ✓ |
| Economy & Experience | C-140 six skill sliders with ADR 0–63 / others 0–6 | Save Editor › Skills | Save Data › Economy & Experience › Skills (six sliders + Apply) | ✓ |
| Economy & Experience | Max All Skills (six staged writes), confirmed | Save Editor › Skills `Max All Skills` | Save Data › Economy & Experience › Skills (shared confirm dialog) | ✓ |
| Economy & Experience | C-139 custom reset values (money plus last-edited XP/level) | Save Editor › Economy batch | Save Data › Economy & Experience › Custom reset (and Fields batch button) | ✓ |
| Config Values | C-141 traffic density | Save Editor › Config | Save Data › Config Values (slider + Apply, 0–10) | ✓ |
| Config Values | C-142 developer mode | Save Editor › Config | Save Data › Config Values (toggle) | ✓ |
| Config Values | C-143 developer console | Save Editor › Config | Save Data › Config Values (toggle) | ✓ |
| Config Values | C-145 max convoy size | Save Editor › Config | Save Data › Config Values (slider + Apply, 1–128) | ✓ |
| Config Values | C-144 double trailer parking | Save Editor › Config | Save Data › Config Values (toggle from profile config) | ✓ |
| Custom Keys | C-146 base `config.cfg` custom key/value rows | Save Editor › Advanced custom keys | Save Data › Custom Keys › Base config keys (key/value table + Set) | ✓ |
| Custom Keys | C-147 save/profile `config.cfg` custom key/value rows | Save Editor › Advanced custom keys | Save Data › Custom Keys › Save config keys (key/value table + Set) | ✓ |
| Custom Keys | Non-empty key + duplicate-key warning and overwrite confirm | Save Editor inline status | Save Data › Custom Keys (inline validation + Issues + shared confirm) | ✓ |
| Refresh | C-135 / U-048 Refresh Save Data | Save Editor `Refresh Save Data` | Toolbar on all four Save Data tiles (re-reads save, skills, config and coaches snapshot) | ✓ |
| Project Coaches | C-092 snapshot read + explicit reload | Coaches `Load Coaches Data` | Save Data › Project Coaches (automatic load + Reload + Refresh Save Data) | ✓ |
| Project Coaches | C-093 / C-097 bus XP edit and derived bus level, hash-checked with confirm on ambiguous snapshots | Coaches career card | Save Data › Project Coaches › Overview (Bus XP + Set bus XP = truck XP) | ✓ |
| Project Coaches | Bus statistics table (distance, jobs, passengers, play time) | Coaches `BUS STATISTICS` grid | Save Data › Project Coaches › Statistics | ✓ |
| Project Coaches | C-094 job history with formatted bus and stop names | Coaches `JOB HISTORY` grid | Save Data › Project Coaches › Job history | ✓ |
| Project Coaches | C-095 / U-037 discovered stops with search and All/Discovered/Undiscovered filter | Coaches `BUS STOPS` search + segmented tabs | Save Data › Project Coaches › Bus stops | ✓ |
| Project Coaches | C-096 formatted bus names in job rows | Coaches job/bus labels | Save Data › Project Coaches › Job history (`display_bus_name`) | ✓ |
| Save Data states | Loading / empty / error + Retry / inline validation + Issues for every data view | Status toasts and silent empty panels | All four tiles (`loading_state`, `empty_state`, `error_state` with Retry, `validation_message` + `Validation:` toast) | ✓ |
| Save Data safety | Staged-edit note, Save owned by Safety Spine, no local backup/flush logic | Top bar Save + inline status | All four Save Data tiles (staging note + Issues opener; shell owns Save/Backups/Undo) | ✓ |

## Operations sector — Garages & AI Drivers (Phase 3 fan-out — `gui/screens/garages.rs`)
## System sector (Phase 3 fan-out — `gui/screens/settings.rs`)

| Area | Feature | Old location | New location | Done |
|---|---|---|---|---|
| Garages | C-054 all garages, headquarters and parse diagnostics | Garages & Drivers list | Operations › Garages › List / Detail / Warnings | ✓ |
| Garages | Search, Owned-only filter and five sort keys | Garages toolbar + sort headers | Operations › Garages › List / Detail (filter + sort controls) | ✓ |
| Garages | C-057 purchase an unowned garage | Garage detail | Operations › Garages › Detail (Buy garage) | ✓ |
| Garages | C-058 upgrade an owned garage | Garage detail | Operations › Garages › Detail (Upgrade) | ✓ |
| Garages | C-059 random driver + truck resource assignment | Garage detail | Operations › Garages › Detail (Assign driver + truck) | ✓ |
| Garages | C-060 set headquarters via garage update | Garage detail | Operations › Garages › Detail (Set as HQ) | ✓ |
| Garages | C-061 sell a garage, confirmed | Garage confirm window | Operations › Garages › Detail → shared confirm dialog | ✓ |
| Garages | C-062 sell empty garages, confirmed (strict sellable count, skipped names) | Garages bulk action | Operations › Garages › List → shared confirm dialog | ✓ |
| Garages | C-063 buy all garages, confirmed | Garages bulk action | Operations › Garages › List → shared confirm dialog | ✓ |
| AI Drivers | C-055 management list, available pool and truck list | Garage detail driver section | Operations › AI Drivers › Generate / Assign / Dismiss | ✓ |
| AI Drivers | C-056 generate drivers (count 1–5, assign-trucks toggle) | Garage detail | Operations › AI Drivers › Generate | ✓ |
| AI Drivers | Assign / reassign a specific driver's truck, and unassign | Backend command only; no old UI control | Operations › AI Drivers › Assign / reassign (`update_driver_truck`) | ✓ |
| AI Drivers | Random truck reassignment at a driver's garage | Garage detail random resources | Operations › AI Drivers › Assign / reassign (Reassign random truck) | ✓ |
| AI Drivers | C-064 dismiss one or many drivers, confirmed | Garage confirm window | Operations › AI Drivers › Generate / Dismiss → shared confirm dialog | ✓ |
| Dialogs | U-038 destructive confirm for sell, sell-empty, buy-all and dismiss | Garage confirm window | Shared `confirm_dialog` with staged-edits warning | ✓ |
| Load | U-043 explicit load via `BackgroundLoad` with repaint + Retry | Garages Load button | All Garages/AI Drivers sub-views (`loading_state`, `error_state` Retry) | ✓ |
| States | Loading / empty / error + Retry and inline validation for every data view | Status toasts and silent empty panels | All Garage/Driver sub-views (`empty_state`, `validation_message`, Issues + toast) | ✓ |
| Safety | Expected save hash on every mutation; staged count; Save/Backups owned by shell | Inline status + top bar | Detail Pane staged note + confirm warning + Issues opener; shell Safety Spine | ✓ |
| Paths & Data | C-024 ETS2 data-location status | Settings › Homedir | System › Paths & Data › Paths status (mode, dirs, validation) | ✓ |
| Paths & Data | C-152 choose custom ETS2 homedir | Settings › Homedir `Browse` | System › Paths & Data › Paths status (Browse, confirmed) | ✓ |
| Paths & Data | C-153 reset data location to default | Settings › Homedir `Reset` | System › Paths & Data › Paths status (Reset, confirmed) | ✓ |
| Paths & Data | C-154 set data-location mode (default/custom) | Settings › Homedir mode | System › Paths & Data › Paths status (Use default mode / Set custom path, confirmed) | ✓ |
| Paths & Data | Custom homedir existence validation and reload guard | Inline status | Inline `validation_message` + `Validation:` toast; context Reload goes through the unsaved prompt | ✓ |
| Paths & Data | C-025 SQLite info (path, size, tables) | Settings › SQLite | System › Paths & Data › SQLite info | ✓ |
| Paths & Data | C-026 SQLite table counts | Settings › SQLite | System › Paths & Data › SQLite info (table counts card) | ✓ |
| Paths & Data | C-155 ETS2 dataset import with summary | Settings › `Re-Import ETS2 Datasets` | System › Paths & Data › Dataset import (confirmed, summary + warnings) | ✓ |
| Paths & Data | Loading / empty / error + Retry for paths, SQLite and imports | Status toast | `loading_state` / `empty_state` / `error_state` with Retry + Issues error mirror | ✓ |
| Logs & Diagnostics | C-149 user-log status (path, size, warnings, errors) | Settings › Logs | System › Logs & Diagnostics › User logs | ✓ |
| Logs & Diagnostics | C-150 level-filtered, line-limited user log list | Settings › Logs | System › Logs & Diagnostics › User logs (level select + max-lines input) | ✓ |
| Logs & Diagnostics | C-151 diagnostic log preview | Settings › Log preview | System › Logs & Diagnostics › Diagnostic preview | ✓ |
| Logs & Diagnostics | C-156 export user logs | Settings › `Export User Logs` | System › Logs & Diagnostics › Exports | ✓ |
| Logs & Diagnostics | C-157 export support log bundle | Settings › `Export Support Log Bundle` | System › Logs & Diagnostics › Exports | ✓ |
| Logs & Diagnostics | C-158 export diagnostic log file | Settings › `Export Diagnostic Log (.txt)` | System › Logs & Diagnostics › Exports | ✓ |
| Logs & Diagnostics | C-159 clear user logs, confirmed | Settings › `Clear User Logs` (no confirm) | System › Logs & Diagnostics › User logs (shared confirm dialog) | ✓ |
| Logs & Diagnostics | Filter validation, error + Retry and Issues mirror | Status toast | `validation_message` + `Validation:` toast, `error_state` with Retry, `OpenIssues(Errors)` | ✓ |
| About | Version footer and language status indicator (C-002, PP#9 preserved) | Sidebar footer / LANG | System › About (version, language status, read-only notice) | ✓ |
| About | Recent activity and full event access | Dashboard `RECENT ACTIVITY` | System › About › Recent activity (last five user-log entries + Issues Activity) | ✓ |

## Overview sector (Phase 3 fan-out — `gui/screens/launcher.rs`)

| Area | Feature | Old location | New location | Done |
|---|---|---|---|---|
| Dashboard | U-014 profile/save header card | Dashboard header | Overview › Dashboard › Active profile & save | ✓ |
| Dashboard | U-015 / C-017 / C-019 Current Setup (truck, plate, fuel, wear, trailer, HQ) | Dashboard Current Setup | Overview › Dashboard › Current Setup | ✓ |
| Dashboard | U-016 eight stat tiles (money, XP, level, cities, trucks, trailers, garages, drivers) | Dashboard clickable tiles | Overview › Dashboard › At a glance — all eight deep-link via `ShellCommand::Navigate` | ✓ |
| Dashboard | C-033 level derived from XP | Dashboard level tile | Overview › Dashboard (`context::calculate_level`) | ✓ |
| Dashboard | U-018 OPEN A SCREEN sector cards | Dashboard action cards | Overview › Dashboard › Open a screen (sector default routes) | ✓ |
| Dashboard | U-020 staging reminder and backup visibility | Dashboard footer | Overview › Dashboard › Staging (live staged count; Safety Spine owns Save/Backups) | ✓ |
| Quick Actions | C-028 / C-029 Repair + Refuel Truck | Dashboard quick action | Overview › Quick Actions (both wrappers, staged outcome toast) | ✓ |
| Quick Actions | C-030 Max All Skills, confirmed | Dashboard quick action (no confirm) | Overview › Quick Actions (shared confirm dialog + per-skill error reporting) | ✓ |
| Quick Actions | C-031 Scan Save Health deep link | Dashboard quick action | Overview › Quick Actions → Safety › Save Health › Scan | ✓ |
| Quick Actions | C-032 Generate Market Jobs deep link | Dashboard quick action | Overview › Quick Actions → Operations › Dispatcher › Generation | ✓ |
| Recent Activity | U-019 last five events + error surfacing | Dashboard `RECENT ACTIVITY` | Overview › Recent Activity (last five persisted log entries + Issues Activity, error + Retry) | ✓ |
| Overview states | Loading / empty / error + Retry for header, stats, setup, quick actions and activity | Status toasts | `loading_state` / `empty_state` / `error_state` with Retry + Issues opener on every data view | ✓ |
