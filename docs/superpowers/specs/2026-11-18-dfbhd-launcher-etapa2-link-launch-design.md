# DFBHD Texture Launcher — Etapa 2 (Link & Launch) Design

**Date:** 2026-11-18  
**Status:** Proposed (pending explicit approval)  
**Builds on:** Etapa 1 UI shell (`docs/superpowers/specs/2026-11-18-dfbhd-launcher-etapa1-ui-design.md`)

## Intent / Why
Advance the launcher from UI stubs to a usable flow for DFBHD: let the user attach an existing installed game folder, detect key files, persist the link, and launch the game from the launcher. This unblocks later texture/profile work by knowing where the game and `.pff` assets live.

## Goals (Etapa 2, Option A)
- **Attach installed folder:** `SCAN/ATTACH` opens a native folder picker; user selects an existing DFBHD install directory.
- **Detect/validate:** locate `dfbhd.exe` (case-insensitive) in the selected folder; recursively enumerate `.pff` files (case-insensitive). If `dfbhd.exe` is found, mark source **linked** and record paths; if not, keep unlinked and log a clear reason.
- **Store paths:** extend `GameNode`/state to hold install directory, executable path, and discovered `.pff` paths (and derived labels/counts for UI). `source_label` reflects the linked folder.
- **Persist:** save/load launcher config as `launcher-config.json` (stored next to the launcher executable; fallback to current working directory) so the DFBHD link is remembered across sessions. Store per-game data keyed by game id.
- **UI:** DFBHD screen shows linked status, install path, detection summary (exe found + count/names of `.pff` files), and allows re-attach/change folder. Log records attach outcomes.
- **Launch enabled:** when linked, `LAUNCH` starts `dfbhd.exe` with the install directory as working directory (spawned, non-blocking) and logs success/failure. When not linked, `LAUNCH` remains guarded with a clear attach/scan/link message.

## Non-goals (Etapa 2)
- No ISO mounting/reading and no InstallShield CAB extraction (Option B/C deferred).
- No binary patching, wrappers (DXVK/d3d8to9/DSOAL), LAA/config generation.
- No PFF parsing, texture extraction/upscale, or real texture/profile application.
- No network/manifest download.

## Constraints
- Windows target, BYOG (user supplies legally obtained installed game).
- Keep Etapa 1 vault UI/theme and structure; evolve existing modules.

## Approach (chosen)
**Option A: link existing installed game folder.** Simple, reliable across Steam/GOG/retail installs; avoids proprietary InstallShield extraction.

## Design Decisions (calls unless corrected)
- **Folder picker:** add `rfd` crate for native directory dialog triggered by `SCAN/ATTACH` in the egui UI.
- **Detection rules:** `dfbhd.exe` matched case-insensitively in the selected root directory. `.pff` files discovered by recursive scan (case-insensitive extension) under the install directory; store relative and/or absolute paths and show count + representative names in UI.
- **Persistence format:** `serde` + `serde_json` for `launcher-config.json` containing per-game link data (e.g. `dfbhd`: `source_linked`, `install_dir`, `exe_path`, `pff_paths`). Load on startup; save on successful attach/unlink changes.
- **Config location:** directory of `std::env::current_exe()` when available, else current working directory.
- **Launch:** `std::process::Command::new(exe_path).current_dir(install_dir).spawn()`; log outcome. No blocking wait. Guarded only when unlinked or exe missing.
- **Error handling:** invalid folder/no exe/missing config -> user-visible log/status, no panic; app remains usable.

## UI Sketch (DFBHD linked)
```text
DFBHD // SOURCE
Source: C:\Games\DFBHD [LINKED]
Found: dfbhd.exe ✓  |  PFFs: 6 (RESOURCE.PFF, dfbhd.pff, ...)
[SCAN/ATTACH (change)] [APPLY PROFILE] [LAUNCH]
Log: linked install folder; detected exe + 6 .pff files
     launch started: C:\Games\DFBHD\dfbhd.exe
```

## What's left to later etapas
- ISO attach/reference and/or InstallShield extraction (if needed).
- Texture pipeline/profile application (PFF handling, HD packs).
- Wrappers/patching, multi-game generalization.
