# DFBHD In-Game HD Textures (`resource.pff` DDS) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add in-game HD DDS textures from `resource.pff` to DFBHD HD profiles (Approach A): deliver a PoC (17 “mog” DDS) bundled in `assets/` with safe launcher apply/restore for `resource.pff`, then scale to all 573 DDS.

**Architecture:** Process target DDS from a clean original `resource.pff` (decode → Upscayl 4x → downscale to original WxH → recompress DXT5 with mipmaps via `texconv`) and repack replacements with `tools/pff3.ps1` into `assets/profiles/dfbhd/hd_textures/resource.pff`. Extend `crates/launcher-gui` to resolve/apply/restore `resource.pff` (backup `resource.pff.original`, never overwritten) alongside existing `localres.pff` handling for `hd_textures`/`hd_widescreen_1080p`, with `Original` restoring both (and wrapper cleanup) idempotently and without panics.

**Tech Stack:** Rust (`eframe/egui` launcher, `std::fs/path/process`), PowerShell `tools/pff3.ps1` (PFF3 repack), Upscayl CLI (`ultrasharp-4x`), DirectXTex `texconv.exe`, `ffmpeg.exe`, Python/Pillow for batch/image helpers as needed.

**Spec:** `docs/superpowers/specs/2026-11-19-dfbhd-ingame-resource-pff-hd-textures-design.md`

## Global Constraints

- **BYOG / no redistribution:** do not commit or redistribute original game assets/binaries; only intended HD outputs/docs/source/assets explicitly scoped by the spec.
- **Preserve compatibility:** keep original DDS dimensions (WxH), format/compression (**DXT5/BC3**), and compatible mipmaps/orientation/alpha for all replacements. Always rebuild from a clean original `resource.pff` base (never compound on modified packs).
- **Safe apply/restore:** require linked install dir; never overwrite `*.original` backups; handle missing packs/targets and IO/permission errors (e.g. `Program Files`) gracefully with actionable log messages; **no panics**. Never modify `dfbhd.exe` or unrelated install files.
- **Profiles/config:** keep profile ids `original`, `hd_textures`, `hd_widescreen_1080p` (backward compatible); Approach A integrates resource textures into existing HD profiles.
- **Bundling:** HD in-game pack path is `assets/profiles/dfbhd/hd_textures/resource.pff` (resolved relative to exe or cwd, same bases as localres). User-approved large asset (~344 MB).
- **Commit hygiene:** do not commit untracked `tools/`, `work/`, zips, logs, or other incidental artifacts; commit only intended changes for each task.
- **Verification baseline:** `cargo test -p launcher-gui` and `cargo check -p launcher-gui` must pass after launcher changes.

## Review Focus

- **Missing or partial backups/originals:** applying HD or restoring `Original` must never destroy the only original copy of `localres.pff` or `resource.pff` (including when one asset was already applied/backed up and the other fails, or on permission errors).
- **Idempotent/partial `Original` restore:** `Original` must safely restore whichever of localres/resource backups exist and still perform wrapper cleanup; if no resource backup exists (resource HD never applied), it must log clearly and not fail the whole restore.
- **DDS pipeline correctness:** processed DDS must keep original WxH, DXT5, alpha, and non-flipped orientation with valid mipmaps; a sample must be validated in-game at **MARKA/Mogadíscio** before accepting PoC/full batches.
- **Resource pack presence/absence:** HD profiles must apply available assets with clear per-asset logs; missing resource pack must not break existing localres behavior and must be reported (not silent).
- **Accidental commits of originals/untracked artifacts:** only the intended bundled HD `resource.pff` and source/docs changes should be staged; never stage `work/`, `tools/`, original game files, or temp outputs.

---

### Task 1: PoC — validate DDS pipeline and build bundled `resource.pff` (17 “mog” DDS)

**Files:**
- Create (working, untracked): `work/resource_poc/` (decoded/enhanced/replacements subdirs), helper script(s) as needed (e.g. `work/resource_poc/process_mog.py`)
- Create/Modify (bundled asset): `assets/profiles/dfbhd/hd_textures/resource.pff`
- Source/base (read-only): `C:\Program Files (x86)\NovaLogic\Delta Force Black Hawk Down\resource.pff`, `work/extracted/resource_pff/*.dds`, `tools/pff3.ps1`

