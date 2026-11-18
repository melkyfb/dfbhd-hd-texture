# DFBHD Texture Launcher (Etapa 1)

A personal/portfolio native GUI launcher foundation for **Delta Force: Black Hawk Down (DFBHD)** textures and mods, designed as a base that can grow to support multiple games. The visual identity is a **melky.tech-inspired** dark **"secure vault / terminal"** aesthetic.

## Etapa 1 scope (current)

This repository currently delivers the **Etapa 1 UI shell and domain stubs only**:

- Runnable Rust desktop app using `eframe/egui` (`launcher-gui`)
- Vault/terminal-themed layout with header, navigation, and status indicators
- Screens: **Dashboard**, **DFBHD** (texture profiles), **Settings**
- Data-driven game/node registry with DFBHD as the first entry and placeholder slots for future games
- Profile concepts (`Original`, `HD Textures`) with stub selection/apply behavior
- Stub actions: `SCAN/ATTACH`, `APPLY PROFILE`, `LAUNCH` (guarded; no real game launch)
- Log/status panel for UI feedback

**Not implemented yet (by design, Etapa 1):**

- No real ISO mounting or CAB extraction
- No real game detection/install logic, binary patching, or wrappers
- No PFF parsing, texture extraction, texture processing/upscaling, or texture application
- No network access or manifest/download functionality
- No real profile application or launch pipeline

Future etapas are expected to build on this foundation (e.g. ISO/CAB/installed-game handling, texture pipeline and profile application, and broader multi-game/manifest architecture).

## BYOG / local context

This project follows a **bring-your-own-game (BYOG)** intent: you must use your own legally obtained copy of the game. A local retail ISO (`Novalogic_DeltaForceBllackHawkDown.iso`) is present in this workspace as development context only; Etapa 1 does **not** read, mount, extract, or modify it.

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
