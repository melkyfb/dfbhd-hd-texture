# DFBHD In-Game HD Textures (`resource.pff` DDS) — Design (Approach A)

**Date:** 2026-11-19  
**Status:** Draft (awaiting user approval)  
**Scope:** `tools/` pipeline (PFF3/texconv/Upscayl), `assets/profiles/dfbhd/hd_textures/`, `crates/launcher-gui` (apply/restore), docs. Medium/large change delivered in two passes (PoC → full).

## Intent and why
Extend the existing DFBHD HD workflow beyond menu/UI (`localres.pff`) to **in-game textures** stored as DDS inside `resource.pff`, so HD profiles improve visible in-game appearance while preserving safe apply/restore and BYOG constraints.

## Decisions (approved by user)
- **Approach A:** integrate in-game HD textures into existing HD profiles (`hd_textures`, `hd_widescreen_1080p`). `Original` restores both menu and in-game textures (and cleans wrapper as today).
- **PoC subset confirmed:** start with the **17 “mog” DDS** (Mogadíscio environment/map textures) for in-game validation, **then** scale to the full set.
- **Bundling:** ship the HD `resource.pff` pack in **`assets/`** (repo size impact acknowledged: ~344 MB per bundled `resource.pff`).
- **Validation start point:** Single Player → **MARKA** (opening convoy/blindado, .50 cal turret) and other Mogadíscio urban missions (`SPBHD_*` load screens) for visible comparison.

## Current state (relevant)
- `C:\Program Files (x86)\NovaLogic\Delta Force Black Hawk Down\resource.pff`: **343,686,891 bytes**, **1,481 entries** including **573 `.dds`** (all sampled as **DXT5/BC3** with mipmaps, various POT/non-square sizes e.g. 256x256, 512x512, 256x128).
- Extracted DDS available in `work/extracted/resource_pff/`. PoC “mog” set (17): `Dmoght3.dds`, `dmoght3d.dds`, `GMogFil1.dds`, `GMogFil2.dds`, `GMoghut1.dds`, `GMoghut2.dds`, `GMogShd1.dds`, `GMogShd2.dds`, `GMogShd3.dds`, `GMogSlm1.dds`, `GMogSlm2.dds`, `Lmogmap.dds`, `Qmog01.dds`, `Qmog02.dds`, `Qmog03.dds`, `Qmog04.dds`, `Qmog05.dds`.
- Launcher today swaps/restores **`localres.pff`** only (`localres.pff.original` backup, `.bak` fallback) and manages wrapper for `hd_widescreen_1080p`; bundled localres pack: `assets/profiles/dfbhd/hd_textures/localres.pff`.
- Tools: `tools/pff3.ps1` (list/extract/repack), Upscayl CLI `...\upscayl-bin.exe` (`ultrasharp-4x`, models at `...\resources\models`), `texconv.exe`, `ffmpeg.exe`.

## Goals
- Produce HD in-game DDS replacements with a repeatable, format-safe pipeline and integrate apply/restore into existing HD profiles with clear logging and no panics on IO/permission errors.
- Deliver in two passes: **(1) PoC “mog” (17 DDS)** validated in-game, **(2) full (573 DDS)**.

## Non-goals / out of scope
- Changing texture dimensions (keep original WxH for compatibility) or altering non-DDS content in `resource.pff`.
- Modifying `dfbhd.exe` or game binaries, redistributing original game assets, or network/download features.
- True widescreen menu/UI layout changes or unrelated texture packs.

## Design

### 1) Bundled asset layout
- Add bundled HD in-game pack: **`assets/profiles/dfbhd/hd_textures/resource.pff`** (resolved relative to executable or current directory, same discovery bases/pattern as the existing localres HD pack).
- **PoC (pass 1):** `resource.pff` built from the original `resource.pff` with **only the 17 “mog” DDS** replaced.  
- **Full (pass 2):** replace/update the same bundled asset with a `resource.pff` containing **all 573 DDS** processed (same filename/path so launcher behavior is unchanged).