**Interfaces:**
- Consumes: clean original `resource.pff`; extracted originals for `Dmoght3.dds, dmoght3d.dds, GMogFil1.dds, GMogFil2.dds, GMoghut1.dds, GMoghut2.dds, GMogShd1.dds, GMogShd2.dds, GMogShd3.dds, GMogSlm1.dds, GMogSlm2.dds, Lmogmap.dds, Qmog01.dds, Qmog02.dds, Qmog03.dds, Qmog04.dds, Qmog05.dds`.
- Produces: `assets/profiles/dfbhd/hd_textures/resource.pff` (PFF3, same entry set as original) with only those 17 DDS replaced; replacements preserve original WxH, DXT5/BC3, mip levels, alpha, and orientation.

- [ ] **Step 1: Prepare PoC workspace and confirm originals**

Run:
```powershell
New-Item -ItemType Directory -Force -Path work/resource_poc/decoded,work/resource_poc/enhanced,work/resource_poc/replacements | Out-Null
$mog = @('Dmoght3.dds','dmoght3d.dds','GMogFil1.dds','GMogFil2.dds','GMoghut1.dds','GMoghut2.dds','GMogShd1.dds','GMogShd2.dds','GMogShd3.dds','GMogSlm1.dds','GMogSlm2.dds','Lmogmap.dds','Qmog01.dds','Qmog02.dds','Qmog03.dds','Qmog04.dds','Qmog05.dds')
$mog | ForEach-Object { if (!(Test-Path "work/extracted/resource_pff/$_")) { throw "missing $_" } }
$mog.Count
```
Expected: prints `17` with no errors.

- [ ] **Step 2: Validate pipeline on one sample (orientation/alpha/mipmaps)**

Pick `Lmogmap.dds` (512x512, 10 mips, DXT5). Decode → upscale 4x (`ultrasharp-4x`) → downscale to 512x512 → recompress DXT5 with 10 mips into `work/resource_poc/replacements/Lmogmap.dds`. Verify header:
Run:
```powershell
python -c "import struct; b=open(r'work/resource_poc/replacements/Lmogmap.dds','rb').read(128); h,w=struct.unpack_from('<II',b,12); m=struct.unpack_from('<I',b,28)[0]; f=struct.unpack_from('<I',b,84)[0].to_bytes(4,'little'); print(w,h,m,f)"
```
Expected: `512 512 10 b'DXT5'` (or equivalent). Visually confirm orientation/alpha are correct (compare decoded original vs processed; fix flip if needed before batch).

- [ ] **Step 3: Process all 17 “mog” DDS and repack PoC `resource.pff`**

Process each of the 17 to `work/resource_poc/replacements/<name>.dds` preserving per-file original WxH and mip count (from DDS header of the original). Repack from the clean original base:
Run:
```powershell
Copy-Item "C:\Program Files (x86)\NovaLogic\Delta Force Black Hawk Down\resource.pff" "work/resource_poc/resource_original_base.pff" -Force
powershell -NoProfile -ExecutionPolicy Bypass -File tools\pff3.ps1 repack "work/resource_poc/resource_original_base.pff" "work/resource_poc/resource_hd_mog_poc.pff" -ReplacementsDir "work/resource_poc/replacements"
Copy-Item "work/resource_poc/resource_hd_mog_poc.pff" "assets/profiles/dfbhd/hd_textures/resource.pff" -Force
(Get-Item "assets/profiles/dfbhd/hd_textures/resource.pff").Length
```
Expected: repack reports `replaced=17`; bundled file exists with non-trivial size (~original magnitude). Spot-check a few outputs keep original dimensions/mips/DXT5.

- [ ] **Step 4: Commit PoC bundled asset (intended only)**

```bash
git add assets/profiles/dfbhd/hd_textures/resource.pff
git commit -m "feat: add DFBHD HD resource.pff PoC (17 mog DDS)"
```
(If `tools/pff3.ps1` must be tracked for reproducibility, add it intentionally in a separate commit; do not add `work/` artifacts.)

---

### Task 2: Launcher — resolve HD `resource.pff` pack path

**Files:**
- Modify: `crates/launcher-gui/src/model.rs`
- Test: `crates/launcher-gui/src/model.rs` (inline `#[cfg(test)]`)

**Interfaces:**
- Produces: `pub fn resolve_dfbhd_hd_resource_pack_path() -> Option<std::path::PathBuf>` and `pub(crate) fn resolve_dfbhd_hd_resource_pack_path_from_bases<I: IntoIterator<Item = std::path::PathBuf>>(bases: I) -> Option<std::path::PathBuf>` resolving `assets/profiles/dfbhd/hd_textures/resource.pff` (parallel to localres/wrapper resolution).

