# DFBHD Texture Launcher — Etapa 2 (Link & Launch) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable attaching an existing DFBHD install folder from the GUI, detecting/validating `dfbhd.exe` and `.pff` assets, persisting the link, and launching the game when linked.

**Architecture:** Extend the existing `launcher-gui` crate with small focused modules: `detect` (filesystem scan/validation of an install folder), `config` (serde JSON load/save of per-game link data), and updates to `model`, `app`, and `main` to wire real attach/launch flows and persisted state. Folder picking uses `rfd` from the UI; core detection/config/model logic remains testable without the GUI.

**Tech Stack:** Rust (stable, Windows), `eframe = "0.37"`/egui, `rfd` (native folder dialog), `serde`/`serde_json` (config persistence), std fs/process.

**Spec:** `docs/superpowers/specs/2026-11-18-dfbhd-launcher-etapa2-link-launch-design.md` (approved 2026-11-18). The spec travels with this plan; executors read both.

## Global Constraints

- Etapa 2 scope (Option A): attach existing installed game folder only. No ISO mounting/reading and no InstallShield CAB extraction.
- No binary patching, wrappers (DXVK/d3d8to9/DSOAL), LAA/config generation.
- No PFF parsing, texture extraction/upscale, or real texture/profile application.
- No network/manifest download.
- Windows target, BYOG (user supplies legally obtained installed game). Keep Etapa 1 vault UI/theme and structure.
- Detection: locate `dfbhd.exe` (case-insensitive) in selected folder root; recursively enumerate `.pff` files (case-insensitive). If `dfbhd.exe` found mark linked and record paths; if not, keep unlinked and log clear reason (do not clobber a previously valid link on failed attach attempt).
- Persistence: `launcher-config.json` next to launcher executable when available, else current working directory. Per-game data keyed by id (e.g. `dfbhd`): linked state/install_dir/exe_path/pff_paths. Load on startup; save on successful link changes.
- Launch: when linked and exe exists, `Command::new(exe_path).current_dir(install_dir).spawn()` (non-blocking) and log outcome. When not linked or exe missing, guarded with clear attach/scan/link or missing-exe message.
- Invalid/missing paths or config must not panic; log/status must be user-visible.

## Review Focus

1. **Attach invalid folder (no dfbhd.exe):** must not mark linked, must not destroy existing valid link/config, and must log a clear reason the user can act on.
2. **Launch when unlinked or exe path missing/stale:** must be guarded (no spawn/panic) with clear log guidance (attach/link or missing executable).
3. **Config missing/corrupt/unreadable:** startup must fall back to defaults without panic; app remains usable.
4. **Case-insensitive detection on Windows paths:** `dfbhd.exe` / `.pff` detection must work regardless of filename case and should not depend on directory iteration order.
5. **Re-attach/replace flow:** attaching a new valid folder updates paths/state/UI and persisted config correctly (replaces previous link for that game).

---

## File Structure

- Modify: `crates/launcher-gui/Cargo.toml` — add dependencies `rfd = "0.15"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`.
- Create: `crates/launcher-gui/src/detect.rs` — `DetectedInstall` and `scan_install_dir` (locate `dfbhd.exe` in root, recursive `.pff` discovery). Unit-tested with temp dirs.
- Create: `crates/launcher-gui/src/config.rs` — serializable config types, `config_path`, `load_config`, `save_config`. Unit-tested (roundtrip/missing/corrupt handling).
- Modify: `crates/launcher-gui/src/model.rs` — extend `GameNode` with path fields, add config apply/export, real folder-link and launch behavior, logging; update tests.
- Modify: `crates/launcher-gui/src/app.rs` — SCAN/ATTACH uses `rfd` folder picker and links/saves config, DFBHD view shows linked path + detection summary, LAUNCH triggers real launch.
- Modify: `crates/launcher-gui/src/main.rs` — declare modules, load config on startup and apply to initial state.
- Modify: `README.md` — document Etapa 2 capabilities (attach installed folder, detection, persistence, launch) and remaining non-goals.

