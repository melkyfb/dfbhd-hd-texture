# DFBHD HD Textures: localres Menu Batch A (Design)

**Date:** 2026-11-19  
**Status:** Approved (user: "sim, continuar")  
**Scope:** Expand HD menu/UI coverage in `localres.pff` (A). In-game `resource.pff` DDS textures deferred to B.

## Intent

Make DFBHD menus/buttons consistently HD by extending the existing prebuilt `localres.pff` HD pack (currently `BHD_main.tga` with "HD" indicator + `Btn_exit.tga`) to cover all main menu backgrounds and button textures.

## Scope (A)

- **In scope:** all remaining **12 `BHD_*.tga`** menu backgrounds (800x600) and all **20 `Btn_*.tga`** button textures in `localres.pff` (32 textures total), in addition to the 2 already included.
- **Out of scope (A):** other `localres.pff` assets (~319 other TGAs, 18 PCXs) and all `resource.pff` textures (DDS/TGA/PCX) — deferred to B.

## Approach

- Use the proven TGA pipeline: extract originals (from original install `localres.pff`) → convert to PNG (ffmpeg) → upscale 4x with Upscayl CLI (`ultrasharp-4x`, models at `C:\Program Files\Upscayl\resources\models`) → downscale to **original dimensions** and preserve original pixel format/orientation intent → encode back to TGA (bgr24/bgra as appropriate) with matching TGA image descriptor/orientation → repack into updated `localres` HD pack based on original `localres.pff`.
- Preserve original dimensions for all textures to avoid in-game layout/cropping issues.
- Rebuild `work/poc/output/localres_hd_poc_fixed.pff` and update bundled launcher asset `assets/profiles/dfbhd/hd_textures/localres.pff`.
- Keep `BHD_main.tga`'s existing "HD" indicator/orientation fix.

## Constraints / notes

- Processing is batch but may take time (Upscayl per image). Run in workspace; do not modify install except via launcher apply flow later.
- Verify representative outputs (sizes/descriptors) before/after repack; final validation is in-game via launcher APPLY PROFILE.
- Commit updated bundled pack (and any supporting pipeline/script changes if needed) as appropriate.

## Verification

- Repacked HD `localres.pff` contains replaced `BHD_*` and `Btn_*` entries with expected sizes; spot-check descriptors/dimensions match originals.
- Launcher applies updated pack successfully; menus/buttons render correctly in-game (no upside-down/crop regressions).