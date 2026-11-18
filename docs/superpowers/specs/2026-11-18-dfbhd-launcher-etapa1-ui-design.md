# DFBHD Texture Launcher — Etapa 1 (Fundação/UI) Design

**Date:** 2026-11-18  
**Status:** Approved by user in conversation  
**Project:** Personal/portfolio. GUI launcher for Delta Force: Black Hawk Down (DFBHD) textures/mods, intended as a reusable design base for multiple games. Visual identity inspired by [melky.tech](https://melky.tech) (dark "secure vault/terminal" aesthetic).

## Context
- Workspace contains retail ISO: `Novalogic_DeltaForceBllackHawkDown.iso` (~684 MB).
- ISO contents (recon): `DFBHD/RESOURCE.PFF` (~344 MB), `DATA1.CAB`, `DATA2.CAB` (~285 MB, likely installed binaries/assets), `SETUP.EXE/INX`, `DFBHDLC.EXE`, `.BIK` videos, manuals. `dfbhd.exe` not exposed directly; likely installed/extracted from CABs.
- Long-term direction discussed: (B) full DFBHD pipeline (PFF extract/upscale/pack) + launcher, (A) polished installer/applicator, (C) generalize to multi-game base — to be approached step by step.

## Intent / Why
Create a visually distinctive, portfolio-quality native launcher foundation that can grow into a DFBHD texture/mod manager and later support other games, reflecting the melky.tech "secure vault/terminal" identity.

## Goals (Etapa 1)
- Deliver a runnable Rust GUI app (`eframe/egui`) with the vault/terminal look-and-feel.
- Provide core screens and navigation: Dashboard, DFBHD game/profile view, Settings.
- Model DFBHD as the first entry in a simple multi-game-ready registry (data-driven), with profile concepts (`Original`, `HD Textures`) and stub actions.
- Keep structure evolvable toward the broader architecture described in the project spec (manifest-driven install, patching, texture pipeline) without implementing it yet.

## Non-goals (Etapa 1)
- No ISO mounting/CAB extraction, no game detection/install logic against real files.
- No binary patching, wrappers (DXVK/d3d8to9/DSOAL), LAA, or config generation.
- No PFF parsing, texture extraction, AI upscale, or real texture application.
- No network/manifest download, no real game launch required (can be stubbed/disabled with clear status).

## Constraints
- Windows target environment; BYOG/legal-copy model respected (ISO present locally).
- Workspace not currently a git repo.
- Keep implementation small, clear, and extensible (portfolio code quality).

## Approach (chosen)
**Etapa 1 = UI/structure first** (recommended and approved): build the themed shell and domain stubs so later etapas can plug in ISO/CAB handling (Etapa 2) and texture pipeline/application (Etapa 3).

## Design Decisions
- **Tech:** Rust stable, single native binary feel. `eframe/egui` for GUI. Start as a small Cargo workspace or single crate with clear modules (`ui`/`theme`, `games`, `profiles`, `app_state`) to allow later split into crates (e.g. manifest/installer/pff) per the reference spec.
- **Visual theme ("vault/terminal"):** dark background, subtle bordered panels/cards, cyan/green accent highlights, monospaced/technical labels and status text (e.g. `SYSTEM_STATE`, `NODE`, `PROFILE`, `STATUS`), header with app/status, sidebar navigation.
- **Screens/layout:**
  - **Dashboard:** overview/status, game nodes/slots list (DFBHD + placeholders).
  - **DFBHD:** game info, source link placeholders (ISO/install path not linked), profile selector (`Original`, `HD Textures` stub), actions `SCAN/ATTACH`, `APPLY PROFILE` (stub), `LAUNCH` (stub/guarded), log/status panel.
  - **Settings:** placeholders (theme/resolution/app prefs).
- **Multi-game base:** in-memory/config-ready registry describing games and profiles; DFBHD as first entry. No complex plugin system yet.

**Text sketch (approved):**
```text
┌─ MELKY.LAUNCHER // SECURE_VAULT ────────────── [STATUS: READY] ─┐
│ [Dashboard] [DFBHD] [Settings]                                  │
├───────────────┬─────────────────────────────────────────────────┤
│ GAMES/NODES   │ DFBHD // TEXTURE PROFILES                       │
│ > DFBHD       │ Source: ISO/Install [not linked]                │
│   (slot)      │ Profiles: (•) Original  ( ) HD Textures [stub]  │
│   (slot)      │ [SCAN/ATTACH] [APPLY PROFILE]  [LAUNCH]         │
│               │ Log: awaiting game link...                      │
└───────────────┴─────────────────────────────────────────────────┘
```

## What's left to later etapas
- **Etapa 2:** ISO/CAB extraction or installed-game detection; locate `dfbhd.exe`, `.pff`, relevant paths.
- **Etapa 3:** texture extraction/management from `.pff`, Original↔HD profile application, packaging/manifest.
- **Later:** full pipeline (AI upscale), wrappers/patching per profiles, generalized multi-game plugin/manifest architecture.
