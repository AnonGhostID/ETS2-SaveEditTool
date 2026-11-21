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