- [ ] **Step 1: Write the failing test**

Add in `model.rs` tests:
```rust
#[test]
fn resolve_dfbhd_resource_pack_finds_assets_relative_to_base() {
    let dir = unique_temp_dir("resource-pack");
    let pack = dir.join("assets/profiles/dfbhd/hd_textures/resource.pff");
    write_file(&pack, b"HD-RESOURCE");
    assert_eq!(
        resolve_dfbhd_hd_resource_pack_path_from_bases([dir.clone()]),
        Some(pack.clone())
    );
    let empty = unique_temp_dir("resource-pack-empty");
    assert_eq!(resolve_dfbhd_hd_resource_pack_path_from_bases([empty.clone()]), None);
    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&empty);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p launcher-gui resolve_dfbhd_resource_pack_finds_assets_relative_to_base -v`  
Expected: FAIL (function not defined).

- [ ] **Step 3: Implement resolution helpers in `model.rs`**

Implement `resolve_dfbhd_hd_resource_pack_path_from_bases` to probe `base.join("assets/profiles/dfbhd/hd_textures/resource.pff").is_file()` (same base order strategy as `resolve_dfbhd_hd_localres_pack_path`: exe dir then cwd). Implement `resolve_dfbhd_hd_resource_pack_path()` delegating to it.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p launcher-gui resolve_dfbhd_resource_pack_finds_assets_relative_to_base -v`  
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/src/model.rs
git commit -m "feat: resolve DFBHD HD resource.pff pack path"
```

---

### Task 3: Launcher — apply/restore `resource.pff` with full-path APIs and dispatch

**Files:**
- Modify: `crates/launcher-gui/src/model.rs`
- Test: `crates/launcher-gui/src/model.rs` (inline `#[cfg(test)]`)

**Interfaces:**
- Consumes: Task 2 resolvers; existing localres/wrapper helpers.
- Produces:
  - `pub fn apply_dfbhd_profile_with_pack_paths(&mut self, profile_id: &str, install_dir: &str, hd_localres_pack_path: &str, hd_resource_pack_path: Option<&str>) -> bool` — core texture apply/restore for `original`/`hd_textures` managing both `localres.pff` and (when applicable) `resource.pff` with `*.original` backups (one-time, never overwritten), `.bak` fallbacks where safe/consistent, clear per-asset logs, and wrapper cleanup on `original` as today.
  - `pub fn apply_dfbhd_profile_with_pack_path(&mut self, profile_id: &str, install_dir: &str, hd_pack_path: &str) -> bool` remains as compat wrapper calling `with_pack_paths(..., hd_pack_path, None)`.
  - `pub fn apply_dfbhd_hd_widescreen_with_all_paths(&mut self, install_dir: &str, hd_localres_pack_path: &str, hd_resource_pack_path: Option<&str>, wrapper_bundle_dir: &str)` — textures (localres+resource via `with_pack_paths`) then wrapper install, with per-step logging (do not claim overall success if a required step failed).
  - `pub fn apply_dfbhd_hd_widescreen_with_paths(&mut self, install_dir: &str, hd_pack_path: &str, wrapper_bundle_dir: &str)` remains as compat wrapper calling `with_all_paths(..., hd_pack_path, None, wrapper_bundle_dir)`.
  - `apply_active_profile_for_game("dfbhd")` resolves localres and resource packs and dispatches: `original` restores textures (both) + wrapper cleanup; `hd_textures` applies available HD packs (localres required if pack resolved as today; resource applied when pack resolved, logged if missing/unappliable); `hd_widescreen_1080p` applies available texture packs + wrapper with clear status when packs missing.

**Behavioral details to encode:** `Original` restores `resource.pff` from `resource.pff.original`/`resource.pff.bak` when present; if no resource backup exists it must log and continue (idempotent, not a hard failure of the whole original restore). `hd_textures`/`hd_widescreen` must never overwrite an existing `resource.pff.original`; if a resource swap is requested but cannot be performed safely (e.g. no source to back up, IO error), log failure and reflect it in result/logs.

- [ ] **Step 1: Write failing tests for resource swap/restore and compatibility wrappers**

