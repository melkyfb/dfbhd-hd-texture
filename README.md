# DFBHD Texture Launcher (Etapa 2 + 3a)

A personal/portfolio native GUI launcher foundation for **Delta Force: Black Hawk Down (DFBHD)** textures and mods, designed as a base that can grow to support multiple games. The visual identity is a **melky.tech-inspired** dark **"secure vault / terminal"** aesthetic.

## Current scope (Etapa 2 + 3a)

This repository currently delivers the Etapa 1 UI shell plus **Etapa 2 (Option A) link-and-launch** support and **Etapa 3a DFBHD profile apply** for menu textures:

- Runnable Rust desktop app using `eframe/egui` (`launcher-gui`) with the existing vault/terminal-themed layout (header, navigation, status indicators) and screens: **Dashboard**, **DFBHD**, **Settings**
- Data-driven game/node registry with DFBHD as the first entry and placeholder slots for future games
- DFBHD profile selection (`Original`, `HD Textures`, `HD Widescreen (1080p)`) with **explicit APPLY PROFILE** behavior (no auto-swap on selection): `Original` and `HD Textures` swap/restore prebuilt `localres.pff` menu textures (`BHD_main.tga`, `Btn_exit.tga`) using the bundled pack at `assets/profiles/dfbhd/hd_textures/localres.pff`, with backup/restore when the install is linked; `HD Widescreen (1080p)` is the combined action that applies the HD textures **and** installs/configures the bundled widescreen wrapper (see below)
- **DFBHD HD Widescreen (1080p) wrapper:** the combined profile copies the bundled runtime files from `assets/wrapper/dfbhd_widescreen/` (`d3d8.dll`, `d3d8.ini`, `dinput8.dll`, `scripts/BHD_Asi.asi`, `scripts/BHD_Asi.ini`) into the linked install; the bundled `scripts/BHD_Asi.ini` defaults to 1920x1080 resolution and FOV 90. If a managed path already exists in the install, the pre-existing file is saved once as `<path>.launcherbak` so it can be restored. Applying `Original` restores managed files from `.launcherbak` backups and removes managed files without backups (an empty managed `scripts/` dir is also removed). Only managed wrapper paths are touched — `dfbhd.exe` and unrelated install files are never modified. Texture-only `HD Textures` never installs or removes wrapper files
- **Attach existing install folder (SCAN/ATTACH):** native folder picker to select an installed DFBHD folder; invalid selections do not clear a previously valid link
- **Install detection:** locates `dfbhd.exe` in the selected folder root (case-insensitive) and recursively enumerates `.pff` assets (case-insensitive)
- **Linked install summary:** DFBHD view shows linked/unlinked status, install path, `dfbhd.exe` detection result, and `.pff` count with representative names; supports re-attaching a different valid folder
- **Config persistence:** per-game link data (`source_linked`, `install_dir`, `exe_path`, `pff_paths`, `active_profile_id`) saved to `launcher-config.json` (next to the launcher executable when available, otherwise the current working directory). Config is loaded on startup and saved on successful link changes and after apply attempts; missing/corrupt config falls back to defaults without panic
- **Real launch (guarded):** when linked and the executable exists, `LAUNCH` starts the game via the detected executable with the install directory as working directory (non-blocking) and logs the outcome. When not linked or the executable is missing/stale, launch is guarded with clear attach/link or missing-executable guidance

**Not implemented yet (remaining non-goals):**

- No runtime upscaling or DDS pipeline
- No ISO mounting/reading or InstallShield CAB extraction
- No other PFFs beyond the scoped DFBHD `localres.pff` menu-texture swap/restore
- No user-supplied pack config [B later]
- No generic DXVK/d3d8to9/DSOAL-style wrappers for other games (the bundled DFBHD 1080p widescreen wrapper is separate and documented above)
- No network access or manifest/download functionality

Future etapas may build on this foundation (e.g. ISO/InstallShield handling, patching/wrapper support, texture pipeline and real profile application, and broader multi-game/manifest architecture).

## BYOG / local context

This project follows a **bring-your-own-game (BYOG)** intent: you must use your own legally obtained copy of the game. A local retail ISO (`Novalogic_DeltaForceBllackHawkDown.iso`) is present in this workspace as development context only; Etapa 2 does **not** read, mount, extract, or modify it.

## Prerequisites

- **Rust (stable)** toolchain with Cargo installed ([rustup.rs](https://rustup.rs/))

## How to run

From the repository root:

```bash
cargo run -p launcher-gui
```

## Verification

```bash
cargo test -p launcher-gui
cargo check -p launcher-gui
```
