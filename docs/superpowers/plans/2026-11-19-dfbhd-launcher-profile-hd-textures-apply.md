# DFBHD Launcher: Profile HD Textures Apply (A) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make DFBHD `Original` / `HD Textures` profiles perform real, safe apply/restore of a prebuilt `localres.pff` HD pack (menu textures) from the launcher, with backup/restore, persisted active profile, and log feedback.

**Architecture:** Extend `launcher-gui` config/model to persist `active_profile_id` and replace the profile-apply stub with real file swap logic for DFBHD (`localres.pff` only). Resolve the bundled HD pack from app base paths (`assets/profiles/dfbhd/hd_textures/localres.pff`), apply via explicit UI action with robust backup/restore (including existing `.bak`/missing `localres.pff` install state), and surface results in the DFBHD log.

**Tech Stack:** Rust (stable, Windows), `eframe`/egui, `serde`/`serde_json`, std `fs`/`path`.

**Spec:** `docs/superpowers/specs/2026-11-19-dfbhd-launcher-profile-hd-textures-apply-design.md` (approved 2026-11-19). The spec travels with this plan; executors read both.

## Global Constraints

- Scope A now (DFBHD only): prebuilt `localres.pff` menu textures (`BHD_main.tga`, `Btn_exit.tga`) via bundled pack `assets/profiles/dfbhd/hd_textures/localres.pff`. No runtime upscaling, no DDS/`resource.pff`/`med.pff` processing, no ISO/InstallShield extraction, no patching/wrappers, no network/manifest.
- Apply is explicit via **APPLY PROFILE** (no auto-swap on selection). Must not panic on missing paths/pack, I/O, or permission errors; log clear actionable messages.
- Backup/restore must be safe and reversible: never overwrite an existing canonical original backup; handle install state where `localres.pff` is missing and `localres.pff.bak` exists; `Original` must restore from `.original` or fallback `.bak`.
- Persist `active_profile_id` per game in `launcher-config.json` with serde defaults/backward compatibility for existing configs.
- B (user-supplied pack paths) deferred; do not add that UI/config now.

## Review Focus

1. **Existing install state (`localres.pff` missing, `localres.pff.bak` present):** APPLY `Original` must restore a working `localres.pff` from backup without data loss; APPLY `HD Textures` must establish/use a recoverable original backup (not lose `.bak`) before writing HD pack.
2. **Idempotent/safe re-apply:** repeated APPLY of `HD Textures` or `Original` must not destroy the canonical original backup or corrupt install state when files already in target state.
3. **Failure safety:** missing/unreadable HD pack, missing/unlinked install, or copy/permission failure must not clobber existing backups or leave misleading success logs.
4. **Config backward compatibility:** loading configs without `active_profile_id` must default sanely (DFBHD -> `original` if unset/invalid), and export/load roundtrip must preserve it.
5. **Pack path resolution:** HD pack must be found in dev runs (repo/cwd base) and when running from built exe dir layout where `assets/...` is adjacent to the base used by config resolution.

---

## File Structure

- Modify: `crates/launcher-gui/src/config.rs` - add persisted per-game `active_profile_id` with serde default/backward compat; update tests.
- Modify: `crates/launcher-gui/src/model.rs` - persist/load active profile, add HD pack path resolution and real profile apply/backup-restore logic for DFBHD `localres.pff`, replace stub usage; update/add tests.
- Modify: `crates/launcher-gui/src/app.rs` - wire APPLY PROFILE to real apply, save config on apply (and keep scan-attach save behavior), adjust log/status text as needed.
- Modify: `README.md` - document Etapa 3a profile-apply capability/scope and remaining non-goals.
- Existing asset: `assets/profiles/dfbhd/hd_textures/localres.pff` (bundled corrected HD pack).

Responsibilities: `config` = serialization of link + active profile; `model` = profile state, pack resolution, apply/swap/backup/restore and logging; `app` = UI action + persistence trigger.

---

### Task 1: Persist `active_profile_id` in config and model

**Files:**
- Modify: `crates/launcher-gui/src/config.rs`
- Modify: `crates/launcher-gui/src/model.rs`
- Test: `crates/launcher-gui/src/config.rs` (`#[cfg(test)]`), `crates/launcher-gui/src/model.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: existing `LauncherConfig`, `GameLinkConfig`, `LauncherState`, `GameNode`.
- Produces: `pub active_profile_id: Option<String>` on `GameLinkConfig` with `#[serde(default)]` so older configs deserialize; `LauncherState::export_config()` includes each game's `active_profile_id` (`Some(id)` when non-empty, else `None` or appropriate default); `LauncherState::apply_loaded_config(&mut self, cfg: &LauncherConfig)` restores a game's `active_profile_id` only if the saved id is present and matches one of that game's profiles, otherwise leaves the existing/default profile id unchanged.