Add tests (temp dirs, explicit paths):
```rust
#[test]
fn apply_hd_with_resource_creates_resource_backup_and_swaps() {
    let install = unique_temp_dir("apply-hd-res");
    write_file(&install.join("localres.pff"), b"ORIG-L");
    write_file(&install.join("resource.pff"), b"ORIG-R");
    let packs = unique_temp_dir("packs-hd-res");
    let lpack = packs.join("localres.pff"); write_file(&lpack, b"HD-L");
    let rpack = packs.join("resource.pff"); write_file(&rpack, b"HD-R");

    let mut s = LauncherState::default();
    let ok = s.apply_dfbhd_profile_with_pack_paths(
        "hd_textures", &install.to_string_lossy(), &lpack.to_string_lossy(), Some(&rpack.to_string_lossy()),
    );
    assert!(ok);
    assert_eq!(fs::read(install.join("localres.pff")).unwrap(), b"HD-L");
    assert_eq!(fs::read(install.join("resource.pff")).unwrap(), b"HD-R");
    assert_eq!(fs::read(install.join("localres.pff.original")).unwrap(), b"ORIG-L");
    assert_eq!(fs::read(install.join("resource.pff.original")).unwrap(), b"ORIG-R");
    let _ = fs::remove_dir_all(&install); let _ = fs::remove_dir_all(&packs);
}

#[test]
fn apply_original_restores_resource_and_is_idempotent_without_resource_backup() {
    let install = unique_temp_dir("apply-orig-res");
    write_file(&install.join("localres.pff"), b"HD-L");
    write_file(&install.join("localres.pff.original"), b"ORIG-L");
    write_file(&install.join("resource.pff"), b"ORIG-R"); // no resource backup

    let mut s = LauncherState::default();
    let ok = s.apply_dfbhd_profile_with_pack_paths("original", &install.to_string_lossy(), "", None);
    assert!(ok);
    assert_eq!(fs::read(install.join("localres.pff")).unwrap(), b"ORIG-L");
    assert_eq!(fs::read(install.join("resource.pff")).unwrap(), b"ORIG-R");
    let log_text = s.games.iter().find(|g| g.id=="dfbhd").unwrap().log.join("\n").to_lowercase();
    assert!(log_text.contains("resource") || log_text.contains("no") || log_text.contains("backup"));
    // idempotent
    let ok2 = s.apply_dfbhd_profile_with_pack_paths("original", &install.to_string_lossy(), "", None);
    assert!(ok2);
    let _ = fs::remove_dir_all(&install);
}

#[test]
fn compat_wrappers_behave_as_before_without_resource_pack() {
    let install = unique_temp_dir("compat-no-res");
    write_file(&install.join("localres.pff"), b"ORIG-L");
    let packs = unique_temp_dir("compat-packs");
    let lpack = packs.join("localres.pff"); write_file(&lpack, b"HD-L");

    let mut s = LauncherState::default();
    assert!(s.apply_dfbhd_profile_with_pack_path("hd_textures", &install.to_string_lossy(), &lpack.to_string_lossy()));
    assert_eq!(fs::read(install.join("localres.pff")).unwrap(), b"HD-L");
    assert!(!install.join("resource.pff.original").exists());
    let _ = fs::remove_dir_all(&install); let _ = fs::remove_dir_all(&packs);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p launcher-gui apply_hd_with_resource_creates_resource_backup_and_swaps apply_original_restores_resource_and_is_idempotent_without_resource_backup compat_wrappers_behave_as_before_without_resource_pack -v`  
Expected: FAIL (missing methods/behavior).

- [ ] **Step 3: Implement resource-aware helpers and full-path APIs; update dispatch**

