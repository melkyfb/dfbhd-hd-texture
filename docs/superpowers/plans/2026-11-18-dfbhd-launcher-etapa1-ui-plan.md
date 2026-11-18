# DFBHD Texture Launcher — Etapa 1 (Fundação/UI) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a runnable Rust `eframe/egui` desktop launcher shell for DFBHD with a melky.tech-inspired "secure vault/terminal" UI, stubbed DFBHD texture profiles, and a simple multi-game-ready registry.

**Architecture:** A small Cargo workspace with one binary crate (`crates/launcher-gui`) split into focused modules: `model` (pure game/profile/state logic), `theme` (vault visual tokens/styles), `app` (egui UI/navigation/screens), and `main` (entrypoint). Etapa 1 uses stubbed data/actions only; ISO/CAB extraction, patching, wrappers, and real texture operations are deferred.

**Tech Stack:** Rust (stable, MSVC toolchain on Windows), Cargo workspace, `eframe = "0.37"` / `egui` (native GUI).

**Spec:** `docs/superpowers/specs/2026-11-18-dfbhd-launcher-etapa1-ui-design.md` (approved 2026-11-18). The spec travels with this plan; executors read both.

## Global Constraints

- Etapa 1 scope only: deliver runnable themed GUI + stubbed DFBHD profiles/multi-game registry. No ISO mounting/CAB extraction, no real file detection/install logic, no binary patching, no wrappers (DXVK/d3d8to9/DSOAL), no LAA/config generation, no PFF parsing, no texture extraction/upscale/application, no network/manifest download.
- BYOG context: ISO `Novalogic_DeltaForceBllackHawkDown.iso` exists locally; do not require or distribute original game binaries/assets.
- Visual identity inspired by `https://melky.tech`: dark "secure vault/terminal" aesthetic, subtle bordered panels, cyan/green accent highlights, monospaced/technical labels/status text (e.g. `SYSTEM_STATE`, `NODE`, `PROFILE`, `STATUS`).
- Screens/layout per spec: Dashboard (status + game nodes/slots incl. DFBHD), DFBHD page (source ISO/install placeholder "not linked", profiles `Original` and `HD Textures` with stub toggle/apply, actions `SCAN/ATTACH`, `APPLY PROFILE`, `LAUNCH` stub/guarded, log/status panel), Settings (placeholders).
- Multi-game base: simple data-driven registry (`id`, name, paths, profiles) with DFBHD as first entry and placeholder slots; no complex plugin system.
- Workspace is not a git repo; commit steps in tasks are N/A here (omit git commands) — verify with `cargo` checks/tests instead.

## Review Focus

1. **LAUNCH with no linked source/path:** clicking `LAUNCH` when ISO/install is not linked must not crash or attempt to start a process; it should record a clear guarded status/log message (expected: user is told to attach/scan first).
2. **APPLY PROFILE with no linked source:** applying a texture profile while source is unlinked must be a safe stub (no file ops) and surface status/log feedback, not silent failure.
3. **Repeated actions/log growth:** repeated `SCAN/ATTACH` (stub) or `APPLY PROFILE` clicks should keep UI responsive and logs bounded or reasonably managed (no unbounded growth causing sluggishness in normal use).
4. **Missing/invalid future paths or config:** state/model helpers must tolerate empty/unset path fields without panicking (paths are placeholders in Etapa 1 but will be real later).
5. **Small/narrow window resize:** layout (sidebar + main panel, labels/buttons) should remain usable without overlap/panic at smaller window sizes (text may wrap/truncate gracefully).

---
## File Structure

- Create: `Cargo.toml` — workspace root (`resolver = "2"`, member `crates/launcher-gui`).
- Create: `crates/launcher-gui/Cargo.toml` — binary crate package, depends on `eframe = "0.37"`.
- Create: `crates/launcher-gui/src/main.rs` — module declarations and `eframe::run_native` entrypoint configuring window/title.
- Create: `crates/launcher-gui/src/model.rs` — pure domain types/state: view, game/profile records, registry defaults (DFBHD + slots), selection/apply/scan stub transitions and log helper. Unit-tested.
- Create: `crates/launcher-gui/src/theme.rs` — vault theme tokens and `apply_vault_theme(&egui::Context)` visual configuration. Unit-tested for key tokens.
- Create: `crates/launcher-gui/src/app.rs` — `LauncherApp` holding `LauncherState` + current view; `eframe::App::update` renders header/nav/sidebar and Dashboard/DFBHD/Settings screens with stub actions.
- Create (optional, Task 5): `README.md` — short project/Etapa 1 scope and `cargo run -p launcher-gui` instructions.