- [ ] **Step 1: Write failing config/model tests for active profile persistence**

In `crates/launcher-gui/src/config.rs` tests add:
```rust
#[test]
fn game_link_config_deserializes_missing_active_profile_for_backward_compat() {
    let json = r#"{
        "games": {
            "dfbhd": {
                "source_linked": true,
                "install_dir": "C:/Games/DFBHD",
                "exe_path": "C:/Games/DFBHD/dfbhd.exe",
                "pff_paths": ["C:/Games/DFBHD/localres.pff"]
            }
        }
    }"#;
    let cfg: LauncherConfig = serde_json::from_str(json).expect("old config should deserialize");
    let g = cfg.games.get("dfbhd").expect("dfbhd entry");
    assert_eq!(g.active_profile_id, None);
}

#[test]
fn roundtrip_config_serialization_includes_active_profile() {
    let mut cfg = LauncherConfig::default();
    cfg.games.insert(
        "dfbhd".to_string(),
        GameLinkConfig {
            source_linked: true,
            install_dir: Some("C:/Games/DFBHD".to_string()),
            exe_path: Some("C:/Games/DFBHD/dfbhd.exe".to_string()),
            pff_paths: vec!["C:/Games/DFBHD/localres.pff".to_string()],
            active_profile_id: Some("hd_textures".to_string()),
        },
    );
    let json = serde_json::to_string(&cfg).unwrap();
    let back: LauncherConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(cfg, back);
}
```
Update existing config tests constructing `GameLinkConfig` to include `active_profile_id` as needed.

In `crates/launcher-gui/src/model.rs` tests add:
```rust
#[test]
fn apply_loaded_and_export_config_preserves_active_profile() {
    let mut cfg = LauncherConfig::default();
    cfg.games.insert(
        "dfbhd".to_string(),
        GameLinkConfig {
            source_linked: true,
            install_dir: Some("C:/Games/DFBHD".to_string()),
            exe_path: Some("C:/Games/DFBHD/dfbhd.exe".to_string()),
            pff_paths: vec!["C:/Games/DFBHD/localres.pff".to_string()],
            active_profile_id: Some("hd_textures".to_string()),
        },
    );
    let mut s = LauncherState::default();
    s.apply_loaded_config(&cfg);
    let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();
    assert_eq!(g.active_profile_id, "hd_textures");
    assert_eq!(
        s.export_config()
            .games
            .get("dfbhd")
            .unwrap()
            .active_profile_id
            .as_deref(),
        Some("hd_textures")
    );
}

#[test]
fn apply_loaded_config_ignores_unknown_active_profile() {
    let mut cfg = LauncherConfig::default();
    cfg.games.insert(
        "dfbhd".to_string(),
        GameLinkConfig {
            source_linked: false,
            install_dir: None,
            exe_path: None,
            pff_paths: vec![],
            active_profile_id: Some("nope".to_string()),
        },
    );
    let mut s = LauncherState::default();
    let before = s.games.iter().find(|g| g.id == "dfbhd").unwrap().active_profile_id.clone();
    s.apply_loaded_config(&cfg);
    let after = s.games.iter().find(|g| g.id == "dfbhd").unwrap().active_profile_id.clone();
    assert_eq!(after, before);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p launcher-gui`
Expected: FAIL (missing `active_profile_id` field / assertions).

- [ ] **Step 3: Implement config/model persistence**

In `config.rs`, add to `GameLinkConfig`:
```rust
pub active_profile_id: Option<String>,
```
(serde default via struct-level `#[serde(default)]` remains). Update `config.rs` and `model.rs` struct literals in tests as required.

In `model.rs`:
- `apply_loaded_config`: after applying link fields for a matched game, if `saved.active_profile_id` is `Some(id)` where `!id.is_empty()` and `game.profiles.iter().any(|p| p.id == *id)`, set `game.active_profile_id = id.clone()`.
- `export_config`: set `GameLinkConfig.active_profile_id = if game.active_profile_id.is_empty() { None } else { Some(game.active_profile_id.clone()) }`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p launcher-gui`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/src/config.rs crates/launcher-gui/src/model.rs
git commit -m "feat: persist active profile in launcher config"
```