Responsibilities: `detect` = filesystem validation only; `config` = serialization/path resolution only; `model` = state transitions/link/launch orchestration; `app` = UI + dialog + save trigger; `main` = bootstrap/load.

---

### Task 1: Dependencies and config persistence module

**Files:**
- Modify: `crates/launcher-gui/Cargo.toml`
- Create: `crates/launcher-gui/src/config.rs`
- Modify: `crates/launcher-gui/src/main.rs` (add `mod config;`)

**Interfaces:**
- Consumes: none (serde/std).
- Produces: `#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)] pub struct GameLinkConfig { pub source_linked: bool, pub install_dir: Option<String>, pub exe_path: Option<String>, pub pff_paths: Vec<String> }`; `#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)] pub struct LauncherConfig { pub games: std::collections::BTreeMap<String, GameLinkConfig> }`; `pub fn config_path() -> std::path::PathBuf` (exe dir joined with `launcher-config.json`, fallback cwd); `pub fn load_config() -> LauncherConfig` (missing/unreadable/invalid JSON => `LauncherConfig::default()`); `pub fn save_config(cfg: &LauncherConfig) -> Result<(), String>`.

- [ ] **Step 1: Add dependencies and failing config tests**

In `crates/launcher-gui/Cargo.toml` add:
```toml
rfd = "0.15"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Create `crates/launcher-gui/src/config.rs` with `#[cfg(test)] mod tests`:
```rust
#[test]
fn roundtrip_config_serialization() {
    let mut cfg = LauncherConfig::default();
    cfg.games.insert("dfbhd".to_string(), GameLinkConfig {
        source_linked: true,
        install_dir: Some("C:/Games/DFBHD".to_string()),
        exe_path: Some("C:/Games/DFBHD/dfbhd.exe".to_string()),
        pff_paths: vec!["C:/Games/DFBHD/resource.pff".to_string()],
    });
    let json = serde_json::to_string(&cfg).unwrap();
    let back: LauncherConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn load_invalid_json_string_as_default_logic() {
    let bad: Result<LauncherConfig, _> = serde_json::from_str("not json");
    assert!(bad.is_err());
}
```
(Also plan for `load_config` to return default on missing/corrupt file; expose pure parse behavior via serde and keep load resilient.)

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p launcher-gui config` (or `cargo test -p launcher-gui`)
Expected: FAIL (missing `config` module/types).

- [ ] **Step 3: Implement config module**

Add `mod config;` in `main.rs`. Implement structs with serde derives, `config_path()` using `std::env::current_exe()` parent fallback to `std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))`, `load_config()` reading file and parsing JSON returning default on any error, `save_config()` writing pretty JSON and mapping IO/serialize errors to `String`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p launcher-gui`
Expected: PASS (existing 7 + new config tests).

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/Cargo.toml crates/launcher-gui/src/config.rs crates/launcher-gui/src/main.rs
git commit -m "feat: add config persistence for launcher links"
```

---

### Task 2: Install folder detection

**Files:**
- Create: `crates/launcher-gui/src/detect.rs`
- Modify: `crates/launcher-gui/src/main.rs` (add `mod detect;`)

**Interfaces:**
- Consumes: std path/fs.
- Produces: `#[derive(Debug, Clone, PartialEq, Eq)] pub struct DetectedInstall { pub install_dir: String, pub exe_path: String, pub pff_paths: Vec<String> }`; `pub fn scan_install_dir(dir: &std::path::Path) -> Result<DetectedInstall, String>` which: (1) errors if dir missing/not a dir, (2) finds `dfbhd.exe` in root directory case-insensitively (returns clear error if not found), (3) recursively collects files with `.pff` extension case-insensitively, (4) returns paths as strings (sorted for stable output/UI).

- [ ] **Step 1: Write failing detection tests using temp dirs**

