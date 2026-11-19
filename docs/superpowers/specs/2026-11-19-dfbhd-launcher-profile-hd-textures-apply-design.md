# DFBHD Launcher: Profile HD Textures Apply (A, now)

**Date:** 2026-11-19  
**Status:** Approved (short design)  
**Scope:** Etapa 3a (profile application foundation) — prebuilt pack swap for DFBHD; B (user-supplied pack paths/config) deferred.

## Intent

Make the DFBHD `Original` / `HD Textures` profiles actually switch textures via **APPLY PROFILE**, using the corrected HD menu/UI pack (fixes prior upside-down/crop by preserving original dimensions and matching original TGA descriptors). Keep behavior safe, explicit, and reversible.

## Scope (now)

- Game: DFBHD only.
- Profiles: `Original`, `HD Textures`.
- Assets: prebuilt `localres.pff` menu textures (`BHD_main.tga` 800x600, `Btn_exit.tga` 64x128) only. No runtime upscaling, no DDS/other PFF processing, no other archives yet.

## Assets

- Bundled HD pack: `assets/profiles/dfbhd/hd_textures/localres.pff` (corrected, sourced from `work/poc/output/localres_hd_poc_fixed.pff`).
- Resolution: locate pack relative to app base (executable directory, with fallback to current working directory) so dev runs work.

## Apply behavior

Triggered explicitly by **APPLY PROFILE** (no auto-swap on profile selection).

**Preconditions:** game linked, `install_dir` known/resolvable, target archive `localres.pff` in install dir identifiable. Log clear, actionable errors (missing link/paths/pack, I/O or permission failures — common under Program Files). No panics.

**HD Textures:**
1. Ensure a recoverable original backup exists before overwriting:
   - If `<install>\localres.pff.original` exists, keep it (never overwrite).
   - Else determine original source: if `<install>\localres.pff` exists, treat as current/original source to back up to `.original` as appropriate; if `<install>\localres.pff` is missing but `<install>\localres.pff.bak` exists (current known state), use `.bak` as the original source for restore/backup continuity (do not lose `.bak`).
2. Copy bundled HD `localres.pff` to `<install>\localres.pff`.
3. Log success/failure with paths.

**Original:**
1. Restore `<install>\localres.pff` from `<install>\localres.pff.original` if present; else fall back to `<install>\localres.pff.bak` if present.
2. This repairs the current state where `localres.pff` is missing and only `.bak` exists.
3. Log success/failure with source used.

## Config / state

- Persist `active_profile_id` per game in `launcher-config.json` (serde defaults for backward compatibility with existing configs).
- Keep UI selection and APPLY action semantics clear; applying the selected profile performs the swap/restore for that profile.

## UI / UX

- `APPLY PROFILE` performs the real apply for the selected DFBHD profile (replacing stub behavior for this scope) and appends log entries.
- Show/log current linked/apply status clearly. No auto-apply on selection.

## Non-goals (now)

- User-supplied pack paths/folders (B, later — will reuse same apply/swap mechanism).
- ISO/InstallShield handling, PFF parsing/extraction/upscaling pipeline, DDS/`resource.pff`/`med.pff` HD packs, wrappers/patching, network/manifest.
- Automatic elevation; permission issues reported to user.

## Risks / constraints

- Writing to install directory may require elevated permissions; surface errors in log.
- Bundled pack must be present at resolved path; missing pack must not corrupt install.

## Verification

- `cargo test -p launcher-gui`
- `cargo check -p launcher-gui`
- Manual: from linked install with current `.bak`/missing `localres.pff`, APPLY `Original` restores playable original; APPLY `HD Textures` installs corrected pack (no upside-down/crop).