Refactor `model.rs`:
- Add internal logic/helpers to back up and swap/restore a single PFF target (`localres.pff` / `resource.pff`) mirroring existing safe semantics (`*.original` created once from existing target or `.bak` fallback; restore from `.original` or `.bak`; log actions/errors; never clobber original backup).
- Implement `apply_dfbhd_profile_with_pack_paths(...)`: for `original` restore localres (as today), restore resource if backup/fallback exists else log-and-continue, run `restore_dfbhd_wrapper_in_install` (as today) and report combined status clearly; for `hd_textures` apply localres from `hd_localres_pack_path` (required) and apply resource when `hd_resource_pack_path` is `Some` (if missing/unreadable/unappliable log failure and reflect in result); for unknown profiles log failure/false.
- Reimplement `apply_dfbhd_profile_with_pack_path` as wrapper with `hd_resource_pack_path: None` (preserve behavior).
- Implement `apply_dfbhd_hd_widescreen_with_all_paths(..., hd_resource_pack_path: Option<&str>, ...)` to call `apply_dfbhd_profile_with_pack_paths("hd_textures", ..., hd_resource_pack_path)` then install wrapper and log aggregate status; keep `apply_dfbhd_hd_widescreen_with_paths` as wrapper with `None`.
- Update `apply_active_profile_for_game`: resolve `resolve_dfbhd_hd_resource_pack_path()` alongside localres/wrapper and pass it into the full-path methods (apply resource when resolved for HD profiles; `Original` always attempts resource restore via backup logic). Adjust log messages so missing resource pack is reported but does not masquerade as localres failure.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p launcher-gui apply_hd_with_resource_creates_resource_backup_and_swaps apply_original_restores_resource_and_is_idempotent_without_resource_backup compat_wrappers_behave_as_before_without_resource_pack -v`  
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/src/model.rs
git commit -m "feat: apply and restore DFBHD resource.pff HD textures"
```

---

### Task 4: Launcher — regression and integration tests for profiles/wrapper/resource

**Files:**
- Modify: `crates/launcher-gui/src/model.rs` (tests)

**Interfaces:**
- Consumes: Task 3 APIs. Ensures existing wrapper/localres sequencing remains correct and resource integrates per Approach A.

- [ ] **Step 1: Add/adjust tests covering combined sequencing and safety**

Add tests:
```rust
#[test]
fn apply_hd_widescreen_with_resource_applies_textures_and_wrapper() {
    let install = unique_temp_dir("ws-all");
    write_file(&install.join("localres.pff"), b"ORIG-L");
    write_file(&install.join("resource.pff"), b"ORIG-R");
    let packs = unique_temp_dir("ws-packs");
    let lpack = packs.join("localres.pff"); write_file(&lpack, b"HD-L");
    let rpack = packs.join("resource.pff"); write_file(&rpack, b"HD-R");
    let bundle = unique_temp_dir("ws-bundle");
    for rel in DFBHD_WRAPPER_MANAGED_FILES { write_file(&bundle.join(rel), b"BHD-WRAPPER"); }

    let mut s = LauncherState::default();
    s.apply_dfbhd_hd_widescreen_with_all_paths(
        &install.to_string_lossy(), &lpack.to_string_lossy(), Some(&rpack.to_string_lossy()), &bundle.to_string_lossy(),
    );
    assert_eq!(fs::read(install.join("localres.pff")).unwrap(), b"HD-L");
    assert_eq!(fs::read(install.join("resource.pff")).unwrap(), b"HD-R");
    assert_eq!(fs::read(install.join("localres.pff.original")).unwrap(), b"ORIG-L");
    assert_eq!(fs::read(install.join("resource.pff.original")).unwrap(), b"ORIG-R");
    for rel in DFBHD_WRAPPER_MANAGED_FILES { assert_eq!(fs::read(install.join(rel)).unwrap(), b"BHD-WRAPPER"); }
    let _ = fs::remove_dir_all(&install); let _ = fs::remove_dir_all(&packs); let _ = fs::remove_dir_all(&bundle);
}

#[test]
fn apply_original_after_hd_widescreen_restores_localres_and_resource_and_cleans_wrapper() {
    let install = unique_temp_dir("cycle-all");
    write_file(&install.join("localres.pff"), b"ORIG-L");
    write_file(&install.join("resource.pff"), b"ORIG-R");
    let packs = unique_temp_dir("cycle-packs");
    let lpack = packs.join("localres.pff"); write_file(&lpack, b"HD-L");
    let rpack = packs.join("resource.pff"); write_file(&rpack, b"HD-R");
    let bundle = unique_temp_dir("cycle-bundle");
    for rel in DFBHD_WRAPPER_MANAGED_FILES { write_file(&bundle.join(rel), b"BHD-WRAPPER"); }

    let mut s = LauncherState::default();
    s.apply_dfbhd_hd_widescreen_with_all_paths(&install.to_string_lossy(), &lpack.to_string_lossy(), Some(&rpack.to_string_lossy()), &bundle.to_string_lossy());
    s.apply_dfbhd_profile_with_pack_paths("original", &install.to_string_lossy(), "", None);

    assert_eq!(fs::read(install.join("localres.pff")).unwrap(), b"ORIG-L");
    assert_eq!(fs::read(install.join("resource.pff")).unwrap(), b"ORIG-R");
    for rel in DFBHD_WRAPPER_MANAGED_FILES { assert!(!install.join(rel).exists()); }
    let _ = fs::remove_dir_all(&install); let _ = fs::remove_dir_all(&packs); let _ = fs::remove_dir_all(&bundle);
}

#[test]
fn hd_textures_does_not_touch_wrapper_when_resource_applied() {
    let install = unique_temp_dir("hd-res-nowrap");
    write_file(&install.join("localres.pff"), b"ORIG-L");
    write_file(&install.join("resource.pff"), b"ORIG-R");
    write_file(&install.join("d3d8.dll"), b"USER-D3D8");
    let packs = unique_temp_dir("hd-res-packs");
    let lpack = packs.join("localres.pff"); write_file(&lpack, b"HD-L");
    let rpack = packs.join("resource.pff"); write_file(&rpack, b"HD-R");
    let mut s = LauncherState::default();
    s.apply_dfbhd_profile_with_pack_paths("hd_textures", &install.to_string_lossy(), &lpack.to_string_lossy(), Some(&rpack.to_string_lossy()));
    assert_eq!(fs::read(install.join("localres.pff")).unwrap(), b"HD-L");
    assert_eq!(fs::read(install.join("resource.pff")).unwrap(), b"HD-R");
    assert_eq!(fs::read(install.join("d3d8.dll")).unwrap(), b"USER-D3D8");
    let _ = fs::remove_dir_all(&install); let _ = fs::remove_dir_all(&packs);
}
```
Also verify existing wrapper/localres tests still compile and pass against compat wrappers (adjust only if signatures changed in a breaking way).

