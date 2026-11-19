# DFBHD HD Textures: localres Menu Batch A Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce an updated prebuilt HD `localres.pff` that includes HD versions of all remaining 12 `BHD_*` menu backgrounds and all 20 `Btn_*` button textures (keeping original dimensions/formats/orientation), update the bundled launcher asset, and verify repack correctness.

**Architecture:** Batch asset pipeline in workspace: extract originals from original `localres.pff` → convert to PNG → upscale 4x with Upscayl CLI (`ultrasharp-4x`) → downscale to original dimensions and encode as TGA preserving original bpp/descriptor/orientation intent → repack into HD `localres.pff` using `tools/pff3.ps1` → update bundled `assets/profiles/dfbhd/hd_textures/localres.pff`. No Rust code changes required for this batch.

**Tech Stack:** PowerShell, `tools/pff3.ps1`, ffmpeg (`C:\Users\itsal\AppData\Local\Microsoft\WinGet\Links\ffmpeg.exe`), Upscayl CLI (`C:\Program Files\Upscayl\resources\bin\upscayl-bin.exe`, models `C:\Program Files\Upscayl\resources\models`).

**Spec:** `docs/superpowers/specs/2026-11-19-dfbhd-hd-textures-localres-menu-batch-a-design.md` (approved 2026-11-19). The spec travels with this plan; executors read both.

## Global Constraints

- Scope A only: localres menu/UI textures — all remaining 12 `BHD_*.tga` and all 20 `Btn_*.tga`. No other localres assets, no `resource.pff` DDS work (B deferred).
- Preserve original dimensions for every texture (upscale for detail, then downscale back) to avoid in-game layout/cropping.
- Preserve original format characteristics: TGA bpp and image descriptor/orientation must match original so in-game rendering is not flipped/cropped.
- Keep existing `BHD_main.tga` HD indicator/orientation fix and existing `Btn_exit.tga` HD replacement in the pack.
- Base repack on original `localres.pff` (`C:\Program Files (x86)\NovaLogic\Delta Force Black Hawk Down\localres.pff` is currently original in this environment; verify before use). Do not destructively modify install files except user-driven launcher APPLY later.
- Update outputs: `work/poc/output/localres_hd_poc_fixed.pff` and `assets/profiles/dfbhd/hd_textures/localres.pff`.

## Review Focus

1. **Orientation/descriptor regressions:** any re-encoded TGA with wrong descriptor/pixel order can render upside down in-game (seen before on `BHD_main`). Verify descriptor/bpp/dimensions against originals for processed textures.
2. **Dimension drift:** exporting at 4x size instead of original causes cropping/layout issues; every replacement must match original width/height.
3. **Incomplete replacement set:** repacked HD pack must include all targeted `BHD_*` and `Btn_*` replacements (not just a subset) while preserving untouched entries.
4. **Pipeline/tool failures:** a failed Upscayl/ffmpeg conversion must not silently leave a stale/wrong file in replacements.
5. **Base-pack correctness:** repack must use an original/clean base `localres.pff`; using an already-modified base can compound errors.

## Texture Sets

- **BHD (12 remaining):** `BHD_camp.tga`, `BHD_Epil.tga`, `BHD_Epl2.tga`, `BHD_Ldot.tga`, `BHD_Ldt2.tga`, `BHD_Lgam.tga`, `BHD_misn.tga`, `BHD_mult.tga`, `BHD_nova.tga`, `BHD_plyr.tga`, `BHD_sply.tga`, `BHD_ups2.tga` (all 800x600 in original; `BHD_main.tga` already done).
- **Btn (20):** `Btn_exit.tga` (already done), `Btn_ext2.tga`, `Btn_fram.tga`, `Btn_gmdm.tga`, `Btn_ign.tga`, `Btn_info.tga`, `Btn_link.tga`, `Btn_lnk2.tga`, `Btn_pre.tga`, `Btn_pre2.tga`, `Btn_tab1.tga`, `Btn_tab2.tga`, `Btn_tab3.tga`, `Btn_tab4.tga`, `Btn_tab5.tga`, `Btn_tab6.tga`, `Btn_tab7.tga`, `Btn_tab8.tga`, `Btn_tab9.tga`, `Btn_zila.tga`.

## Working Paths

- Originals extracted: `work/extracted/localres_pff/` (and/or re-extract specific files from base original PFF as needed)
- Pipeline workspace: `work/batch_a/png_src/`, `work/batch_a/upscaled/`, `work/poc/replacements_localres/`
- Repack output: `work/poc/output/localres_hd_poc_fixed.pff`
- Bundled asset: `assets/profiles/dfbhd/hd_textures/localres.pff`
- Base original PFF: `C:\Program Files (x86)\NovaLogic\Delta Force Black Hawk Down\localres.pff` (confirm original before repack)

---

### Task 1: Prepare batch workspace and capture original metadata

**Files:**
- Create/use: `work/batch_a/png_src/`, `work/batch_a/upscaled/`
- Modify/use: `work/poc/replacements_localres/` (accumulate replacements; keep existing `BHD_main.tga`, `Btn_exit.tga`)

- [ ] **Step 1: Create directories and confirm base PFF**

Run: `New-Item -ItemType Directory -Force -Path work\batch_a\png_src,work\batch_a\upscaled | Out-Null` and verify base original exists at install path with expected original size/state.
Expected: directories exist; base file identified.

- [ ] **Step 2: Record original dimensions/bpp/descriptor for targets**

For each target in BHD/Btn sets (excluding already-final `BHD_main.tga` special-cased indicator if kept as-is, and `Btn_exit.tga` already done), read original TGA header (width/height at bytes 12-15, bpp byte 16, descriptor byte 17) from `work/extracted/localres_pff/<name>` or extracted-from-base copy and save a manifest (e.g. `work/batch_a/manifest.csv` or json).
Expected: manifest lists all 31 textures to process with original w/h/bpp/desc.