---

### Task 2: Implement HD/Original profile apply with backup-restore

**Files:**
- Modify: `crates/launcher-gui/src/model.rs`
- Test: `crates/launcher-gui/src/model.rs`

**Interfaces:**
- Consumes: `GameNode { source_linked, install_dir, profiles, active_profile_id, log, .. }`, filesystem, bundled pack file at resolved asset path.
- Produces:
  - `pub fn resolve_dfbhd_hd_localres_pack_path() -> Option<std::path::PathBuf>`: returns the first existing candidate among `<base>/assets/profiles/dfbhd/hd_textures/localres.pff` for base = `std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()))` and base = `std::env::current_dir().ok()` (try exe-base first, then cwd; ignore missing bases/paths).
  - `pub fn apply_active_profile_for_game(&mut self, game_id: &str)`: for `dfbhd` and profiles `original`/`hd_textures`, performs safe swap/restore of `<install_dir>/localres.pff` and logs outcome; for unlinked/missing install/unknown game/profile, logs clear error and makes no destructive changes. Replaces prior stub behavior for this flow.
  - Keep `pub fn apply_profile_stub(&mut self)` delegating to applying the selected game (e.g. resolve selected game id and call `apply_active_profile_for_game`) so existing call sites remain valid.

- [ ] **Step 1: Write failing tests for apply behavior and pack resolution helper contract**

In `crates/launcher-gui/src/model.rs` tests add helpers and tests:

```rust
fn write_file(path: &std::path::Path, contents: &[u8]) {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent).unwrap(); }
    std::fs::write(path, contents).unwrap();
}

#[test]
fn apply_original_restores_localres_from_bak_when_missing() {
    let dir = unique_temp_dir("apply-orig-bak");
    let bak = dir.join("localres.pff.bak");
    write_file(&bak, b"ORIGINAL");
    // no localres.pff
    let mut s = LauncherState::default();
    let g = s.games.iter_mut().find(|g| g.id == "dfbhd").unwrap();
    g.source_linked = true;
    g.install_dir = Some(dir.to_string_lossy().to_string());
    g.active_profile_id = "original".to_string();
    g.log.clear();

    s.apply_active_profile_for_game("dfbhd");

    let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();
    assert_eq!(std::fs::read(dir.join("localres.pff")).unwrap(), b"ORIGINAL");
    assert!(g.log.join("\n").to_lowercase().contains("original") || g.log.join("\n").to_lowercase().contains("restor"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn apply_hd_creates_original_backup_and_copies_pack() {
    let dir = unique_temp_dir("apply-hd");
    write_file(&dir.join("localres.pff"), b"ORIGINAL");
    let pack_dir = unique_temp_dir("pack-hd");
    let pack = pack_dir.join("localres.pff");
    write_file(&pack, b"HD");

    let mut s = LauncherState::default();
    let g = s.games.iter_mut().find(|g| g.id == "dfbhd").unwrap();
    g.source_linked = true;
    g.install_dir = Some(dir.to_string_lossy().to_string());
    g.active_profile_id = "hd_textures".to_string();
    g.log.clear();

    // helper intended for tests to supply explicit pack path
    s.apply_dfbhd_profile_with_pack_path("hd_textures", &dir.to_string_lossy(), &pack.to_string_lossy());

    assert_eq!(std::fs::read(dir.join("localres.pff.original")).unwrap(), b"ORIGINAL");
    assert_eq!(std::fs::read(dir.join("localres.pff")).unwrap(), b"HD");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&pack_dir);
}

#[test]
fn apply_hd_uses_bak_as_original_source_when_localres_missing() {
    let dir = unique_temp_dir("apply-hd-bak");
    write_file(&dir.join("localres.pff.bak"), b"ORIGINAL");
    let pack_dir = unique_temp_dir("pack-hd-bak");
    let pack = pack_dir.join("localres.pff");
    write_file(&pack, b"HD");

    let mut s = LauncherState::default();
    s.apply_dfbhd_profile_with_pack_path("hd_textures", &dir.to_string_lossy(), &pack.to_string_lossy());

    assert_eq!(std::fs::read(dir.join("localres.pff")).unwrap(), b"HD");
    assert!(dir.join("localres.pff.bak").exists());
    assert_eq!(std::fs::read(dir.join("localres.pff.original")).unwrap_or_else(|_| std::fs::read(dir.join("localres.pff.bak")).unwrap()), b"ORIGINAL");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&pack_dir);
}

#[test]
fn apply_hd_missing_pack_does_not_overwrite_or_drop_backup() {
    let dir = unique_temp_dir("apply-hd-missing");
    write_file(&dir.join("localres.pff"), b"ORIGINAL");
    let mut s = LauncherState::default();
    let g = s.games.iter_mut().find(|g| g.id == "dfbhd").unwrap();
    g.source_linked = true;
    g.install_dir = Some(dir.to_string_lossy().to_string());
    g.active_profile_id = "hd_textures".to_string();
    g.log.clear();

    s.apply_dfbhd_profile_with_pack_path("hd_textures", &dir.to_string_lossy(), "C:/definitely/missing/localres.pff");

    assert_eq!(std::fs::read(dir.join("localres.pff")).unwrap(), b"ORIGINAL");
    let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();
    assert!(g.log.join("\n").to_lowercase().contains("missing") || g.log.join("\n").to_lowercase().contains("not found") || g.log.join("\n").to_lowercase().contains("fail"));
    let _ = std::fs::remove_dir_all(&dir);
}
```

