# DFBHD Launcher HD Widescreen Wrapper Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a DFBHD "HD Widescreen (1080p)" combined profile that applies HD menu textures and installs/configures the bundled widescreen wrapper, with safe backup/restore so "Original" returns the install to a clean state.

**Architecture:** Extend `crates/launcher-gui` (`model`/`app`/tests, README) with wrapper bundle resolution, managed-file install/restore helpers, and profile dispatch updates. Keep texture swap behavior in existing helpers and layer wrapper install/cleanup on top for the new combined profile and for Original restore. Bundle runtime wrapper assets under `assets/wrapper/dfbhd_widescreen/` with 1080p defaults.

**Tech Stack:** Rust 2021, `eframe/egui`, `serde/serde_json`, std fs/path/process, crate tests in `crates/launcher-gui/src/model.rs`.

**Spec:** `docs/superpowers/specs/2026-11-19-dfbhd-launcher-hd-widescreen-wrapper-design.md` (approved 2026-11-19)

## Global Constraints

- **Scope:** DFBHD launcher integration in `crates/launcher-gui` + project assets/docs only. No changes to `dfbhd.exe`, no new PFF format work, no `resource.pff` DDS work, no network/download behavior.
- **Profiles:** keep `original` ("Original") and `hd_textures` ("HD Textures", texture-only) and add `hd_widescreen_1080p` ("HD Widescreen (1080p)", combined textures + wrapper). `active_profile_id` persistence/backward compat preserved (`serde(default)` behavior unchanged).
- **Bundled wrapper source:** `assets/wrapper/dfbhd_widescreen/` resolved relative to executable directory and/or current working directory (same base strategy style as `resolve_dfbhd_hd_localres_pack_path`).
- **Wrapper HD defaults:** bundled `assets/wrapper/dfbhd_widescreen/scripts/BHD_Asi.ini` must contain `ResolutionX = 1920`, `ResolutionY = 1080`, `FOV = 90` (default comment allowed).
- **Managed wrapper runtime files (install/restore scope):** relative paths `d3d8.dll`, `d3d8.ini`, `dinput8.dll`, `scripts/BHD_Asi.asi`, `scripts/BHD_Asi.ini`. Do not install/manage debug/build/docs artifacts (`*.pdb`, `*.iobj`, `*.ipdb`, `LICENSE.md`, `README.md`) from the bundle.
- **Backup/restore safety:** before overwriting any existing install file at a managed path, create `<path>.launcherbak` only if it does not already exist. On Original restore/cleanup: for each managed path, if `<path>.launcherbak` exists restore it and remove bak; else if file exists remove it. Remove managed script files; remove `scripts/` directory only if empty after cleanup. Never modify `dfbhd.exe` or unrelated install files.
- **Texture behavior:** `hd_textures` and combined profile use existing safe `localres.pff` swap logic (preserve `localres.pff.original`, never overwrite it; fallback from `.bak` as today). Original restores `localres.pff` as today.
- **Robustness:** missing bundle/install targets and IO/permission errors must be handled gracefully (no panics) with clear actionable log messages via existing `push_log`-style logging.
- **Verification:** `cargo test -p launcher-gui` and `cargo check -p launcher-gui` must pass.

## Review Focus

1. **Incomplete wrapper cleanup on Original:** Original must remove/restore managed wrapper files (`d3d8.dll`, `d3d8.ini`, `dinput8.dll`, `scripts/BHD_Asi.asi`, `scripts/BHD_Asi.ini`) and launcherbaks so game returns to non-wrapper state; idempotent when wrapper not installed.
2. **Overwriting user files without restorable backup:** if an install already has a managed filename, first apply must create `<file>.launcherbak` once and later restore it on Original (never lose pre-existing user content).
3. **Partial apply / wrong sequencing:** combined profile must not claim success if texture or wrapper critical steps fail; failures must be logged clearly (permission/IO/missing bundle) without panic.
4. **Scope creep to unrelated files:** install/restore must touch only managed wrapper paths + `localres.pff` flows and must never delete/alter `dfbhd.exe`, unrelated PFFs, or unrelated `scripts/` contents.
5. **Profile/config compatibility:** new `hd_widescreen_1080p` profile is selectable/persisted; existing configs missing/using known ids still load correctly; `hd_textures` remains texture-only (no wrapper side effects).

## Task Structure Notes

- **Files likely touched:** `assets/wrapper/dfbhd_widescreen/scripts/BHD_Asi.ini`, `crates/launcher-gui/src/model.rs`, `crates/launcher-gui/src/app.rs` (if UI text/behavior needs adjustment), `README.md`, and this plan/spec already added.
- **Testing style:** extend existing in-module tests in `crates/launcher-gui/src/model.rs` using temp dirs and explicit-path helpers (follow `apply_dfbhd_profile_with_pack_path` patterns).