Responsibility boundaries: `model` has no egui dependency and owns state transitions; `theme` owns visuals only; `app` owns layout/rendering and calls model/theme; `main` owns bootstrapping.

---

### Task 1: Initialize workspace and runnable launcher-gui shell

**Files:**
- Create: `Cargo.toml`
- Create: `crates/launcher-gui/Cargo.toml`
- Create: `crates/launcher-gui/src/main.rs`

**Interfaces:**
- Consumes: none.
- Produces: workspace builds; `crates/launcher-gui` binary target `launcher-gui` (package name) runs a native window via `eframe::run_native` and displays a minimal vault label so later tasks can plug in modules.

- [ ] **Step 1: Create workspace and crate manifests**

Create `Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = ["crates/launcher-gui"]
```

Create `crates/launcher-gui/Cargo.toml`:
```toml
[package]
name = "launcher-gui"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.37"
```

- [ ] **Step 2: Create minimal entrypoint window**

Create `crates/launcher-gui/src/main.rs` with a minimal `eframe` 0.37 app that opens a window titled `MELKY.LAUNCHER // SECURE_VAULT` and renders text `SYSTEM_STATE: READY` (so the app proves runnable). Use `eframe::run_native` returning `eframe::Result<()>` and a creation closure like `Box::new(|_cc| Ok(Box::new(VaultShellApp::default())))`; implement `Default` and `eframe::App` for `VaultShellApp` with `fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) { egui::CentralPanel::default().show(ctx, |ui| { ui.label("SYSTEM_STATE: READY"); }); }`.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p launcher-gui`
Expected: PASS (compiles; dependencies fetched).

Run: `cargo test -p launcher-gui`
Expected: PASS (0 tests).

---

### Task 2: Add domain model for games and texture profiles

**Files:**
- Create: `crates/launcher-gui/src/model.rs`
- Modify: `crates/launcher-gui/src/main.rs` (add `mod model;`)

**Interfaces:**
- Consumes: none (pure Rust).
- Produces: public types/functions usable by `app`: `View` enum (`Dashboard`, `Dfbhd`, `Settings`); `Profile { id: String, name: String }`; `GameNode { id, name, source_linked: bool, source_label: String, profiles: Vec<Profile>, active_profile_id: String, log: Vec<String> }`; `LauncherState { games: Vec<GameNode>, selected_game_idx: usize, view: View }` with `LauncherState::default()` seeding DFBHD (`id "dfbhd"`, profiles `original`/"Original" and `hd_textures`/"HD Textures", active `original`, source unlinked label like `ISO/Install [not linked]`, initial log e.g. `awaiting game link...`) plus placeholder slot nodes; methods `fn selected_game(&self) -> Option<&GameNode>`, `fn selected_game_mut(&mut self) -> Option<&mut GameNode>`, `fn select_profile(&mut self, profile_id: &str)` (no-op if missing), `fn apply_profile_stub(&mut self)` (appends bounded/status log; safe if unlinked), `fn scan_attach_stub(&mut self)` (appends status log; no fs), `fn launch_stub(&mut self)` (guarded: if not linked append clear "attach/scan first" log; never panics). Log helper keeps at most ~100 entries.

- [ ] **Step 1: Write failing unit tests for default state and stub transitions**

In `crates/launcher-gui/src/model.rs` add `#[cfg(test)] mod tests` with tests (exact names/intent):
```rust
use super::*;

#[test]
fn default_state_has_dfbhd_with_profiles() {
    let s = LauncherState::default();
    let g = s.games.iter().find(|g| g.id == "dfbhd").expect("dfbhd");
    assert_eq!(g.name, "Delta Force: Black Hawk Down");
    assert!(g.profiles.iter().any(|p| p.id == "original" && p.name == "Original"));
    assert!(g.profiles.iter().any(|p| p.id == "hd_textures" && p.name == "HD Textures"));
    assert_eq!(g.active_profile_id, "original");
    assert!(!g.source_linked);
}

#[test]
fn select_and_apply_profile_stub_are_safe() {
    let mut s = LauncherState::default();
    s.select_profile("hd_textures");
    assert_eq!(s.selected_game().unwrap().active_profile_id, "hd_textures");
    s.apply_profile_stub();
    assert!(!s.selected_game().unwrap().log.is_empty());
}

#[test]
fn launch_stub_guarded_when_unlinked() {
    let mut s = LauncherState::default();
    let before = s.selected_game().unwrap().log.len();
    s.launch_stub();
    let g = s.selected_game().unwrap();
    assert!(g.log.len() >= before);
    let last = g.log.last().unwrap().to_lowercase();
    assert!(last.contains("attach") || last.contains("scan") || last.contains("link"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p launcher-gui`