Adjust exact helper signature names in implementation to match (`apply_dfbhd_profile_with_pack_path` as a testable/internal helper taking `(profile_id, install_dir, pack_path)` is acceptable if `pub(crate)` or `pub`).

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p launcher-gui`
Expected: FAIL (missing apply functions/helpers).

- [ ] **Step 3: Implement apply logic and pack resolution**

In `model.rs` add:
- `resolve_dfbhd_hd_localres_pack_path() -> Option<PathBuf>` checking, in order, `current_exe().parent()/assets/profiles/dfbhd/hd_textures/localres.pff` and `current_dir()/assets/profiles/dfbhd/hd_textures/localres.pff`, returning first existing file.
- Internal/pubic helper, e.g. `pub fn apply_dfbhd_profile_with_pack_path(&mut self, profile_id: &str, install_dir: &str, hd_pack_path: &str)`, that:
  - resolves `install = Path::new(install_dir)`, `target = install.join("localres.pff")`, `original_backup = install.join("localres.pff.original")`, `bak = install.join("localres.pff.bak")`.
  - for `"original"`: copy from `original_backup` if exists else from `bak` if exists into `target`; log success (`restored original ... from ...`) or clear failure if no source/copy fails.
  - for `"hd_textures"`: verify HD pack path exists as file else log "HD pack not found" and return without modifying target/backups; ensure canonical original backup: if `original_backup` missing, create it from `bak` when `bak` exists (preferred original source, especially if target missing) else from `target` when `target` exists; if still no backup source and target missing, log clear error and return; then copy HD pack to `target` and log success.
  - never delete `.bak`; never overwrite existing `localres.pff.original`.
- `pub fn apply_active_profile_for_game(&mut self, game_id: &str)`:
  - locate game; if `dfbhd`, require `source_linked` and `install_dir` else log guarded message and return; branch on `active_profile_id`: `original` calls helper with empty/ignored pack path, `hd_textures` resolves pack via `resolve_dfbhd_hd_localres_pack_path()` and errors clearly if `None` else calls helper; unknown profile logs message.
  - for non-dfbhd games, keep simple informative log (no file ops).
- `pub fn apply_profile_stub(&mut self)`: if selected game exists call `apply_active_profile_for_game(&selected_id)` else no-op. (Remove/replace old stub-only log behavior.)

Ensure `fs::copy` errors are caught and logged (no panic).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p launcher-gui`
Expected: PASS, including new apply tests. Update any old tests expecting `"apply profile stub"` text (e.g. `select_and_apply_profile_stub_are_safe`, `log_is_bounded_to_cap_after_repeated_pushes`, empty/unselected safety tests) to assert non-panic and/or presence of new meaningful log content.

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/src/model.rs
git commit -m "feat: apply and restore DFBHD localres texture profiles"
```

---

### Task 3: Wire UI apply/save and document behavior

**Files:**
- Modify: `crates/launcher-gui/src/app.rs`
- Modify: `README.md`
- Test: existing crate tests (`cargo test -p launcher-gui`)

**Interfaces:**
- Consumes: `LauncherState::apply_active_profile_for_game`/`apply_profile_stub`, `config::save_config`, `LauncherState::export_config`.
- Produces: APPLY PROFILE button triggers real apply for DFBHD and persists config (`active_profile_id` and existing link state) after apply attempt/success as appropriate (at minimum call save after apply to persist active profile/log-relevant state safely); README reflects that profile apply for DFBHD `localres.pff` (Original/HD Textures, prebuilt pack) is implemented with backup/restore, and lists remaining non-goals.

- [ ] **Step 1: Write/update expectations (tests) if needed**

If any app behavior is covered by model tests only, ensure no tests depend on old stub strings. (UI not directly unit-tested here; rely on model tests and compile.)

- [ ] **Step 2: Run tests to establish baseline**

Run: `cargo test -p launcher-gui`
Expected: PASS before UI/doc edits (or identify failures to address).

- [ ] **Step 3: Implement UI and README updates**

In `app.rs` DFBHD APPLY PROFILE handler, replace `self.state.apply_profile_stub()` call with `self.state.apply_active_profile_for_game("dfbhd")` (or equivalent) and then attempt `config::save_config(&self.state.export_config())`, logging save failure to DFBHD game log like scan/attach does.

In `README.md`, update current scope/non-goals to state that DFBHD profile apply now swaps/restores prebuilt `localres.pff` (Original/HD Textures, menu textures) with backup/restore when linked, while still noting remaining non-goals (no runtime upscaling/DDS pipeline, no ISO/CAB extraction, no other PFFs, no user-supplied pack config [B later], no wrappers/patching/network).

- [ ] **Step 4: Run full verification**

Run: `cargo test -p launcher-gui`
Expected: PASS.

Run: `cargo check -p launcher-gui`
Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add crates/launcher-gui/src/app.rs README.md
git commit -m "feat: wire DFBHD profile apply in launcher UI"
```