---

### Task 1: Add widescreen profile and wrapper bundle defaults/resolution helpers

**Files:**
- Modify: `assets/wrapper/dfbhd_widescreen/scripts/BHD_Asi.ini`
- Modify: `crates/launcher-gui/src/model.rs`
- Test: `crates/launcher-gui/src/model.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces: DFBHD default profiles include `Profile { id: "hd_widescreen_1080p", name: "HD Widescreen (1080p)" }` in addition to existing `original` and `hd_textures`.
- Produces: `pub(crate) fn resolve_dfbhd_wrapper_bundle_dir() -> Option<std::path::PathBuf>` resolving `assets/wrapper/dfbhd_widescreen` from `current_exe().parent()` and `current_dir()` (first existing directory wins), analogous to `resolve_dfbhd_hd_localres_pack_path`.
- Produces: managed wrapper relative paths constant/helper usable by install/restore logic: `["d3d8.dll", "d3d8.ini", "dinput8.dll", "scripts/BHD_Asi.asi", "scripts/BHD_Asi.ini"]`.

- [ ] **Step 1: Update bundled ASI config for 1080p defaults**

Edit `assets/wrapper/dfbhd_widescreen/scripts/BHD_Asi.ini` to:
```ini
[MAIN]
ResolutionX = 1920
ResolutionY = 1080
FOV = 90 //Default is 80
```

- [ ] **Step 2: Write failing tests for profile presence and wrapper bundle resolution helper**

Add tests in `model.rs` tests:
- `default_state_includes_hd_widescreen_profile`: default DFBHD profiles contain ids/names `original`/"Original", `hd_textures`/"HD Textures", `hd_widescreen_1080p`/"HD Widescreen (1080p)".
- `resolve_dfbhd_wrapper_bundle_dir_finds_assets_relative_to_cwd` (or equivalent): in a temp cwd context, or by testing against repo layout assumption if run from repo root, ensure resolver can find `assets/wrapper/dfbhd_widescreen` when present; if environment-dependent, implement helper that accepts base dirs or structure test to create expected relative path in temp dir and call an internal resolver over provided bases. Prefer adding `fn resolve_dfbhd_wrapper_bundle_dir_from_bases<I>(bases: I) -> Option<PathBuf>` (`pub(crate)`) and making `resolve_dfbhd_wrapper_bundle_dir()` delegate to it, mirroring testable pattern.

- [ ] **Step 3: Run tests to verify failures**

Run: `cargo test -p launcher-gui -- default_state_includes_hd_widescreen_profile resolve_dfbhd_wrapper_bundle_dir_finds_assets_relative_to_cwd`
Expected: FAIL (missing profile/helper).

- [ ] **Step 4: Implement profile and resolver helpers**

In `crates/launcher-gui/src/model.rs`:
- Add `HD Widescreen (1080p)` profile to `LauncherState::default()` DFBHD profiles.
- Add wrapper bundle path resolver (`resolve_dfbhd_wrapper_bundle_dir` and/or base-driven helper) checking `<base>/assets/wrapper/dfbhd_widescreen` exists and is a directory.
- Add `pub(crate) const DFBHD_WRAPPER_MANAGED_FILES: &[&str] = &["d3d8.dll", "d3d8.ini", "dinput8.dll", "scripts/BHD_Asi.asi", "scripts/BHD_Asi.ini"];` (or equivalent helper returning the list) for reuse.

- [ ] **Step 5: Run tests to verify pass**

Run: `cargo test -p launcher-gui -- default_state_includes_hd_widescreen_profile resolve_dfbhd_wrapper_bundle_dir_finds_assets_relative_to_cwd`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add assets/wrapper/dfbhd_widescreen/scripts/BHD_Asi.ini crates/launcher-gui/src/model.rs
git commit -m "feat: add DFBHD widescreen wrapper profile and bundle resolver"
```

---

### Task 2: Implement wrapper install with safe backups

**Files:**
- Modify: `crates/launcher-gui/src/model.rs`
- Test: `crates/launcher-gui/src/model.rs`

**Interfaces:**
- Produces: `pub(crate) fn install_dfbhd_wrapper_to_install(install_dir: &Path, bundle_dir: &Path, log: &mut Vec<String>) -> Result<(), String>` (or equivalent name) that copies managed wrapper files from bundle to install.
- Behavior: creates parent dirs as needed; for each managed file if destination exists and `<dest>.launcherbak` does not exist, copy existing destination to `<dest>.launcherbak` before overwrite; copy bundle file to destination; logs actions; returns/records actionable errors on missing bundle files or IO failures without panic.