In `crates/launcher-gui/src/detect.rs` add tests that create unique temp directories under `std::env::temp_dir()` (e.g. include pid/timestamp), populate files, and assert:
```rust
#[test]
fn detects_exe_case_insensitively_and_collects_pffs_recursively() { /* dir with DFBHD.EXE and subdir/a.pff, root/B.PFF */ }

#[test]
fn errors_when_exe_missing() { /* dir without dfbhd.exe */ }
```
Clean up temp dirs in tests where practical.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p launcher-gui detect`
Expected: FAIL (missing module/types/function).

- [ ] **Step 3: Implement scan_install_dir**

Add `mod detect;` in `main.rs`. Implement recursive directory walk (manual stack/recursion over `std::fs::read_dir`) to find root-level `dfbhd.exe` match (case-insensitive file name compare) and collect `.pff` paths; sort `pff_paths` for deterministic results. Return descriptive `Err(String)` for invalid dir or missing exe.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p launcher-gui`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/src/detect.rs crates/launcher-gui/src/main.rs
git commit -m "feat: detect installed DFBHD folder and PFF assets"
```

---

### Task 3: Model integration for link, config, and launch

**Files:**
- Modify: `crates/launcher-gui/src/model.rs`
- Test: `crates/launcher-gui/src/model.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: `crate::detect::{DetectedInstall, scan_install_dir}`, `crate::config::{GameLinkConfig, LauncherConfig}`.
- Produces: `GameNode` gains `pub install_dir: Option<String>, pub exe_path: Option<String>, pub pff_paths: Vec<String>` (defaulted for DFBHD/slots). `LauncherState` gains: `pub fn apply_loaded_config(&mut self, cfg: &crate::config::LauncherConfig)` (applies saved link for matching game ids, updates source_linked/label/paths/log note safely), `pub fn link_game_folder(&mut self, game_id: &str, selected_dir: &str) -> Result<(), String>` (runs detection; on success updates node to linked with paths/label and logs summary including pff count/names hint; on failure logs clear reason and returns Err without clearing an existing valid link), `pub fn export_config(&self) -> crate::config::LauncherConfig`, `pub fn launch_game(&mut self, game_id: &str)` (if not linked or exe missing -> guarded log "attach/scan/link" or "missing executable"; if linked and exe exists -> `std::process::Command::new(exe).current_dir(install_dir).spawn()` and log success or spawn error; never panic). `apply_profile_stub`/existing behavior preserved.

- [ ] **Step 1: Write failing tests for link/config/launch behavior**