Expected: FAIL (missing `model` types/functions or module not found).

- [ ] **Step 3: Implement model types and state transitions**

Implement the structs/enums and methods described in Interfaces. Ensure: `select_profile` ignores unknown ids; `apply_profile_stub`, `scan_attach_stub`, `launch_stub` never panic on empty games/paths and append informative log lines; log is capped (e.g. truncate oldest when > 100). Seed at least 2 placeholder slot nodes (e.g. ids `slot-1`, `slot-2`, marked as empty/available) for multi-game base feel.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p launcher-gui`
Expected: PASS (model tests green).

---

### Task 3: Add vault terminal theme

**Files:**
- Create: `crates/launcher-gui/src/theme.rs`
- Modify: `crates/launcher-gui/src/main.rs` (add `mod theme;`)

**Interfaces:**
- Consumes: `egui` types.
- Produces: `pub struct VaultPalette { pub bg: egui::Color32, pub panel: egui::Color32, pub border: egui::Color32, pub accent: egui::Color32, pub accent_alt: egui::Color32, pub text: egui::Color32, pub muted: egui::Color32 }` with `impl Default for VaultPalette` providing dark vault colors (dark bg, cyan/green accents, light text); `pub fn palette() -> VaultPalette`; `pub fn apply_vault_theme(ctx: &egui::Context)` that sets dark visuals and applies palette to widgets/panels (backgrounds, selection/active accents, stroke/borders) to evoke melky.tech secure-vault/terminal style.

- [ ] **Step 1: Write failing unit tests for theme tokens**

In `crates/launcher-gui/src/theme.rs` add tests:
```rust
use super::*;