- [ ] **Step 3: Checkpoint**

Confirm 31 targets identified (12 BHD remaining + 19 Btn remaining; `Btn_exit` kept).

### Task 2: Batch upscale and re-encode BHD backgrounds

**Files:**
- Read: originals from `work/extracted/localres_pff/BHD_*.tga` (except `BHD_main.tga` keep current HD-marked version)
- Write: `work/batch_a/png_src/*.png`, `work/batch_a/upscaled/*.png`, `work/poc/replacements_localres/BHD_*.tga`

- [ ] **Step 1: Convert 12 BHD originals to PNG**

For each of the 12 BHD targets: `ffmpeg -y -i <orig.tga> -frames:v 1 work\batch_a\png_src\<name>.png`.
Expected: 12 PNGs created.

- [ ] **Step 2: Upscale 12 BHD PNGs with Upscayl**

For each: `& "C:\Program Files\Upscayl\resources\bin\upscayl-bin.exe" -i work\batch_a\png_src\<name>.png -o work\batch_a\upscaled\<name>.png -m "C:\Program Files\Upscayl\resources\models" -n ultrasharp-4x -s 4 -f png`.
Expected: 12 upscaled PNGs (3200x2400 for 800x600 sources).

- [ ] **Step 3: Downscale to original size, encode TGA, restore original descriptor/bpp**

For each, using manifest values (w,h,bpp,desc): `ffmpeg -y -i work\batch_a\upscaled\<name>.png -vf "scale=<w>:<h>,vflip" -frames:v 1 -c:v targa -pix_fmt <bgr24|bgra> work\poc\replacements_localres\<name>.tga` (choose pix_fmt from bpp: 24->bgr24, 32->bgra), then set byte 17 to original descriptor.
Expected: 12 TGAs in replacements with original dimensions and descriptor/bpp matching originals.

- [ ] **Step 4: Spot-check**

Verify dimensions/descriptors for a sample (e.g. `BHD_camp.tga`, `BHD_sply.tga`) match originals.
Expected: no size/orientation metadata regressions.

### Task 3: Batch upscale and re-encode Btn textures

**Files:**
- Read: originals from `work/extracted/localres_pff/Btn_*.tga` (except `Btn_exit.tga` keep current HD version)
- Write: `work/batch_a/png_src/*.png`, `work/batch_a/upscaled/*.png`, `work/poc/replacements_localres/Btn_*.tga`

- [ ] **Step 1: Convert 19 Btn originals to PNG**

For each target Btn except `Btn_exit`: `ffmpeg -y -i <orig.tga> -frames:v 1 work\batch_a\png_src\<name>.png`.
Expected: 19 PNGs created.

- [ ] **Step 2: Upscale 19 Btn PNGs with Upscayl**

For each: run Upscayl CLI `-n ultrasharp-4x -s 4 -f png` to `work\batch_a\upscaled\<name>.png`.
Expected: 19 upscaled PNGs at 4x original dimensions.

- [ ] **Step 3: Downscale to original size, encode TGA, restore original descriptor/bpp**

For each using manifest (w,h,bpp,desc): scale back to original WxH, `vflip`, encode targa with correct pix_fmt, set descriptor byte 17 to original.
Expected: 19 TGAs in replacements with original dimensions and descriptor/bpp.

- [ ] **Step 4: Spot-check**

Verify a sample (e.g. 24bpp and 32bpp buttons if present, various sizes) match original dimensions/descriptor/bpp.
Expected: correct metadata preserved.

### Task 4: Repack, update bundled asset, and verify

**Files:**
- Modify: `work/poc/output/localres_hd_poc_fixed.pff`, `assets/profiles/dfbhd/hd_textures/localres.pff`
- Read: base original `localres.pff`, `work/poc/replacements_localres/`

- [ ] **Step 1: Repack HD localres**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File tools\pff3.ps1 repack "<base original localres.pff>" "work\poc\output\localres_hd_poc_fixed.pff" -ReplacementsDir "work\poc\replacements_localres"`.
Expected: repack succeeds; replaced count reflects all accumulated replacements (>= 33 including prior `BHD_main` marker and `Btn_exit`, plus 31 new = 33 total targeted).

- [ ] **Step 2: Update bundled launcher asset**

Copy `work/poc/output/localres_hd_poc_fixed.pff` to `assets/profiles/dfbhd/hd_textures/localres.pff`.
Expected: bundled asset updated.

- [ ] **Step 3: Verify pack entries and metadata**

Use `tools\pff3.ps1 list` on output pack for spot targets to confirm sizes changed/replaced as expected; confirm original-dimension/descriptor preservation for sampled TGAs in replacements.
Expected: targeted entries present with updated data sizes; no missing replacements.

- [ ] **Step 4: Report completion and in-game test instructions**

Summarize processed textures, output paths, and instruct user to APPLY HD Textures in launcher and test menus/buttons in-game. Note B (`resource.pff` DDS) as next phase.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-11-19-dfbhd-hd-textures-localres-menu-batch-a.md`. Please review the plan. Which execution approach would you prefer?

- **Native** - I implement every task myself in this session (batch the Upscayl/ffmpeg/repack steps), then verify outputs. Cheapest/fastest for this asset pipeline.
- **Subagent-driven** - Fresh subagent per task with reviews. More overhead for shell/batch asset work.

**For this plan I recommend Native, because it is a linear asset-processing batch (32 textures) using known local tools with straightforward verification, not interdependent code design tasks. Does the plan capture what you want, and which approach should we use?** Also: want me to commit the current HD pack update (with the “HD” menu indicator fix) before/with this batch?