- [ ] **Step 2: Run targeted tests to verify they pass (and fail first if written before implementation is complete)**

Run: `cargo test -p launcher-gui apply_hd_widescreen_with_resource_applies_textures_and_wrapper apply_original_after_hd_widescreen_restores_localres_and_resource_and_cleans_wrapper hd_textures_does_not_touch_wrapper_when_resource_applied -v`  
Expected: PASS.

- [ ] **Step 3: Run full crate verification**

Run: `cargo test -p launcher-gui` and `cargo check -p launcher-gui`  
Expected: all tests pass; check succeeds.

- [ ] **Step 4: Commit**

```bash
git add crates/launcher-gui/src/model.rs
git commit -m "test: cover DFBHD resource.pff profile apply and restore"
```

---

### Task 5: Docs/README, in-game validation notes, and Pass 1 wrap-up

**Files:**
- Modify: `README.md`
- Modify (optional, if clarifications needed): `docs/superpowers/specs/2026-11-19-dfbhd-ingame-resource-pff-hd-textures-design.md`
- Bundled asset (from Task 1): `assets/profiles/dfbhd/hd_textures/resource.pff`

**Interfaces:**
- Produces: README accurately documents that HD profiles also apply/restore `resource.pff` in-game textures (PoC “mog” set in pass 1, full in pass 2) with `resource.pff.original` backup, alongside existing `localres.pff` and wrapper behavior; documents validation starting point (MARKA/Mogadíscio).

- [ ] **Step 1: Update README scope and behavior**

Document: `assets/profiles/dfbhd/hd_textures/resource.pff` bundled pack; `hd_textures`/`hd_widescreen_1080p` apply `localres.pff` and `resource.pff` (when available) with backups `localres.pff.original`/`resource.pff.original`; `Original` restores both and cleans wrapper; PoC currently covers 17 “mog” DDS (in-game), pass 2 will expand to all 573; validation via Single Player → MARKA and Mogadíscio missions.

- [ ] **Step 2: Run verification and confirm intended diff**

Run: `cargo test -p launcher-gui && cargo check -p launcher-gui`  
Run: `git status --short` and review staged/unstaged files to ensure no `work/`, `tools/`, zips, or originals are included.

- [ ] **Step 3: Commit docs (and any intended remaining pass-1 files)**

```bash
git add README.md
git commit -m "docs: document DFBHD in-game resource.pff HD textures"
```
(Commit the spec/plan files only if intentionally tracked in this repo state.)

- [ ] **Step 4: Manual in-game validation (PoC)**