Add tests in `model.rs` (use temp dirs/files for detection/launch-safe cases):
- `link_game_folder_marks_linked_and_records_paths` (temp dir with `dfbhd.exe` and `.pff` -> linked, paths populated, log mentions detected exe/pffs).
- `link_game_folder_failure_does_not_clear_existing_link` (link valid dir, then attempt invalid dir -> still linked to previous valid paths, error logged).
- `launch_game_guarded_when_unlinked` (replaces/updates prior expectation: logs attach/scan/link guidance).
- `apply_loaded_and_export_config_roundtrip_state` (apply config sets fields; export returns them).

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p launcher-gui`
Expected: FAIL (missing fields/methods/changed behavior).

- [ ] **Step 3: Implement model changes**

Update `GameNode`/`Default` with new fields; implement `apply_loaded_config`, `export_config`, `link_game_folder` (uses `scan_install_dir`, preserves prior link on failure), and `launch_game` with spawn+logging and guarded paths. Keep logs bounded. Ensure `source_label` becomes the linked install dir (or clear "[not linked]" state) consistently.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p launcher-gui`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/src/model.rs
git commit -m "feat: integrate install linking, config state, and real launch"
```

---

### Task 4: UI wiring and startup persistence

**Files:**
- Modify: `crates/launcher-gui/src/app.rs`
- Modify: `crates/launcher-gui/src/main.rs`
- (uses) `crates/launcher-gui/src/config.rs`, `detect.rs`, `model.rs`

**Interfaces:**
- Consumes: `LauncherState` APIs from Task 3 (`link_game_folder`, `launch_game`, `export_config`), `config::{load_config, save_config}`.
- Produces: `SCAN/ATTACH` button opens `rfd::FileDialog::new().pick_folder()` for the selected/DFBHD context; on selection attempts link and on success saves config via `save_config(&state.export_config())` logging save errors; DFBHD view displays `[LINKED]/[UNLINKED]`, linked install path, detection summary (`dfbhd.exe` found + PFF count and representative names), and allows re-attach; `LAUNCH` button calls `launch_game` (enabled action but behavior guarded by model). `main` loads config at startup and applies it to default state before running app.

- [ ] **Step 1: Wire startup config load in main**

In `main.rs`, load config with `config::load_config()`, create `LauncherState::default()`, apply loaded config, and pass initialized app/state (add `LauncherApp::with_state` or equivalent constructor) into eframe.

- [ ] **Step 2: Update app UI for attach, summary, and launch**

In `app.rs`: replace stub `scan_attach_stub` call with folder picker + `link_game_folder("dfbhd", ...)` and config save on success; update DFBHD source/summary lines to show linked path, exe presence, and PFF count/names (e.g. first few names) from state; wire `LAUNCH` to `launch_game("dfbhd")`. Keep layout resilient and text monospaced/vault-styled.

- [ ] **Step 3: Verify compile and tests**

Run: `cargo check -p launcher-gui`
Expected: PASS.

Run: `cargo test -p launcher-gui`
Expected: PASS.

Manual check (Windows desktop): `cargo run -p launcher-gui` -> SCAN/ATTACH opens folder picker, linking a valid install shows LINKED/paths/PFF summary and persists to `launcher-config.json`; invalid folder logs error without clearing prior link; LAUNCH starts game when linked or logs guarded/missing-exe when not.

- [ ] **Step 4: Commit**

```bash
git add crates/launcher-gui/src/app.rs crates/launcher-gui/src/main.rs
git commit -m "feat: wire folder attach UI, persistence, and launch"
```

---

### Task 5: Documentation and final verification

**Files:**
- Modify: `README.md`

**Interfaces:**
- Consumes: implemented Etapa 2 behavior.
- Produces: README reflecting Etapa 2 (attach installed folder, detection of dfbhd.exe/.pff, config persistence, real launch when linked) and clearly stating remaining non-goals (no ISO/InstallShield extraction, no patching/wrappers, no texture processing/application, no network).

- [ ] **Step 1: Run full verification**

Run: `cargo test -p launcher-gui`
Expected: PASS.

Run: `cargo check -p launcher-gui`
Expected: PASS.

- [ ] **Step 2: Update README**

Update `README.md` purpose/capabilities/scope to include Etapa 2 features and keep BYOG/ISO-as-context notes accurate (ISO not read/extracted by launcher in Etapa 2).

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: document etapa 2 linking and launch support"
```

- [ ] **Step 4: Final check**

Run: `cargo test -p launcher-gui`
Expected: PASS.

---

## Self-Review Notes

- **Spec coverage:** Option A attach via folder picker, detection (dfbhd.exe case-insensitive + recursive .pff), path storage/UI summary, persistence in `launcher-config.json` near exe with fallback, startup load, and real `LAUNCH` when linked/guarded otherwise all mapped to Tasks 1-5. Non-goals (ISO/InstallShield extraction, patching/wrappers, PFF/texture processing, network) respected.
- **Interfaces:** `DetectedInstall`/`scan_install_dir` (Task 2) consumed by model (Task 3); config types/load/save (Task 1) consumed by main/app/model (Tasks 3-4); model APIs (`link_game_folder`, `launch_game`, `export_config`, `apply_loaded_config`) consumed by UI/startup (Task 4).
- **Type/name consistency:** to be maintained as specified (`GameNode` gains `install_dir`, `exe_path`, `pff_paths`; methods named as above). Implementers must keep signatures aligned across tasks.
- **Review Focus mapped:** invalid-folder non-clobber/log (Tasks 2,3 tests/UI); guarded launch on unlinked/missing exe (Task 3 tests); corrupt/missing config fallback (Task 1/3 behavior); case-insensitive detection and stable results (Task 2 tests); re-attach replaces persisted link (Tasks 3,4).
- **Proportion:** UI rendering details left to implementer within specified behaviors; core logic specified by signatures/tests/commands.