#[test]
fn palette_is_dark_with_accents() {
    let p = palette();
    assert!(p.bg.r() < 40 && p.bg.g() < 40 && p.bg.b() < 40);
    assert!(p.accent != p.bg);
    assert!(p.accent_alt != p.bg);
    assert!(p.text.r() > 180 || p.text.g() > 180 || p.text.b() > 180);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p launcher-gui`
Expected: FAIL (missing theme items).

- [ ] **Step 3: Implement palette and apply_vault_theme**

Implement `VaultPalette`/`palette()` with concrete dark + cyan/green accent values and `apply_vault_theme` configuring `egui::Visuals::dark()` then overriding key colors (window/panel fill, extreme bg, selection, widget active/hover strokes/fills) using the palette. Keep it simple and readable.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p launcher-gui`
Expected: PASS.

---

### Task 4: Build app shell, navigation, and screens

**Files:**
- Create: `crates/launcher-gui/src/app.rs`
- Modify: `crates/launcher-gui/src/main.rs` (declare `mod app;`, construct/run `LauncherApp`)

**Interfaces:**
- Consumes: `model::{LauncherState, View}` and `theme::{apply_vault_theme, palette}`.
- Produces: `pub struct LauncherApp { state: LauncherState }` with `impl Default` and `impl eframe::App for LauncherApp` rendering the approved layout: top/header bar with app branding `MELKY.LAUNCHER // SECURE_VAULT` and status indicator (e.g. `STATUS: READY`), navigation for `Dashboard` / `DFBHD` / `Settings`, left sidebar listing game nodes/slots (selectable, highlights DFBHD), and main content per view:
  - **Dashboard:** status overview and nodes list/summary.
  - **DFBHD:** `DFBHD // TEXTURE PROFILES`, source label (`ISO/Install [not linked]` placeholder), profile selection (`Original` / `HD Textures`) reflecting active profile, buttons `SCAN/ATTACH`, `APPLY PROFILE`, `LAUNCH` wired to model stub methods, and a log/status panel showing recent log lines.
  - **Settings:** placeholders (e.g. resolution/theme/app prefs labels).
  Buttons must be guarded via model behavior (no crash on unlinked). Use monospaced/technical styling cues consistent with theme.

- [ ] **Step 1: Implement LauncherApp and UI layout**

Create `app.rs` implementing `LauncherApp` and `eframe::App::update`: call `apply_vault_theme(ctx)` once per frame or on first frame as appropriate, draw header/top panel, left `SidePanel` for `GAMES/NODES`, and `CentralPanel` switching on `state.view`. Wire interactions: nav changes view; clicking game node selects it; profile controls call `select_profile`; buttons call `scan_attach_stub`, `apply_profile_stub`, `launch_stub`. Keep layout resilient to narrower widths (use wrapping/scroll areas like `ScrollArea` for logs/lists as needed).

- [ ] **Step 2: Wire entrypoint to LauncherApp**

Modify `main.rs` to `mod app; mod model; mod theme;` and run `LauncherApp::default()` (remove temporary shell). Configure viewport/title appropriately (e.g. `MELKY.LAUNCHER // SECURE_VAULT`).

- [ ] **Step 3: Verify compile and tests**

Run: `cargo check -p launcher-gui`
Expected: PASS.

Run: `cargo test -p launcher-gui`
Expected: PASS (model and theme tests still green).

Manual check (if GUI available): `cargo run -p launcher-gui` opens the themed window with navigation, DFBHD profiles/log, and stub buttons producing log/status updates without crashes when unlinked.

---

### Task 5: Final verification and Etapa 1 usage notes

**Files:**
- Create/Modify: `README.md`
- (verify existing files)

**Interfaces:**
- Consumes: implemented crate.
- Produces: documented run instructions and clear Etapa 1 scope note for portfolio/users.

- [ ] **Step 1: Run full verification**

Run: `cargo test -p launcher-gui`
Expected: PASS.

Run: `cargo check -p launcher-gui`
Expected: PASS (no errors).

- [ ] **Step 2: Add README**

Create `README.md` describing: project purpose (DFBHD texture/mod launcher GUI, personal/portfolio, melky.tech-inspired secure-vault theme, base for multiple games), Etapa 1 scope (UI shell + stubs; no real ISO/CAB extraction/patching/texture processing yet), prerequisites (Rust stable), and how to run (`cargo run -p launcher-gui`). Mention ISO present in workspace as context and BYOG intent.

- [ ] **Step 3: Final check**

Run: `cargo test -p launcher-gui`
Expected: PASS.

---

## Self-Review Notes

- **Spec coverage:** Etapa 1 goals covered (runnable eframe/egui app, vault theme, Dashboard/DFBHD/Settings, DFBHD + placeholder slots registry, `Original`/`HD Textures` stub profiles, `SCAN/ATTACH`/`APPLY PROFILE`/`LAUNCH` stubs, log/status, source placeholder unlinked). Explicit non-goals respected (no ISO/CAB, patching, wrappers, PFF/texture ops, network). Multi-game base via data-driven registry.
- **Placeholders/type consistency:** `LauncherState`, `GameNode`, `Profile`, `View`, `VaultPalette`, `apply_vault_theme`, `LauncherApp` defined before/within consuming tasks; method names consistent (`select_profile`, `apply_profile_stub`, `scan_attach_stub`, `launch_stub`).
- **Review Focus:** each item mapped to model behavior/UI guards (guarded launch/apply when unlinked, bounded logs, empty-path safety, resizable layout) and should be validated during Task 4/5 verification.
- **Proportion:** steps favor signatures, test intent, and commands over full UI code bodies.