### 2) DDS processing pipeline (per texture)
For each target DDS (PoC 17, then 573), preserving original dimensions/format/mip characteristics:
1. **Source:** original bytes from a clean base copy of `resource.pff` (never from an already-modified pack) extracted via `tools/pff3.ps1`.
2. **Decode:** `texconv` to a lossless/intermediate image (e.g. PNG/TGA) preserving alpha.
3. **Enhance:** Upscayl CLI 4x (`ultrasharp-4x`) → **downscale back to original WxH** (detail enhancement at original resolution). Use `ffmpeg`/PIL for resize as needed; preserve alpha channel.
4. **Recompress:** `texconv` to **DXT5 (BC3)** with a compatible **mip chain** (target original mip count/levels) and correct orientation. Validate on at least one sample (round-trip visual/orientation/alpha, in-game) before batch.
5. **Repack:** copy clean original `resource.pff` → working output, then `tools/pff3.ps1 repack <base> <out> -ReplacementsDir <dir>` with processed DDS files named to match originals. PoC output becomes `assets/profiles/dfbhd/hd_textures/resource.pff` (pass 1).

Working artifacts live under `work/` (e.g. `work/resource_poc/` for PoC, separate dir for full) and are not committed except the final bundled asset(s) intended for `assets/`.

### 3) Launcher apply/restore (Approach A)

**Pack resolution**
- Keep `resolve_dfbhd_hd_localres_pack_path()` behavior.
- Add `resolve_dfbhd_hd_resource_pack_path()` (and a testable `*_from_bases` helper, consistent with existing patterns) resolving `assets/profiles/dfbhd/hd_textures/resource.pff`.

**Install targets and backups (in linked install dir)**
- `localres.pff` (existing): backup `localres.pff.original` (one-time, never overwritten), fallback sources as today.
- `resource.pff` (new): backup **`resource.pff.original`** (one-time, never overwritten). Use the existing installed `resource.pff` as the source to preserve the original; treat any `.bak` fallback consistently/safely if present. Never perform a destructive swap that would lose the only original copy.

**`Original` profile**
- Restore `localres.pff` from `localres.pff.original`/fallback (existing) **and** restore `resource.pff` from `resource.pff.original`/fallback when such a backup exists (idempotent: if no resource backup/HD was ever applied, log and continue without failing the whole restore). Wrapper cleanup remains as today and runs as best-effort per existing behavior.

**`hd_textures` profile**
- Apply HD `localres.pff` (existing behavior). If an HD `resource.pff` pack is resolved, apply it with the same backup/swap safety; if the resource pack is missing/unavailable, log clearly and still complete the localres apply (additive/backward-compatible, apply what is available and report status per asset).

**`hd_widescreen_1080p` profile**
- Combined as today (textures + wrapper): apply HD textures including `localres.pff` and, when available, `resource.pff`, then install/configure the wrapper. Report failures clearly per step; do not claim overall success if a required step failed.

**Safety and errors**
- Require a linked install (install dir known). Handle missing packs/targets and IO/permission errors (e.g. `Program Files`) gracefully with actionable log messages; no panics. Never modify `dfbhd.exe` or unrelated files. Best-effort sequencing with clear status.

### 4) UI / config
- No new profile ids required (Approach A). Profile list and persisted `active_profile_id` remain `original`, `hd_textures`, `hd_widescreen_1080p` (backward compatible). Logs should distinguish localres vs resource apply/restore outcomes.

### 5) Testing (expected)
- Unit tests (temp dirs, explicit paths/helpers): `resource.pff` HD apply creates `resource.pff.original` once and swaps; `Original` restores `resource.pff`; idempotent/no-backup cases logged safely; missing resource pack behavior as specified; `hd_widescreen_1080p` sequencing still applies wrapper and reports texture outcomes.
- Regression: existing localres/wrapper/config/profile tests remain passing.
- Verification: `cargo test -p launcher-gui`, `cargo check -p launcher-gui`.

### 6) Delivery sequence
1. **Pass 1 (PoC):** process 17 “mog” DDS → build bundled `assets/profiles/dfbhd/hd_textures/resource.pff` → implement launcher resource apply/restore + tests/docs → validate in-game (MARKA/Mogadíscio).
2. **Pass 2 (Full):** process remaining/all 573 DDS (rebuild from clean original base including PoC set) → update bundled `assets/profiles/dfbhd/hd_textures/resource.pff` → revalidate and run tests.

## Risks / notes
- **Orientation/alpha/compression:** the DDS pipeline must be validated on samples to avoid flipped or incorrect-alpha textures; always rebuild from the clean original base.
- **Repo size:** bundling `resource.pff` (~344 MB) in `assets/` significantly increases repo size/history weight (user-approved).
- **Perceived gain:** same-dimension enhancement (upscale→downscale) may yield subtle improvements depending on source; in-game validation at MARKA is required.
- **Commit scope:** `tools/`, `work/`, wrapper zip, and other untracked artifacts must not be committed unintentionally; commit only intended source/docs/assets changes.
