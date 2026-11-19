# DFBHD Texture Launcher (Etapa 2)

A personal/portfolio native GUI launcher foundation for **Delta Force: Black Hawk Down (DFBHD)** textures and mods, designed as a base that can grow to support multiple games. The visual identity is a **melky.tech-inspired** dark **"secure vault / terminal"** aesthetic.

## Etapa 2 scope (current)

This repository currently delivers the Etapa 1 UI shell plus **Etapa 2 (Option A) link-and-launch** support:

- Runnable Rust desktop app using `eframe/egui` (`launcher-gui`) with the existing vault/terminal-themed layout (header, navigation, status indicators) and screens: **Dashboard**, **DFBHD**, **Settings**
- Data-driven game/node registry with DFBHD as the first entry and placeholder slots for future games
- Profile concepts (`Original`, `HD Textures`) with stub selection/apply behavior (no real texture/profile application)
- **Attach existing install folder (SCAN/ATTACH):** native folder picker to select an installed DFBHD folder; invalid selections do not clear a previously valid link
- **Install detection:** locates `dfbhd.exe` in the selected folder root (case-insensitive) and recursively enumerates `.pff` assets (case-insensitive)
- **Linked install summary:** DFBHD view shows linked/unlinked status, install path, `dfbhd.exe` detection result, and `.pff` count with representative names; supports re-attaching a different valid folder
- **Config persistence:** per-game link data (`source_linked`, `install_dir`, `exe_path`, `pff_paths`) saved to `launcher-config.json` (next to the launcher executable when available, otherwise the current working directory). Config is loaded on startup and saved on successful link changes; missing/corrupt config falls back to defaults without panic
- **Real launch (guarded):** when linked and the executable exists, `LAUNCH` starts the game via the detected executable with the install directory as working directory (non-blocking) and logs the outcome. When not linked or the executable is missing/stale, launch is guarded with clear attach/link or missing-executable guidance

**Not implemented yet (remaining non-goals):**

- No ISO mounting or ISO reading
- No InstallShield CAB extraction
- No binary patching or wrappers (e.g. DXVK/d3d8to9/DSOAL)
- No PFF parsing, texture extraction, texture processing/upscaling, or texture application
- No network access or manifest/download functionality
- No real profile application pipeline (APPLY PROFILE remains a stub)

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
