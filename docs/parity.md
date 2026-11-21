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