With DFBHD install linked: apply `HD Textures` or `HD Widescreen (1080p)` (elevated if needed for `Program Files`), launch Single Player → **MARKA** and compare against `Original` in the same scene/angles (outdoor Mogadíscio areas for “mog” textures). Confirm no obvious flips/missing-alpha artifacts and that restore to `Original` returns prior state. Record results in a short note/log if desired.

---

### Task 6: Pass 2 — scale to all 573 DDS and update bundled `resource.pff`

**Files:**
- Create (working, untracked): `work/resource_full/` (decoded/enhanced/replacements), batch helper script(s) as needed
- Modify (bundled asset): `assets/profiles/dfbhd/hd_textures/resource.pff` (replace PoC with full set)
- Source/base (read-only): clean original `resource.pff`, `work/extracted/resource_pff/*.dds`

**Interfaces:**
- Consumes: validated pipeline from Task 1; original 573 DDS in `work/extracted/resource_pff/` (all DXT5 with mipmaps).
- Produces: `assets/profiles/dfbhd/hd_textures/resource.pff` rebuilt from clean original base with **all 573 DDS** replaced, each preserving original WxH, DXT5, mip count, alpha, and orientation.

- [ ] **Step 1: Enumerate all DDS and prepare full workspace from clean base**

Run:
```powershell
New-Item -ItemType Directory -Force -Path work/resource_full/decoded,work/resource_full/enhanced,work/resource_full/replacements | Out-Null
Copy-Item "C:\Program Files (x86)\NovaLogic\Delta Force Black Hawk Down\resource.pff" "work/resource_full/resource_original_base.pff" -Force
(Get-ChildItem "work/extracted/resource_pff" -Filter *.dds).Count
```
Expected: `573`.

- [ ] **Step 2: Batch-process all 573 DDS with per-file WxH/mip preservation**

For each DDS: decode → upscale 4x `ultrasharp-4x` → downscale to original WxH → recompress DXT5 with original mip count → write to `work/resource_full/replacements/<name>.dds`. Verify a sample across size classes (e.g. 64x64, 256x128, 512x512) retain correct dimensions/format/mips/orientation.

- [ ] **Step 3: Repack full `resource.pff` and update bundle**

Run:
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\pff3.ps1 repack "work/resource_full/resource_original_base.pff" "work/resource_full/resource_hd_full.pff" -ReplacementsDir "work/resource_full/replacements"
Copy-Item "work/resource_full/resource_hd_full.pff" "assets/profiles/dfbhd/hd_textures/resource.pff" -Force
```
Expected: repack reports `replaced=573`.

- [ ] **Step 4: Verify, validate in-game, and commit**

Run: `cargo test -p launcher-gui && cargo check -p launcher-gui` (regression). Spot-check in-game (MARKA/Mogadíscio and other areas) for regressions/artifacts; compare vs Original. Commit intended bundle/docs updates:
```bash
git add assets/profiles/dfbhd/hd_textures/resource.pff README.md
git commit -m "feat: expand DFBHD HD resource.pff to all 573 DDS"
```
(Do not commit `work/` or untracked tools/artifacts.)

## Self-Review

**1. Spec coverage:** Approach A integration (Task 3), PoC 17 mog + full 573 pipeline/assets (Tasks 1,6), `assets/profiles/dfbhd/hd_textures/resource.pff` bundling (Tasks 1,6), safe backup/restore and logging (Task 3,4), tests and verification (Tasks 2,4,5), README/docs (Task 5), MARKA/Mogadíscio validation (Tasks 1,5,6). **OK.**

**2. Step scan:** Code steps specify exact signatures/files; pipeline steps give concrete commands and expected outputs; verification steps specify commands and pass criteria. No "TBD"/"handle edge cases" placeholders. **OK.**

**3. Type consistency:** `apply_dfbhd_profile_with_pack_paths(..., hd_resource_pack_path: Option<&str>) -> bool`, `apply_dfbhd_hd_widescreen_with_all_paths(..., hd_resource_pack_path: Option<&str>, ...)`, compat wrappers retain existing signatures, resolvers `resolve_dfbhd_hd_resource_pack_path(_from_bases)` mirror existing patterns. Consistent across tasks. **OK.**

**4. Review Focus:** Each focus item mapped to owning tasks/tests/steps (backup/original safety, idempotent Original, DDS correctness/sample validation, missing resource pack behavior, commit hygiene). **OK.**

**5. Proportion:** Mix of exact code/test blocks for launcher changes and command-level steps for asset pipeline; no full function bodies beyond signatures/tests where behavior is defined. **OK.**