- [ ] **Step 1: Write failing tests for wrapper install/backup behavior**

Add tests:
- `install_wrapper_copies_managed_files_and_creates_scripts_dir`: temp install + temp bundle containing managed files; after install, all managed files exist in install with copied contents.
- `install_wrapper_backs_up_existing_files_once`: pre-create `d3d8.dll` (and/or script file) in install with sentinel content, include different content in bundle, run install twice; `d3d8.dll.launcherbak` exists with original sentinel, installed file has bundle content, second run does not overwrite bak.

- [ ] **Step 2: Run tests to verify failures**

Run: `cargo test -p launcher-gui -- install_wrapper_copies_managed_files_and_creates_scripts_dir install_wrapper_backs_up_existing_files_once`
Expected: FAIL.

- [ ] **Step 3: Implement install helper**

Implement managed-file iteration over `DFBHD_WRAPPER_MANAGED_FILES`, validating `bundle_dir.join(rel)` is a file, ensuring destination parent exists, creating `.launcherbak` only when dest exists and bak missing, then copying bundle -> dest. Use `push_log` for success/skip/failure details and return `Err(String)` on hard failures (missing bundle file, copy/IO error) so callers can report accurately.

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p launcher-gui -- install_wrapper_copies_managed_files_and_creates_scripts_dir install_wrapper_backs_up_existing_files_once`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/src/model.rs
git commit -m "feat: install DFBHD widescreen wrapper with backups"
```

---

### Task 3: Implement wrapper restore/cleanup for Original

**Files:**
- Modify: `crates/launcher-gui/src/model.rs`
- Test: `crates/launcher-gui/src/model.rs`

**Interfaces:**
- Produces: `pub(crate) fn restore_dfbhd_wrapper_in_install(install_dir: &Path, log: &mut Vec<String>) -> Result<(), String>` (or equivalent) that restores/removes managed wrapper files.
- Behavior: for each managed path, if `<path>.launcherbak` exists restore it to `<path>` and remove bak; else if `<path>` exists remove it. After processing managed script files, remove `install_dir.join("scripts")` only if it exists and is empty. Idempotent when nothing installed. Must not touch unrelated files.

- [ ] **Step 1: Write failing tests for restore/cleanup behavior**

Add tests:
- `restore_wrapper_removes_installed_files_when_no_backup`: install dir has managed wrapper files but no baks; restore removes them (and removes empty `scripts/` dir).
- `restore_wrapper_restores_launcherbak_and_preserves_unrelated_files`: pre-existing user `d3d8.dll` content saved as `d3d8.dll.launcherbak`, wrapper content currently at `d3d8.dll`, unrelated `scripts/custom.txt` exists; restore brings back user `d3d8.dll`, removes bak, removes managed ASI files but does not delete `scripts/custom.txt` and does not remove non-empty `scripts/`.

- [ ] **Step 2: Run tests to verify failures**

Run: `cargo test -p launcher-gui -- restore_wrapper_removes_installed_files_when_no_backup restore_wrapper_restores_launcherbak_and_preserves_unrelated_files`
Expected: FAIL.

- [ ] **Step 3: Implement restore helper**

Implement reverse/cleanup logic over managed files with clear logs per action and safe handling of missing paths. Attempt best-effort cleanup but return/report errors for IO failures that prevent correct restore where practical (and log them). Ensure no deletion outside managed paths except optional empty `scripts/` dir removal.

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p launcher-gui -- restore_wrapper_removes_installed_files_when_no_backup restore_wrapper_restores_launcherbak_and_preserves_unrelated_files`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/src/model.rs
git commit -m "feat: restore DFBHD install from widescreen wrapper"
```

---

### Task 4: Integrate combined/original profile apply flows and UI/config behavior

**Files:**
- Modify: `crates/launcher-gui/src/model.rs`
- Modify: `crates/launcher-gui/src/app.rs` (only if needed for labels/messages; profile list is model-driven)
- Modify: `README.md`
- Test: `crates/launcher-gui/src/model.rs`

**Interfaces:**
- Consumes: `resolve_dfbhd_hd_localres_pack_path`, `resolve_dfbhd_wrapper_bundle_dir`, wrapper install/restore helpers, existing `apply_dfbhd_profile_with_pack_path`.
- Produces: `apply_active_profile_for_game("dfbhd")` handles `hd_widescreen_1080p` by applying HD textures and installing wrapper (using explicit-path helper(s) suitable for tests), handles `original` by restoring `localres.pff` and restoring/removing wrapper, and keeps `hd_textures` texture-only (no wrapper install/removal).
- Produces: `apply_dfbhd_hd_widescreen_with_paths(&mut self, install_dir: &str, hd_pack_path: &str, wrapper_bundle_dir: &str)` (or equivalent explicit helper) and, if useful, `apply_dfbhd_original_with_paths`/direct flow that performs texture restore + wrapper cleanup for deterministic tests.

