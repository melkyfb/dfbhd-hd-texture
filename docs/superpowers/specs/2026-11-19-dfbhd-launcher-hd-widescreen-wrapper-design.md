# DFBHD Launcher: HD Widescreen Wrapper Integration (Option A)

**Date:** 2026-11-19  
**Status:** Approved by user ("aprovado")  
**Scope:** `crates/launcher-gui` + project assets/docs (small-medium change)

## Intent and why
Enable a one-click "HD" experience for Delta Force: Black Hawk Down from the launcher: apply HD menu textures and install/configure a widescreen wrapper so the linked game can run at 1920x1080 (with safe restore to original state).

## Current state (relevant)
- Launcher links a DFBHD install (detects `dfbhd.exe` and `.pff` files), persists config, and launches guarded.
- DFBHD profiles today: `original`, `hd_textures`. Apply swaps/restores `localres.pff` in the install using bundled `assets/profiles/dfbhd/hd_textures/localres.pff` with safe backup `localres.pff.original` (never overwritten) and fallback from `.bak`.
- Wrapper zip extracted to `assets/wrapper/dfbhd_widescreen/` (d3d8/ASI loader: `d3d8.dll`, `d3d8.ini`, `dinput8.dll`, `scripts/BHD_Asi.asi`, `scripts/BHD_Asi.ini`, plus included build/license/readme files). Wrapper supports resolution override/FOV (`scripts/BHD_Asi.ini`: `ResolutionX/Y`, `FOV`).

## Goals
- Add a combined "HD Widescreen (1080p)" action/profile that applies HD textures and installs/configures the widescreen wrapper in the linked DFBHD install.
- Keep texture-only and original flows available and non-destructive to user data, with clear logging and no panics on IO/permission errors.
- Provide safe backup/restore so "Original" returns the install to a non-wrapper/original-texture state.

## Non-goals / out of scope
- Advanced in-launcher UI for resolution/FOV editing (wrapper config remains file-based in `.ini` for now).
- Changes to `dfbhd.exe`, game binaries, or PFF formats beyond existing `localres.pff` swap.
- `resource.pff` DDS HD pipeline work (deferred).
- Network/download features or redistributing original game assets.

## Approaches considered
- **(A) Combined profile/action (chosen):** e.g. "HD Widescreen (1080p)" applies HD textures + installs/configures wrapper (1 click). Pro: simple "HD mode"; con: couples texture+wrapper.
- **(B) Separate toggle/action:** install/remove wrapper independently of texture profiles. Pro: flexible; con: more steps/state.
- **(C) Settings-driven config:** manage wrapper install and resolution/FOV in UI. Pro: control; con: higher complexity now.

**Decision:** A, with texture-only profile retained for flexibility/backward compatibility.

## Design (approved)

### Profiles (DFBHD)
- Keep `original` (id/name as appropriate, "Original") and `hd_textures` ("HD Textures", texture-only behavior preserved).
- Add `hd_widescreen_1080p` ("HD Widescreen (1080p)") as combined action.
- `active_profile_id` remains persisted in config (serde defaults/backward compat preserved; unknown ids handled safely as today).

### Bundled wrapper assets
- Source: `assets/wrapper/dfbhd_widescreen/` (resolved relative to executable/cwd, similar to existing HD pack resolution strategy as appropriate).
- Default "HD" config in bundled `scripts/BHD_Asi.ini`: `ResolutionX = 1920`, `ResolutionY = 1080`, `FOV = 90` (adjustable by user in installed copy later).
- Files installed to game dir (minimum set): `d3d8.dll`, `d3d8.ini`, `dinput8.dll`, and `scripts/` contents required for ASI loading/config (at least `BHD_Asi.asi`, `BHD_Asi.ini`; may include companion files shipped in wrapper). Do not install unnecessary build artifacts if avoidable, but correctness/safety takes precedence.

### Apply behaviors
- **`HD Textures` (textures only):** existing behavior — swap/restore `localres.pff` HD pack only; does not modify wrapper files.
- **`HD Widescreen (1080p)` (combined):**
  1. Apply HD textures via existing safe `localres.pff` logic (create/preserve `localres.pff.original` as today; do not overwrite it).
  2. Install/configure wrapper into linked install: copy bundled wrapper files to install dir. Before overwriting any file that already exists in the install, create a one-time backup `<file>.launcherbak` (only if `.launcherbak` does not already exist) so a pre-existing user file can be restored. Create `scripts/` as needed.
  3. Log each significant action (backups created, files copied, skips/failures).
- **`Original`:**
  1. Restore original `localres.pff` (from `localres.pff.original` or `.bak` fallback, as today).
  2. Remove wrapper files installed/managed by this integration from the install. For each managed path: if `<file>.launcherbak` exists, restore it to `<file>` and remove the bak; if no bak exists and the file was installed by us, remove the file. Remove managed `scripts/` wrapper artifacts (and clean empty `scripts/` dir only if safe/empty as appropriate) without deleting unrelated user files.
  3. Log restore/removal results clearly.

### Safety and constraints
- Require linked DFBHD install (install dir known) for apply; guard and log like existing flows.
- Handle missing bundle/wrapper assets, missing targets, and IO/permission errors gracefully (no panics), with actionable log messages.
- Never modify `dfbhd.exe`, unrelated PFFs, or unrelated install contents. Preserve user-owned pre-existing files via `.launcherbak` restore on Original.
- Operations should be best-effort with clear status; partial failures must be reported in log.

### UI
- DFBHD profile selector shows `Original`, `HD Textures`, `HD Widescreen (1080p)`.
- `APPLY PROFILE` applies the selected active profile (including combined behavior for widescreen profile) and saves config as today.
- Existing scan/link/launch behavior unchanged.

### Tests (expected)
- Unit tests for wrapper file install/backup behavior, restore/removal behavior, and combined profile apply sequencing using temp dirs and explicit paths/helpers (analogous to existing `apply_dfbhd_profile_with_pack_path`-style testability).
- Config/profile selection backward-compat tests remain passing; add coverage for new profile id persistence/recognition as needed.
- Run `cargo test -p launcher-gui` and `cargo check -p launcher-gui`.

## What's left to the builder
- Exact helper decomposition/API names in `model.rs` (e.g. wrapper bundle resolution, managed file list, install/restore helpers) consistent with existing style.
- Exact set/order of wrapper files to treat as "managed" (derive from bundled `assets/wrapper/dfbhd_widescreen/` contents, excluding non-runtime docs if desired).
- README updates describing the new profile and wrapper behavior.
- Adjust bundled `scripts/BHD_Asi.ini` to 1920x1080/FOV 90 as specified.