---

## Self-Review

**1. Spec coverage:** Intent/scope (DFBHD, `localres.pff` menu textures, explicit APPLY) -> Tasks 2-3. Bundled pack at `assets/profiles/dfbhd/hd_textures/localres.pff` and base-dir resolution -> Task 2. Safe backup/restore including `.bak`/missing `localres.pff`, never overwrite `.original`, `Original` restores from `.original`/`.bak` -> Task 2 tests/impl. Persist `active_profile_id` per game with backward compat -> Task 1. UI apply + logging, no auto-swap -> Task 3. README/non-goals updated -> Task 3. Permission/missing-pack failures logged, no panic -> Task 2/3 constraints.

**2. Step scan:** Each step is a concrete action (write specific tests/signatures, run command with expected result, implement named behavior, commit). No TBD.

**3. Type consistency:** `GameLinkConfig.active_profile_id: Option<String>` used consistently in config/model/tests; helper `apply_dfbhd_profile_with_pack_path(profile_id: &str, install_dir: &str, hd_pack_path: &str)` and `apply_active_profile_for_game(&mut self, game_id: &str)` defined/used consistently; `resolve_dfbhd_hd_localres_pack_path() -> Option<PathBuf>`.

**4. Review Focus:** Missing-`localres.pff`+`.bak` covered by `apply_original_restores_localres_from_bak_when_missing` and `apply_hd_uses_bak_as_original_source_when_localres_missing` (Task 2). Idempotent/safe re-apply implied by "never overwrite existing canonical original backup" and backup-creation rules (implementer must follow; add assertion in implementation reasoning/tests if convenient). Missing pack failure covered by `apply_hd_missing_pack_does_not_overwrite_or_drop_backup`. Config compat covered in Task 1. Pack resolution covered by deterministic candidate logic and dev-layout path.

**5. Proportion:** Plan focuses on decisions, signatures, and key test cases; implementation bodies described by rules rather than fully transcribed.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-11-19-dfbhd-launcher-profile-hd-textures-apply.md`. Please review the plan. Which execution approach would you prefer?

- **Subagent-driven** - A fresh subagent implements each task and a fresh reviewer checks it before the next one starts, then a whole-branch review at the end. Most thorough; costs a fresh context per task and per review.
- **Native** - I implement every task myself in this session, the way this harness runs work, then one fresh reviewer on the most capable model checks the whole branch. Cheapest and fastest; no independent review until the end. Runs well with a mid-tier session model, since the plan carries the design.

**For this plan I recommend Subagent-driven, because the tasks touch shared config/model/app state and have important backup/restore edge cases where an independent review per task reduces risk of data-loss regressions. Does the plan capture what you want, and which approach should we use?**