- [ ] **Step 1: Write failing tests for profile integration**

Add tests:
- `apply_hd_widescreen_applies_textures_and_installs_wrapper`: temp install with `localres.pff`, temp HD pack, temp wrapper bundle; applying combined profile copies HD pack to `localres.pff`, creates `localres.pff.original`, and installs managed wrapper files.
- `apply_original_restores_textures_and_removes_wrapper`: after combined apply (or equivalent setup with wrapper files + `localres.pff.original`), applying `original` restores original `localres.pff` and removes managed wrapper files/launcherbaks appropriately.
- `apply_hd_textures_does_not_install_or_remove_wrapper`: install has no wrapper files; applying `hd_textures` affects only `localres.pff` and leaves wrapper paths absent (and if wrapper already present in a setup, texture-only apply must not remove it per spec — include assertion accordingly).
- `select_and_persist_hd_widescreen_profile_id`: `select_profile("hd_widescreen_1080p")` works; `export_config`/`apply_loaded_config` roundtrip preserves it (or equivalent serialization path) and unknown ids still ignored.

- [ ] **Step 2: Run tests to verify failures**

Run: `cargo test -p launcher-gui -- apply_hd_widescreen_applies_textures_and_installs_wrapper apply_original_restores_textures_and_removes_wrapper apply_hd_textures_does_not_install_or_remove_wrapper select_and_persist_hd_widescreen_profile_id`
Expected: FAIL.

- [ ] **Step 3: Implement apply dispatch/integration**

In `model.rs`:
- Extend `apply_active_profile_for_game` for DFBHD to resolve needed bundle paths and dispatch:
  - `"original"`: perform texture restore and wrapper restore/cleanup (idempotent) with clear logs.
  - `"hd_textures"`: texture-only apply via existing HD pack resolution/path logic.
  - `"hd_widescreen_1080p"`: resolve HD pack and wrapper bundle dir; apply textures then install wrapper (or a clearly reported sequencing that does not hide failures). Block/log if install not linked/dir missing or bundles missing.
- Add explicit-path helper(s) so tests do not depend on executable/cwd bundle discovery.
- Keep log messages actionable and avoid panics.

In `app.rs`: ensure DFBHD profile buttons display/select all profiles from model (current implementation already iterates profiles; adjust only if needed). Optionally clarify labels/log text if helpful.

Update `README.md` to document `HD Widescreen (1080p)` profile, wrapper bundle location (`assets/wrapper/dfbhd_widescreen`), 1920x1080/FOV 90 default in `scripts/BHD_Asi.ini`, managed files/backup-restore behavior, and that texture-only `HD Textures` remains available.

- [ ] **Step 4: Run focused tests to verify pass**

Run: `cargo test -p launcher-gui -- apply_hd_widescreen_applies_textures_and_installs_wrapper apply_original_restores_textures_and_removes_wrapper apply_hd_textures_does_not_install_or_remove_wrapper select_and_persist_hd_widescreen_profile_id`
Expected: PASS.

- [ ] **Step 5: Run full crate verification**

Run: `cargo test -p launcher-gui`
Expected: PASS.

Run: `cargo check -p launcher-gui`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/launcher-gui/src/model.rs crates/launcher-gui/src/app.rs README.md docs/superpowers/specs/2026-11-19-dfbhd-launcher-hd-widescreen-wrapper-design.md docs/superpowers/plans/2026-11-19-dfbhd-launcher-hd-widescreen-wrapper.md
git commit -m "feat: add DFBHD HD widescreen wrapper launcher profile"
```

(Include only files actually changed in this task/feature; do not commit unrelated untracked files such as `tools/`, `work/`, zips, temp artifacts, or `.superpowers/`.)

---

## Self-Review Checklist (to satisfy during/after authoring)

- [ ] Spec coverage: combined profile, wrapper install/config 1080p, backup/restore on Original, texture-only profile retained, bundle location, UI/profile visibility, logging/safety, README.
- [ ] Interfaces/signatures are explicit enough for independent task execution (`resolve_dfbhd_wrapper_bundle_dir*`, managed files list, install/restore helpers, apply helpers).
- [ ] Type/name consistency across tasks (profile id `hd_widescreen_1080p`, helper names).
- [ ] Review Focus items each have corresponding tests/ownership in tasks.
- [ ] Plan is decision-focused (not a code transcript); steps unambiguous with verifiable commands.
