use std::path::Path;
use std::process::Command;

use crate::config::{GameLinkConfig, LauncherConfig};
use crate::detect::scan_install_dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Dfbhd,
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameNode {
    pub id: String,
    pub name: String,
    pub source_linked: bool,
    pub source_label: String,
    pub install_dir: Option<String>,
    pub exe_path: Option<String>,
    pub pff_paths: Vec<String>,
    pub profiles: Vec<Profile>,
    pub active_profile_id: String,
    pub log: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherState {
    pub games: Vec<GameNode>,
    pub selected_game_idx: usize,
    pub view: View,
}

const LOG_CAP: usize = 100;

pub(crate) fn push_log(log: &mut Vec<String>, entry: impl Into<String>) {
    log.push(entry.into());
    if log.len() > LOG_CAP {
        let overflow = log.len() - LOG_CAP;
        log.drain(0..overflow);
    }
}

pub fn resolve_dfbhd_hd_localres_pack_path() -> Option<std::path::PathBuf> {
    let mut bases: Vec<std::path::PathBuf> = Vec::new();
    if let Some(base) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        bases.push(base);
    }
    if let Some(base) = std::env::current_dir().ok() {
        bases.push(base);
    }
    for base in bases {
        let candidate = base.join("assets/profiles/dfbhd/hd_textures/localres.pff");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn resolve_dfbhd_hd_resource_pack_path() -> Option<std::path::PathBuf> {
    let mut bases: Vec<std::path::PathBuf> = Vec::new();
    if let Some(base) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        bases.push(base);
    }
    if let Some(base) = std::env::current_dir().ok() {
        bases.push(base);
    }
    resolve_dfbhd_hd_resource_pack_path_from_bases(bases)
}

pub(crate) fn resolve_dfbhd_hd_resource_pack_path_from_bases<
    I: IntoIterator<Item = std::path::PathBuf>,
>(
    bases: I,
) -> Option<std::path::PathBuf> {
    for base in bases {
        let candidate = base.join("assets/profiles/dfbhd/hd_textures/resource.pff");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub fn resolve_dfbhd_wrapper_bundle_dir() -> Option<std::path::PathBuf> {
    let mut bases: Vec<std::path::PathBuf> = Vec::new();
    if let Some(base) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        bases.push(base);
    }
    if let Some(base) = std::env::current_dir().ok() {
        bases.push(base);
    }
    resolve_dfbhd_wrapper_bundle_dir_from_bases(bases)
}

pub(crate) fn resolve_dfbhd_wrapper_bundle_dir_from_bases<I>(bases: I) -> Option<std::path::PathBuf>
where
    I: IntoIterator<Item = std::path::PathBuf>,
{
    for base in bases {
        let candidate = base.join("assets/wrapper/dfbhd_widescreen");
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

pub(crate) const DFBHD_WRAPPER_MANAGED_FILES: &[&str] = &[
    "d3d8.dll",
    "d3d8.ini",
    "dinput8.dll",
    "scripts/BHD_Asi.asi",
    "scripts/BHD_Asi.ini",
];

pub(crate) fn install_dfbhd_wrapper_to_install(
    install_dir: &Path,
    bundle_dir: &Path,
    log: &mut Vec<String>,
) -> Result<(), String> {
    for rel in DFBHD_WRAPPER_MANAGED_FILES {
        let src = bundle_dir.join(rel);
        let dest = install_dir.join(rel);

        if !src.is_file() {
            let err = format!(
                "wrapper install failed: missing bundle file '{}' (expected '{}')",
                rel,
                src.display()
            );
            push_log(log, err.clone());
            return Err(err);
        }

        if let Some(parent) = dest.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                let err = format!(
                    "wrapper install failed: could not create directory '{}': {}",
                    parent.display(),
                    e
                );
                push_log(log, err.clone());
                return Err(err);
            }
        }

        let mut bak_os = dest.clone().into_os_string();
        bak_os.push(".launcherbak");
        let bak = std::path::PathBuf::from(bak_os);

        if dest.exists() {
            if bak.exists() {
                push_log(
                    log,
                    format!(
                        "wrapper backup already present for '{}'; keeping existing '{}'",
                        dest.display(),
                        bak.display()
                    ),
                );
            } else {
                match std::fs::copy(&dest, &bak) {
                    Ok(_) => push_log(
                        log,
                        format!(
                            "wrapper backup created: '{}' from '{}'",
                            bak.display(),
                            dest.display()
                        ),
                    ),
                    Err(e) => {
                        let err = format!(
                            "wrapper install failed: could not backup '{}' to '{}': {}",
                            dest.display(),
                            bak.display(),
                            e
                        );
                        push_log(log, err.clone());
                        return Err(err);
                    }
                }
            }
        }

        match std::fs::copy(&src, &dest) {
            Ok(_) => push_log(
                log,
                format!("wrapper installed: '{}' (from bundle)", dest.display()),
            ),
            Err(e) => {
                let err = format!(
                    "wrapper install failed: could not copy '{}' to '{}': {}",
                    src.display(),
                    dest.display(),
                    e
                );
                push_log(log, err.clone());
                return Err(err);
            }
        }
    }
    Ok(())
}

/// Reverse/cleanup of the wrapper files installed by
/// [`install_dfbhd_wrapper_to_install`]. For each managed path, if a
/// `<path>.launcherbak` exists it is restored to `<path>` and the backup is
/// removed; otherwise the managed file is removed when present. Best effort,
/// but IO failures that prevent correct restore are logged and returned.
pub(crate) fn restore_dfbhd_wrapper_in_install(
    install_dir: &Path,
    log: &mut Vec<String>,
) -> Result<(), String> {
    for rel in DFBHD_WRAPPER_MANAGED_FILES {
        let dest = install_dir.join(rel);
        let mut bak_os = dest.clone().into_os_string();
        bak_os.push(".launcherbak");
        let bak = std::path::PathBuf::from(bak_os);

        if bak.exists() {
            match std::fs::copy(&bak, &dest) {
                Ok(_) => push_log(
                    log,
                    format!("wrapper restored from backup: '{}'", dest.display()),
                ),
                Err(e) => {
                    let err = format!(
                        "wrapper restore failed: could not restore '{}' from backup '{}': {}",
                        dest.display(),
                        bak.display(),
                        e
                    );
                    push_log(log, err.clone());
                    return Err(err);
                }
            }
            if let Err(e) = std::fs::remove_file(&bak) {
                let err = format!(
                    "wrapper restore failed: could not remove backup '{}': {}",
                    bak.display(),
                    e
                );
                push_log(log, err.clone());
                return Err(err);
            }
            push_log(
                log,
                format!("wrapper backup removed: '{}'", bak.display()),
            );
        } else if dest.exists() {
            match std::fs::remove_file(&dest) {
                Ok(_) => push_log(
                    log,
                    format!(
                        "wrapper removed: '{}' (no backup present)",
                        dest.display()
                    ),
                ),
                Err(e) => {
                    let err = format!(
                        "wrapper restore failed: could not remove '{}': {}",
                        dest.display(),
                        e
                    );
                    push_log(log, err.clone());
                    return Err(err);
                }
            }
        }
    }

    // Remove the managed scripts dir only if it exists and is empty; never
    // touch a non-empty dir (may hold unrelated user files) or other paths.
    let scripts_dir = install_dir.join("scripts");
    if scripts_dir.is_dir() {
        match std::fs::read_dir(&scripts_dir) {
            Ok(mut entries) => {
                if entries.next().is_none() {
                    if let Err(e) = std::fs::remove_dir(&scripts_dir) {
                        push_log(
                            log,
                            format!(
                                "wrapper cleanup warning: could not remove empty '{}': {}",
                                scripts_dir.display(),
                                e
                            ),
                        );
                    } else {
                        push_log(
                            log,
                            format!(
                                "wrapper cleanup: removed empty '{}'",
                                scripts_dir.display()
                            ),
                        );
                    }
                } else {
                    push_log(
                        log,
                        format!(
                            "wrapper cleanup: '{}' not empty; left in place",
                            scripts_dir.display()
                        ),
                    );
                }
            }
            Err(e) => {
                let err = format!(
                    "wrapper restore failed: could not read '{}': {}",
                    scripts_dir.display(),
                    e
                );
                push_log(log, err.clone());
                return Err(err);
            }
        }
    }

    Ok(())
}

fn unlinked_source_label(game_id: &str) -> String {
    match game_id {
        "dfbhd" => "ISO/Install [not linked]".to_string(),
        _ => "[empty / available]".to_string(),
    }
}

pub(crate) fn pff_names_hint(pff_paths: &[String], max_names: usize) -> String {
    let names: Vec<String> = pff_paths
        .iter()
        .filter_map(|p| Path::new(p).file_name().and_then(|n| n.to_str()))
        .map(|s| s.to_string())
        .collect();
    if names.is_empty() {
        return "none".to_string();
    }
    if names.len() <= max_names {
        return names.join(", ");
    }
    let shown = names[..max_names].join(", ");
    format!("{shown}, ... (+{})", names.len() - max_names)
}

impl Default for LauncherState {
    fn default() -> Self {
        let dfbhd = GameNode {
            id: "dfbhd".to_string(),
            name: "Delta Force: Black Hawk Down".to_string(),
            source_linked: false,
            source_label: unlinked_source_label("dfbhd"),
            install_dir: None,
            exe_path: None,
            pff_paths: Vec::new(),
            profiles: vec![
                Profile {
                    id: "original".to_string(),
                    name: "Original".to_string(),
                },
                Profile {
                    id: "hd_textures".to_string(),
                    name: "HD Textures".to_string(),
                },
                Profile {
                    id: "hd_widescreen_1080p".to_string(),
                    name: "HD Widescreen (1080p)".to_string(),
                },
            ],
            active_profile_id: "original".to_string(),
            log: vec!["awaiting game link...".to_string()],
        };

        let slot1 = GameNode {
            id: "slot-1".to_string(),
            name: "Empty Slot".to_string(),
            source_linked: false,
            source_label: unlinked_source_label("slot-1"),
            install_dir: None,
            exe_path: None,
            pff_paths: Vec::new(),
            profiles: Vec::new(),
            active_profile_id: String::new(),
            log: Vec::new(),
        };

        let slot2 = GameNode {
            id: "slot-2".to_string(),
            name: "Empty Slot".to_string(),
            source_linked: false,
            source_label: unlinked_source_label("slot-2"),
            install_dir: None,
            exe_path: None,
            pff_paths: Vec::new(),
            profiles: Vec::new(),
            active_profile_id: String::new(),
            log: Vec::new(),
        };

        Self {
            games: vec![dfbhd, slot1, slot2],
            selected_game_idx: 0,
            view: View::Dashboard,
        }
    }
}

impl LauncherState {
    pub fn selected_game(&self) -> Option<&GameNode> {
        self.games.get(self.selected_game_idx)
    }

    pub fn selected_game_mut(&mut self) -> Option<&mut GameNode> {
        self.games.get_mut(self.selected_game_idx)
    }

    pub fn select_profile(&mut self, profile_id: &str) {
        let Some(game) = self.selected_game_mut() else {
            return;
        };
        if game.profiles.iter().any(|p| p.id == profile_id) {
            game.active_profile_id = profile_id.to_string();
        }
    }

    #[allow(dead_code)]
    pub fn apply_profile_stub(&mut self) {
        let Some(id) = self.selected_game().map(|g| g.id.clone()) else {
            return;
        };
        self.apply_active_profile_for_game(&id);
    }

    #[allow(dead_code)]
    pub fn scan_attach_stub(&mut self) {
        let Some(game) = self.selected_game_mut() else {
            return;
        };
        push_log(
            &mut game.log,
            "scan/attach stub: no filesystem scan performed; source not linked".to_string(),
        );
    }

    #[allow(dead_code)]
    pub fn launch_stub(&mut self) {
        let Some(id) = self.selected_game().map(|g| g.id.clone()) else {
            return;
        };
        self.launch_game(&id);
    }

    pub fn apply_loaded_config(&mut self, cfg: &LauncherConfig) {
        for game in &mut self.games {
            let Some(saved) = cfg.games.get(&game.id) else {
                continue;
            };
            game.source_linked = saved.source_linked;
            game.install_dir = saved.install_dir.clone();
            game.exe_path = saved.exe_path.clone();
            game.pff_paths = saved.pff_paths.clone();
            if let Some(id) = saved.active_profile_id.as_ref() {
                if !id.is_empty() && game.profiles.iter().any(|p| p.id == *id) {
                    game.active_profile_id = id.clone();
                }
            }
            if saved.source_linked {
                game.source_label = saved
                    .install_dir
                    .clone()
                    .or_else(|| saved.exe_path.clone())
                    .unwrap_or_else(|| "linked install".to_string());
                push_log(
                    &mut game.log,
                    format!(
                        "loaded saved link config: {}",
                        game.source_label
                    ),
                );
            } else {
                game.source_label = unlinked_source_label(&game.id);
                push_log(&mut game.log, "loaded saved config: source unlinked".to_string());
            }
        }
    }

    pub fn export_config(&self) -> LauncherConfig {
        let mut cfg = LauncherConfig::default();
        for game in &self.games {
            cfg.games.insert(
                game.id.clone(),
                GameLinkConfig {
                    source_linked: game.source_linked,
                    install_dir: game.install_dir.clone(),
                    exe_path: game.exe_path.clone(),
                    pff_paths: game.pff_paths.clone(),
                    active_profile_id: if game.active_profile_id.is_empty() {
                        None
                    } else {
                        Some(game.active_profile_id.clone())
                    },
                },
            );
        }
        cfg
    }

    pub fn link_game_folder(&mut self, game_id: &str, selected_dir: &str) -> Result<(), String> {
        let Some(idx) = self.games.iter().position(|g| g.id == game_id) else {
            return Err(format!("unknown game id: {game_id}"));
        };
        let result = scan_install_dir(Path::new(selected_dir));
        match result {
            Ok(detected) => {
                let game = &mut self.games[idx];
                game.source_linked = true;
                game.install_dir = Some(detected.install_dir.clone());
                game.exe_path = Some(detected.exe_path.clone());
                game.pff_paths = detected.pff_paths.clone();
                game.source_label = detected.install_dir.clone();
                let names_hint = pff_names_hint(&detected.pff_paths, 3);
                push_log(
                    &mut game.log,
                    format!(
                        "linked install: {} (dfbhd.exe detected: {}; .pff files: {} [{}])",
                        detected.install_dir,
                        detected.exe_path,
                        detected.pff_paths.len(),
                        names_hint
                    ),
                );
                Ok(())
            }
            Err(err) => {
                let game = &mut self.games[idx];
                push_log(
                    &mut game.log,
                    format!("scan/attach failed for '{}': {}", selected_dir, err),
                );
                Err(err)
            }
        }
    }

    pub fn launch_game(&mut self, game_id: &str) {
        let Some(idx) = self.games.iter().position(|g| g.id == game_id) else {
            return;
        };

        let (source_linked, install_dir, exe_path) = {
            let game = &self.games[idx];
            (
                game.source_linked,
                game.install_dir.clone(),
                game.exe_path.clone(),
            )
        };

        if !source_linked {
            push_log(
                &mut self.games[idx].log,
                "launch blocked: attach/scan and link game source first".to_string(),
            );
            return;
        }

        let Some(exe) = exe_path else {
            push_log(
                &mut self.games[idx].log,
                "launch blocked: missing executable path; re-attach/scan and link the game".to_string(),
            );
            return;
        };

        let exe_path_buf = Path::new(&exe);
        if !exe_path_buf.exists() {
            push_log(
                &mut self.games[idx].log,
                format!(
                    "launch blocked: missing executable '{}'; re-attach/scan and link the game",
                    exe
                ),
            );
            return;
        }

        let cwd = install_dir
            .clone()
            .or_else(|| {
                exe_path_buf
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| ".".to_string());

        match Command::new(&exe).current_dir(&cwd).spawn() {
            Ok(child) => {
                push_log(
                    &mut self.games[idx].log,
                    format!(
                        "launch started: '{}' (cwd: '{}', pid: {})",
                        exe,
                        cwd,
                        child.id()
                    ),
                );
            }
            Err(e) => {
                push_log(
                    &mut self.games[idx].log,
                    format!("launch failed: '{}' (cwd: '{}'): {}", exe, cwd, e),
                );
            }
        }
    }

    fn restore_dfbhd_pff_asset_original(
        &mut self,
        idx: usize,
        install: &Path,
        asset_name: &str,
        missing_backup_is_ok: bool,
    ) -> bool {
        let target = install.join(asset_name);
        let original_backup = install.join(format!("{asset_name}.original"));
        let bak = install.join(format!("{asset_name}.bak"));

        let source = if original_backup.exists() {
            Some((original_backup.clone(), format!("{asset_name}.original")))
        } else if bak.exists() {
            Some((bak.clone(), format!("{asset_name}.bak")))
        } else {
            None
        };

        match source {
            Some((src, label)) => match std::fs::copy(&src, &target) {
                Ok(_) => {
                    push_log(
                        &mut self.games[idx].log,
                        format!(
                            "restored original {} from '{}' to '{}'",
                            asset_name,
                            src.display(),
                            target.display()
                        ),
                    );
                    true
                }
                Err(e) => {
                    push_log(
                        &mut self.games[idx].log,
                        format!(
                            "profile apply failed: could not restore original {} from '{}': {}",
                            asset_name, label, e
                        ),
                    );
                    false
                }
            },
            None => {
                if missing_backup_is_ok {
                    push_log(
                        &mut self.games[idx].log,
                        format!(
                            "profile apply: no {} original backup found to restore (expected {}.original or {}.bak); continuing",
                            asset_name, asset_name, asset_name
                        ),
                    );
                    true
                } else {
                    push_log(
                        &mut self.games[idx].log,
                        format!(
                            "profile apply failed: no original backup found to restore {} (expected {}.original or {}.bak)",
                            asset_name, asset_name, asset_name
                        ),
                    );
                    false
                }
            }
        }
    }

    fn apply_dfbhd_pff_asset_hd(
        &mut self,
        idx: usize,
        install: &Path,
        asset_name: &str,
        hd_pack_path: &str,
    ) -> bool {
        let pack = Path::new(hd_pack_path);
        if !pack.is_file() {
            if asset_name == "localres.pff" {
                push_log(
                    &mut self.games[idx].log,
                    format!(
                        "profile apply failed: HD textures pack not found at '{}'",
                        hd_pack_path
                    ),
                );
            } else {
                push_log(
                    &mut self.games[idx].log,
                    format!(
                        "profile apply failed: HD textures {} pack not found at '{}'",
                        asset_name, hd_pack_path
                    ),
                );
            }
            return false;
        }

        let target = install.join(asset_name);
        let original_backup = install.join(format!("{asset_name}.original"));
        let bak = install.join(format!("{asset_name}.bak"));

        if !original_backup.exists() {
            let backup_source = if bak.exists() {
                Some((bak.clone(), format!("{asset_name}.bak")))
            } else if target.exists() {
                Some((target.clone(), asset_name.to_string()))
            } else {
                None
            };
            if let Some((src, label)) = backup_source {
                match std::fs::copy(&src, &original_backup) {
                    Ok(_) => push_log(
                        &mut self.games[idx].log,
                        format!(
                            "created original backup '{}' from '{}'",
                            original_backup.display(),
                            label
                        ),
                    ),
                    Err(e) => {
                        push_log(
                            &mut self.games[idx].log,
                            format!(
                                "profile apply failed: could not create original backup for {} from '{}': {}",
                                asset_name, label, e
                            ),
                        );
                        return false;
                    }
                }
            } else {
                push_log(
                    &mut self.games[idx].log,
                    format!(
                        "profile apply failed: cannot apply HD textures because no {}, {}.bak, or {}.original was found to preserve as original",
                        asset_name, asset_name, asset_name
                    ),
                );
                return false;
            }
        }

        match std::fs::copy(pack, &target) {
            Ok(_) => {
                push_log(
                    &mut self.games[idx].log,
                    format!(
                        "applied HD textures: copied '{}' to '{}'",
                        pack.display(),
                        target.display()
                    ),
                );
                true
            }
            Err(e) => {
                push_log(
                    &mut self.games[idx].log,
                    format!(
                        "profile apply failed: could not copy HD textures pack for {} to '{}': {}",
                        asset_name,
                        target.display(),
                        e
                    ),
                );
                false
            }
        }
    }

    /// Returns `true` when the texture step succeeded; `false` when it was
    /// skipped or failed (failure is logged via the game log). Compatibility
    /// wrapper: manages `localres.pff` only (no `resource.pff` pack).
    pub fn apply_dfbhd_profile_with_pack_path(
        &mut self,
        profile_id: &str,
        install_dir: &str,
        hd_pack_path: &str,
    ) -> bool {
        self.apply_dfbhd_profile_with_pack_paths(profile_id, install_dir, hd_pack_path, None)
    }

    /// Core texture apply/restore for `original`/`hd_textures` managing both
    /// `localres.pff` and (when applicable) `resource.pff` with `*.original`
    /// backups (one-time, never overwritten), clear per-asset logs, and wrapper
    /// cleanup on `original`.
    pub fn apply_dfbhd_profile_with_pack_paths(
        &mut self,
        profile_id: &str,
        install_dir: &str,
        hd_localres_pack_path: &str,
        hd_resource_pack_path: Option<&str>,
    ) -> bool {
        let Some(idx) = self.games.iter().position(|g| g.id == "dfbhd") else {
            return false;
        };
        let install = Path::new(install_dir);

        match profile_id {
            "original" => {
                let localres_ok = self.restore_dfbhd_pff_asset_original(
                    idx,
                    install,
                    "localres.pff",
                    false,
                );
                let resource_ok = self.restore_dfbhd_pff_asset_original(
                    idx,
                    install,
                    "resource.pff",
                    true,
                );
                // Idempotent wrapper cleanup: runs even when texture restore
                // could not, so the install returns to a non-wrapper state when
                // possible.
                let wrapper_ok =
                    match restore_dfbhd_wrapper_in_install(install, &mut self.games[idx].log) {
                        Ok(()) => {
                            push_log(
                                &mut self.games[idx].log,
                                "profile apply: wrapper cleanup done".to_string(),
                            );
                            true
                        }
                        Err(e) => {
                            push_log(
                                &mut self.games[idx].log,
                                format!("profile apply: wrapper cleanup failed: {e}"),
                            );
                            false
                        }
                    };
                localres_ok && resource_ok && wrapper_ok
            }
            "hd_textures" => {
                let localres_ok = self.apply_dfbhd_pff_asset_hd(
                    idx,
                    install,
                    "localres.pff",
                    hd_localres_pack_path,
                );
                let resource_ok = match hd_resource_pack_path {
                    Some(rpack) => {
                        self.apply_dfbhd_pff_asset_hd(idx, install, "resource.pff", rpack)
                    }
                    None => true,
                };
                localres_ok && resource_ok
            }
            _ => {
                push_log(
                    &mut self.games[idx].log,
                    format!("profile apply failed: unknown profile '{}'", profile_id),
                );
                false
            }
        }
    }

    /// Combined "HD Widescreen (1080p)" apply: HD textures via the safe PFF
    /// swap logic (preserving `*.original` backups), then install/configure the
    /// widescreen wrapper. Explicit paths so tests do not depend on
    /// executable/cwd bundle discovery. Failures are logged; a wrapper-install
    /// failure does not report overall success.
    pub fn apply_dfbhd_hd_widescreen_with_paths(
        &mut self,
        install_dir: &str,
        hd_pack_path: &str,
        wrapper_bundle_dir: &str,
    ) {
        self.apply_dfbhd_hd_widescreen_with_all_paths(
            install_dir,
            hd_pack_path,
            None,
            wrapper_bundle_dir,
        );
    }

    /// Combined "HD Widescreen (1080p)" apply managing both `localres.pff` and
    /// (when provided) `resource.pff`, then installs/configures the widescreen
    /// wrapper. Per-step logging; does not claim overall success if a required
    /// step failed.
    pub fn apply_dfbhd_hd_widescreen_with_all_paths(
        &mut self,
        install_dir: &str,
        hd_localres_pack_path: &str,
        hd_resource_pack_path: Option<&str>,
        wrapper_bundle_dir: &str,
    ) {
        let Some(idx) = self.games.iter().position(|g| g.id == "dfbhd") else {
            return;
        };

        // Step 1: apply HD textures (localres required, resource when provided).
        let textures_ok = self.apply_dfbhd_profile_with_pack_paths(
            "hd_textures",
            install_dir,
            hd_localres_pack_path,
            hd_resource_pack_path,
        );

        // Step 2: install/configure the widescreen wrapper.
        match install_dfbhd_wrapper_to_install(
            Path::new(install_dir),
            Path::new(wrapper_bundle_dir),
            &mut self.games[idx].log,
        ) {
            Ok(()) => push_log(
                &mut self.games[idx].log,
                if textures_ok {
                    "profile apply: HD Widescreen (1080p): textures + wrapper applied".to_string()
                } else {
                    "profile apply: HD Widescreen (1080p): wrapper applied, but the HD textures step failed (see log above)".to_string()
                },
            ),
            Err(e) => push_log(
                &mut self.games[idx].log,
                format!("profile apply: HD Widescreen (1080p) wrapper step failed: {e}"),
            ),
        }
    }

    pub fn apply_active_profile_for_game(&mut self, game_id: &str) {
        let Some(idx) = self.games.iter().position(|g| g.id == game_id) else {
            return;
        };

        if game_id != "dfbhd" {
            push_log(
                &mut self.games[idx].log,
                format!(
                    "profile apply: no file-swap apply implemented for game '{}' (informational only)",
                    game_id
                ),
            );
            return;
        }

        let (source_linked, install_dir, active_profile_id) = {
            let g = &self.games[idx];
            (
                g.source_linked,
                g.install_dir.clone(),
                g.active_profile_id.clone(),
            )
        };

        if !source_linked || install_dir.as_deref().unwrap_or("").is_empty() {
            push_log(
                &mut self.games[idx].log,
                "profile apply blocked: attach/scan and link DFBHD install first (install directory required)"
                    .to_string(),
            );
            return;
        }
        let install_dir = install_dir.unwrap_or_default();

        match active_profile_id.as_str() {
            "original" => {
                self.apply_dfbhd_profile_with_pack_paths("original", &install_dir, "", None);
            }
            "hd_textures" => {
                let localres_pack = resolve_dfbhd_hd_localres_pack_path();
                let resource_pack = resolve_dfbhd_hd_resource_pack_path();
                match localres_pack {
                    Some(lpack) => {
                        if resource_pack.is_none() {
                            push_log(
                                &mut self.games[idx].log,
                                "profile apply: HD resource.pff pack not found (expected assets/profiles/dfbhd/hd_textures/resource.pff relative to executable or current directory); applying localres textures only"
                                    .to_string(),
                            );
                        }
                        self.apply_dfbhd_profile_with_pack_paths(
                            "hd_textures",
                            &install_dir,
                            &lpack.to_string_lossy(),
                            resource_pack.as_ref().map(|p| p.to_string_lossy()).as_deref(),
                        );
                    }
                    None => {
                        push_log(
                            &mut self.games[idx].log,
                            "profile apply failed: HD textures pack not found (expected assets/profiles/dfbhd/hd_textures/localres.pff relative to executable or current directory)"
                                .to_string(),
                        );
                    }
                }
            }
            "hd_widescreen_1080p" => {
                let localres_pack = resolve_dfbhd_hd_localres_pack_path();
                let resource_pack = resolve_dfbhd_hd_resource_pack_path();
                let bundle = resolve_dfbhd_wrapper_bundle_dir();
                match (localres_pack, bundle) {
                    (Some(lpack), Some(bundle)) => {
                        if resource_pack.is_none() {
                            push_log(
                                &mut self.games[idx].log,
                                "profile apply: HD resource.pff pack not found (expected assets/profiles/dfbhd/hd_textures/resource.pff relative to executable or current directory); applying localres textures and wrapper"
                                    .to_string(),
                            );
                        }
                        self.apply_dfbhd_hd_widescreen_with_all_paths(
                            &install_dir,
                            &lpack.to_string_lossy(),
                            resource_pack.as_ref().map(|p| p.to_string_lossy()).as_deref(),
                            &bundle.to_string_lossy(),
                        );
                    }
                    (None, Some(_)) => {
                        if resource_pack.is_none() {
                            push_log(
                                &mut self.games[idx].log,
                                "profile apply failed: HD textures localres.pff pack not found (expected assets/profiles/dfbhd/hd_textures/localres.pff relative to executable or current directory); HD resource.pff pack not found (expected assets/profiles/dfbhd/hd_textures/resource.pff relative to executable or current directory); wrapper not installed"
                                    .to_string(),
                            );
                        } else {
                            push_log(
                                &mut self.games[idx].log,
                                "profile apply failed: HD textures localres.pff pack not found (expected assets/profiles/dfbhd/hd_textures/localres.pff relative to executable or current directory); wrapper not installed (resource.pff pack was found but not applied)"
                                    .to_string(),
                            );
                        }
                    }
                    (Some(_), None) => {
                        if resource_pack.is_none() {
                            push_log(
                                &mut self.games[idx].log,
                                "profile apply failed: DFBHD widescreen wrapper bundle not found (expected assets/wrapper/dfbhd_widescreen relative to executable or current directory); wrapper not installed (HD resource.pff pack not found)"
                                    .to_string(),
                            );
                        } else {
                            push_log(
                                &mut self.games[idx].log,
                                "profile apply failed: DFBHD widescreen wrapper bundle not found (expected assets/wrapper/dfbhd_widescreen relative to executable or current directory); wrapper not installed"
                                    .to_string(),
                            );
                        }
                    }
                    (None, None) => {
                        if resource_pack.is_none() {
                            push_log(
                                &mut self.games[idx].log,
                                "profile apply failed: HD textures localres.pff pack and DFBHD widescreen wrapper bundle not found (expected assets/profiles/dfbhd/hd_textures/localres.pff and assets/wrapper/dfbhd_widescreen relative to executable or current directory); HD resource.pff pack not found (expected assets/profiles/dfbhd/hd_textures/resource.pff relative to executable or current directory); wrapper not installed"
                                    .to_string(),
                            );
                        } else {
                            push_log(
                                &mut self.games[idx].log,
                                "profile apply failed: HD textures localres.pff pack and DFBHD widescreen wrapper bundle not found (expected assets/profiles/dfbhd/hd_textures/localres.pff and assets/wrapper/dfbhd_widescreen relative to executable or current directory); wrapper not installed (resource.pff pack was found but not applied)"
                                    .to_string(),
                            );
                        }
                    }
                }
            }
            other => {
                push_log(
                    &mut self.games[idx].log,
                    format!("profile apply failed: unknown DFBHD profile '{}'", other),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GameLinkConfig, LauncherConfig};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "launcher-gui-model-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn default_state_has_dfbhd_with_profiles() {
        let s = LauncherState::default();
        let g = s.games.iter().find(|g| g.id == "dfbhd").expect("dfbhd");
        assert_eq!(g.name, "Delta Force: Black Hawk Down");
        assert!(g
            .profiles
            .iter()
            .any(|p| p.id == "original" && p.name == "Original"));
        assert!(g
            .profiles
            .iter()
            .any(|p| p.id == "hd_textures" && p.name == "HD Textures"));
        assert_eq!(g.active_profile_id, "original");
        assert!(!g.source_linked);
    }

    #[test]
    fn default_state_includes_hd_widescreen_profile() {
        let s = LauncherState::default();
        let g = s.games.iter().find(|g| g.id == "dfbhd").expect("dfbhd");
        assert!(g
            .profiles
            .iter()
            .any(|p| p.id == "original" && p.name == "Original"));
        assert!(g
            .profiles
            .iter()
            .any(|p| p.id == "hd_textures" && p.name == "HD Textures"));
        assert!(g.profiles.iter().any(|p| {
            p.id == "hd_widescreen_1080p" && p.name == "HD Widescreen (1080p)"
        }));
    }

    #[test]
    fn resolve_dfbhd_wrapper_bundle_dir_finds_assets_relative_to_cwd() {
        let dir = unique_temp_dir("wrapper-bundle");
        let bundle = dir.join("assets/wrapper/dfbhd_widescreen");
        fs::create_dir_all(&bundle).unwrap();

        let empty = unique_temp_dir("wrapper-bundle-empty");
        assert_eq!(
            resolve_dfbhd_wrapper_bundle_dir_from_bases([empty.clone()]),
            None
        );
        assert_eq!(
            resolve_dfbhd_wrapper_bundle_dir_from_bases([dir.clone()]),
            Some(bundle.clone())
        );
        assert!(bundle.is_dir());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&empty);
    }

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

    #[test]
    fn wrapper_managed_files_matches_bundle_layout() {
        assert_eq!(
            DFBHD_WRAPPER_MANAGED_FILES,
            &[
                "d3d8.dll",
                "d3d8.ini",
                "dinput8.dll",
                "scripts/BHD_Asi.asi",
                "scripts/BHD_Asi.ini",
            ]
        );
    }

    #[test]
    fn select_and_apply_profile_stub_are_safe() {
        let mut s = LauncherState::default();
        s.select_profile("hd_textures");
        assert_eq!(s.selected_game().unwrap().active_profile_id, "hd_textures");
        s.apply_profile_stub();
        let g = s.selected_game().unwrap();
        assert!(!g.log.is_empty());
        let last = g.log.last().unwrap().to_lowercase();
        assert!(
            last.contains("profile apply") || last.contains("blocked") || last.contains("link"),
            "expected actionable apply log, got: {last}"
        );
    }

    #[test]
    fn launch_game_guarded_when_unlinked() {
        let mut s = LauncherState::default();
        let before = s.selected_game().unwrap().log.len();
        s.launch_game("dfbhd");
        let g = s.selected_game().unwrap();
        assert!(g.log.len() >= before);
        let last = g.log.last().unwrap().to_lowercase();
        assert!(last.contains("attach") || last.contains("scan") || last.contains("link"));
    }

    #[test]
    fn log_is_bounded_to_cap_after_repeated_pushes() {
        let mut s = LauncherState::default();
        for _ in 0..(LOG_CAP + 20) {
            s.apply_profile_stub();
        }
        let g = s.selected_game().unwrap();
        assert_eq!(g.log.len(), LOG_CAP);
        let last = g.log.last().unwrap().to_lowercase();
        assert!(
            last.contains("profile apply") || last.contains("blocked"),
            "expected bounded apply log entry, got: {last}"
        );
        assert!(!g.log.iter().any(|l| l == "awaiting game link..."));
    }

    #[test]
    fn selecting_unknown_profile_is_noop() {
        let mut s = LauncherState::default();
        let before = s.selected_game().unwrap().active_profile_id.clone();
        s.select_profile("does_not_exist");
        assert_eq!(s.selected_game().unwrap().active_profile_id, before);
    }

    #[test]
    fn stub_methods_safe_on_empty_and_unselected_state() {
        let mut empty = LauncherState {
            games: vec![],
            selected_game_idx: 0,
            view: View::Dashboard,
        };
        assert!(empty.selected_game().is_none());
        empty.select_profile("x");
        empty.apply_profile_stub();
        empty.scan_attach_stub();
        empty.launch_stub();
        assert!(empty.selected_game().is_none());

        let mut oob = LauncherState {
            games: LauncherState::default().games,
            selected_game_idx: 99,
            view: View::Dashboard,
        };
        assert!(oob.selected_game().is_none());
        oob.select_profile("x");
        oob.apply_profile_stub();
        oob.scan_attach_stub();
        oob.launch_stub();
        assert!(oob.selected_game().is_none());
    }

    #[test]
    fn link_game_folder_marks_linked_and_records_paths() {
        let dir = unique_temp_dir("link-ok");
        let subdir = dir.join("assets");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(dir.join("dfbhd.exe"), b"exe").unwrap();
        fs::write(subdir.join("mission.pff"), b"pff").unwrap();

        let mut s = LauncherState::default();
        let res = s.link_game_folder("dfbhd", &dir.to_string_lossy());
        assert!(res.is_ok(), "expected success, got {res:?}");

        let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();
        assert!(g.source_linked);
        assert_eq!(g.install_dir.as_deref(), Some(dir.to_string_lossy().as_ref()));
        assert_eq!(
            g.exe_path.as_deref(),
            Some(dir.join("dfbhd.exe").to_string_lossy().as_ref())
        );
        assert_eq!(g.pff_paths.len(), 1);
        let log_text = g.log.join("\n").to_lowercase();
        assert!(log_text.contains("dfbhd.exe"));
        assert!(log_text.contains("pff"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn link_game_folder_failure_does_not_clear_existing_link() {
        let valid = unique_temp_dir("valid");
        fs::write(valid.join("dfbhd.exe"), b"exe").unwrap();
        fs::write(valid.join("data.pff"), b"pff").unwrap();
        let invalid = unique_temp_dir("invalid");
        fs::write(invalid.join("readme.txt"), b"no exe").unwrap();

        let mut s = LauncherState::default();
        s.link_game_folder("dfbhd", &valid.to_string_lossy())
            .expect("initial link should succeed");
        let before = s.games.iter().find(|g| g.id == "dfbhd").unwrap().clone();

        let err = s.link_game_folder("dfbhd", &invalid.to_string_lossy());
        assert!(err.is_err());

        let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();
        assert!(g.source_linked);
        assert_eq!(g.install_dir, before.install_dir);
        assert_eq!(g.exe_path, before.exe_path);
        assert_eq!(g.pff_paths, before.pff_paths);
        let last = g.log.last().unwrap().to_lowercase();
        assert!(
            last.contains("dfbhd") || last.contains("exe") || last.contains("error") || last.contains("fail"),
            "expected clear failure log, got: {last}"
        );

        let _ = fs::remove_dir_all(&valid);
        let _ = fs::remove_dir_all(&invalid);
    }

    #[test]
    fn apply_loaded_and_export_config_roundtrip_state() {
        let dir = unique_temp_dir("roundtrip");
        fs::write(dir.join("dfbhd.exe"), b"exe").unwrap();
        fs::write(dir.join("world.pff"), b"pff").unwrap();

        let mut s = LauncherState::default();
        s.link_game_folder("dfbhd", &dir.to_string_lossy()).unwrap();
        let exported = s.export_config();

        let mut s2 = LauncherState::default();
        s2.apply_loaded_config(&exported);
        let g = s2.games.iter().find(|g| g.id == "dfbhd").unwrap();
        assert!(g.source_linked);
        assert_eq!(
            g.install_dir,
            s.games
                .iter()
                .find(|g| g.id == "dfbhd")
                .unwrap()
                .install_dir
        );
        assert_eq!(
            g.exe_path,
            s.games.iter().find(|g| g.id == "dfbhd").unwrap().exe_path
        );
        assert_eq!(
            g.pff_paths,
            s.games.iter().find(|g| g.id == "dfbhd").unwrap().pff_paths
        );
        assert_eq!(s2.export_config(), exported);

        let _ = fs::remove_dir_all(&dir);
    }

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
        let before = s
            .games
            .iter()
            .find(|g| g.id == "dfbhd")
            .unwrap()
            .active_profile_id
            .clone();
        s.apply_loaded_config(&cfg);
        let after = s
            .games
            .iter()
            .find(|g| g.id == "dfbhd")
            .unwrap()
            .active_profile_id
            .clone();
        assert_eq!(after, before);
    }

    #[test]
    fn relink_replaces_prior_linked_paths_and_state() {
        let dir_a = unique_temp_dir("relink-a");
        fs::write(dir_a.join("dfbhd.exe"), b"exe").unwrap();
        fs::write(dir_a.join("a.pff"), b"pff").unwrap();

        let dir_b = unique_temp_dir("relink-b");
        let sub = dir_b.join("assets");
        fs::create_dir_all(&sub).unwrap();
        fs::write(dir_b.join("dfbhd.exe"), b"exe2").unwrap();
        fs::write(dir_b.join("b.pff"), b"pff").unwrap();
        fs::write(sub.join("c.pff"), b"pff").unwrap();

        let mut s = LauncherState::default();
        s.link_game_folder("dfbhd", &dir_a.to_string_lossy())
            .unwrap();
        let first = s.games.iter().find(|g| g.id == "dfbhd").unwrap().clone();

        s.link_game_folder("dfbhd", &dir_b.to_string_lossy())
            .unwrap();
        let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();

        assert!(g.source_linked);
        assert_eq!(
            g.install_dir.as_deref(),
            Some(dir_b.to_string_lossy().as_ref())
        );
        assert_eq!(
            g.exe_path.as_deref(),
            Some(dir_b.join("dfbhd.exe").to_string_lossy().as_ref())
        );
        assert_ne!(g.install_dir, first.install_dir);
        assert_ne!(g.exe_path, first.exe_path);
        assert_ne!(g.pff_paths, first.pff_paths);
        assert_eq!(g.pff_paths.len(), 2);
        assert!(g
            .pff_paths
            .iter()
            .all(|p| p.starts_with(&dir_b.to_string_lossy().to_string())));
        assert_eq!(g.source_label, dir_b.to_string_lossy().to_string());
        let log_text = g.log.join("\n");
        assert!(log_text.contains(&dir_b.to_string_lossy().to_string()));
        assert!(log_text.contains("linked install"));

        let _ = fs::remove_dir_all(&dir_a);
        let _ = fs::remove_dir_all(&dir_b);
    }

    #[test]
    fn launch_blocked_when_linked_but_executable_missing() {
        let mut s = LauncherState::default();
        {
            let g = s.games.iter_mut().find(|g| g.id == "dfbhd").unwrap();
            g.source_linked = true;
            g.install_dir = Some("C:/Games/DFBHD-missing".to_string());
            g.exe_path = Some("C:/Games/DFBHD-missing/dfbhd.exe".to_string());
            g.log.clear();
        }

        s.launch_game("dfbhd");

        let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();
        let last = g
            .log
            .last()
            .expect("expected launch guard log")
            .to_lowercase();
        assert!(
            last.contains("missing executable"),
            "expected missing executable guard, got: {last}"
        );
        assert!(
            last.contains("re-attach") || last.contains("attach") || last.contains("scan") || last.contains("link"),
            "expected re-attach guidance, got: {last}"
        );
    }

    fn write_file(path: &std::path::Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
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
        assert!(
            g.log.join("\n").to_lowercase().contains("original")
                || g.log.join("\n").to_lowercase().contains("restor")
        );
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

        assert_eq!(
            std::fs::read(dir.join("localres.pff.original")).unwrap(),
            b"ORIGINAL"
        );
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
        s.apply_dfbhd_profile_with_pack_path(
            "hd_textures",
            &dir.to_string_lossy(),
            &pack.to_string_lossy(),
        );

        assert_eq!(std::fs::read(dir.join("localres.pff")).unwrap(), b"HD");
        assert!(dir.join("localres.pff.bak").exists());
        assert_eq!(
            std::fs::read(dir.join("localres.pff.original"))
                .unwrap_or_else(|_| std::fs::read(dir.join("localres.pff.bak")).unwrap()),
            b"ORIGINAL"
        );
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

        s.apply_dfbhd_profile_with_pack_path(
            "hd_textures",
            &dir.to_string_lossy(),
            "C:/definitely/missing/localres.pff",
        );

        assert_eq!(std::fs::read(dir.join("localres.pff")).unwrap(), b"ORIGINAL");
        let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();
        assert!(
            g.log.join("\n").to_lowercase().contains("missing")
                || g.log.join("\n").to_lowercase().contains("not found")
                || g.log.join("\n").to_lowercase().contains("fail")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

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

    #[test]
    fn install_wrapper_copies_managed_files_and_creates_scripts_dir() {
        let install = unique_temp_dir("wrapper-install-copy");
        let bundle = unique_temp_dir("wrapper-bundle-copy");

        for (rel, content) in [
            ("d3d8.dll", b"BHD-D3D8DLL" as &[u8]),
            ("d3d8.ini", b"BHD-D3D8INI"),
            ("dinput8.dll", b"BHD-DINPUT8"),
            ("scripts/BHD_Asi.asi", b"BHD-ASI"),
            ("scripts/BHD_Asi.ini", b"BHD-ASI-INI"),
        ] {
            write_file(&bundle.join(rel), content);
        }

        let mut log: Vec<String> = Vec::new();
        let res = install_dfbhd_wrapper_to_install(&install, &bundle, &mut log);
        assert!(res.is_ok(), "expected install ok, got {res:?}");

        for rel in DFBHD_WRAPPER_MANAGED_FILES {
            let dest = install.join(rel);
            assert!(dest.is_file(), "missing installed file: {dest:?}");
            let expected = std::fs::read(bundle.join(rel)).unwrap();
            assert_eq!(
                std::fs::read(&dest).unwrap(),
                expected,
                "installed content mismatch for {rel}"
            );
        }
        assert!(install.join("scripts").is_dir());

        let _ = std::fs::remove_dir_all(&install);
        let _ = std::fs::remove_dir_all(&bundle);
    }

    #[test]
    fn install_wrapper_backs_up_existing_files_once() {
        let install = unique_temp_dir("wrapper-install-bak");
        let bundle = unique_temp_dir("wrapper-bundle-bak");

        // Pre-existing user files in the install that must survive via backups.
        write_file(&install.join("d3d8.dll"), b"USER-D3D8");
        write_file(&install.join("scripts/BHD_Asi.asi"), b"USER-ASI");

        // Bundle content differs from the installed user content.
        for (rel, content) in [
            ("d3d8.dll", b"BHD-D3D8DLL" as &[u8]),
            ("d3d8.ini", b"BHD-D3D8INI"),
            ("dinput8.dll", b"BHD-DINPUT8"),
            ("scripts/BHD_Asi.asi", b"BHD-ASI"),
            ("scripts/BHD_Asi.ini", b"BHD-ASI-INI"),
        ] {
            write_file(&bundle.join(rel), content);
        }

        let mut log: Vec<String> = Vec::new();
        install_dfbhd_wrapper_to_install(&install, &bundle, &mut log).unwrap();

        assert_eq!(
            std::fs::read(install.join("d3d8.dll.launcherbak")).unwrap(),
            b"USER-D3D8"
        );
        assert_eq!(
            std::fs::read(install.join("scripts/BHD_Asi.asi.launcherbak")).unwrap(),
            b"USER-ASI"
        );
        assert_eq!(
            std::fs::read(install.join("d3d8.dll")).unwrap(),
            b"BHD-D3D8DLL"
        );
        assert_eq!(std::fs::read(install.join("scripts/BHD_Asi.asi")).unwrap(), b"BHD-ASI");

        // Second run must not overwrite existing backups.
        let mut log2: Vec<String> = Vec::new();
        install_dfbhd_wrapper_to_install(&install, &bundle, &mut log2).unwrap();
        assert_eq!(
            std::fs::read(install.join("d3d8.dll.launcherbak")).unwrap(),
            b"USER-D3D8"
        );
        assert_eq!(
            std::fs::read(install.join("scripts/BHD_Asi.asi.launcherbak")).unwrap(),
            b"USER-ASI"
        );

        let _ = std::fs::remove_dir_all(&install);
        let _ = std::fs::remove_dir_all(&bundle);
    }

    #[test]
    fn install_wrapper_missing_bundle_file_errors() {
        let install = unique_temp_dir("wrapper-install-missing-bundle");
        let bundle = unique_temp_dir("wrapper-bundle-missing");

        write_file(&bundle.join("d3d8.dll"), b"BHD-D3D8DLL");
        // scripts/BHD_Asi.asi missing from bundle.

        let mut log: Vec<String> = Vec::new();
        let res = install_dfbhd_wrapper_to_install(&install, &bundle, &mut log);
        assert!(res.is_err(), "expected error for missing bundle file");
        let err_text = res.unwrap_err().to_lowercase();
        assert!(
            err_text.contains("missing") || err_text.contains("not found") || err_text.contains("no such"),
            "actionable error expected, got: {err_text}"
        );
        assert!(
            log.iter().any(|l| l.to_lowercase().contains("missing") || l.to_lowercase().contains("fail")),
            "failure should be logged, got: {log:?}"
        );

        let _ = std::fs::remove_dir_all(&install);
        let _ = std::fs::remove_dir_all(&bundle);
    }

    #[test]
    fn restore_wrapper_removes_installed_files_when_no_backup() {
        let install = unique_temp_dir("wrapper-restore-nobak");

        for rel in DFBHD_WRAPPER_MANAGED_FILES {
            write_file(&install.join(rel), b"WRAPPER");
        }

        let mut log: Vec<String> = Vec::new();
        let res = restore_dfbhd_wrapper_in_install(&install, &mut log);
        assert!(res.is_ok(), "expected restore ok, got {res:?}");

        for rel in DFBHD_WRAPPER_MANAGED_FILES {
            assert!(
                !install.join(rel).exists(),
                "expected removed: {}",
                install.join(rel).display()
            );
        }
        assert!(
            !install.join("scripts").exists(),
            "empty scripts dir should be removed"
        );

        // Idempotent when nothing installed.
        let mut log2: Vec<String> = Vec::new();
        let res2 = restore_dfbhd_wrapper_in_install(&install, &mut log2);
        assert!(res2.is_ok(), "expected idempotent restore ok, got {res2:?}");

        let _ = std::fs::remove_dir_all(&install);
    }

    #[test]
    fn restore_wrapper_restores_launcherbak_and_preserves_unrelated_files() {
        let install = unique_temp_dir("wrapper-restore-bak");

        // Wrapper content currently at d3d8.dll; user's original saved as bak.
        write_file(&install.join("d3d8.dll"), b"WRAPPER");
        write_file(&install.join("d3d8.dll.launcherbak"), b"USER-D3D8");
        write_file(&install.join("scripts/BHD_Asi.asi"), b"WRAPPER-ASI");
        write_file(&install.join("scripts/custom.txt"), b"CUSTOM");
        // Unrelated install content must not be touched.
        write_file(&install.join("dfbhd.exe"), b"EXE");
        write_file(&install.join("other.pff"), b"PFF");

        let mut log: Vec<String> = Vec::new();
        let res = restore_dfbhd_wrapper_in_install(&install, &mut log);
        assert!(res.is_ok(), "expected restore ok, got {res:?}");

        assert_eq!(
            std::fs::read(install.join("d3d8.dll")).unwrap(),
            b"USER-D3D8",
            "user's d3d8.dll must be restored from launcherbak"
        );
        assert!(
            !install.join("d3d8.dll.launcherbak").exists(),
            "launcherbak must be removed after restore"
        );

        assert!(!install.join("scripts/BHD_Asi.asi").exists());
        assert!(!install.join("scripts/BHD_Asi.ini").exists());
        assert_eq!(
            std::fs::read(install.join("scripts/custom.txt")).unwrap(),
            b"CUSTOM",
            "unrelated scripts/custom.txt must be preserved"
        );
        assert!(
            install.join("scripts").is_dir(),
            "non-empty scripts dir must be preserved"
        );

        assert_eq!(std::fs::read(install.join("dfbhd.exe")).unwrap(), b"EXE");
        assert_eq!(std::fs::read(install.join("other.pff")).unwrap(), b"PFF");

        let _ = std::fs::remove_dir_all(&install);
    }

    #[test]
    fn apply_hd_widescreen_applies_textures_and_installs_wrapper() {
        let install = unique_temp_dir("apply-hd-ws");
        write_file(&install.join("localres.pff"), b"ORIGINAL");
        let pack_dir = unique_temp_dir("pack-hd-ws");
        let pack = pack_dir.join("localres.pff");
        write_file(&pack, b"HD");
        let bundle = unique_temp_dir("wrapper-bundle-ws");
        for rel in DFBHD_WRAPPER_MANAGED_FILES {
            write_file(&bundle.join(rel), b"BHD-WRAPPER");
        }

        let mut s = LauncherState::default();
        {
            let g = s.games.iter_mut().find(|g| g.id == "dfbhd").unwrap();
            g.source_linked = true;
            g.install_dir = Some(install.to_string_lossy().to_string());
            g.active_profile_id = "hd_widescreen_1080p".to_string();
            g.log.clear();
        }

        s.apply_dfbhd_hd_widescreen_with_paths(
            &install.to_string_lossy(),
            &pack.to_string_lossy(),
            &bundle.to_string_lossy(),
        );

        // Textures swapped, original preserved.
        assert_eq!(std::fs::read(install.join("localres.pff")).unwrap(), b"HD");
        assert_eq!(
            std::fs::read(install.join("localres.pff.original")).unwrap(),
            b"ORIGINAL"
        );
        // Wrapper files installed with bundle content.
        for rel in DFBHD_WRAPPER_MANAGED_FILES {
            assert_eq!(
                std::fs::read(install.join(rel)).unwrap(),
                b"BHD-WRAPPER",
                "managed wrapper file should be installed: {rel}"
            );
        }
        let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();
        assert!(g
            .log
            .iter()
            .any(|l| l.to_lowercase().contains("wrapper installed")));

        let _ = std::fs::remove_dir_all(&install);
        let _ = std::fs::remove_dir_all(&pack_dir);
        let _ = std::fs::remove_dir_all(&bundle);
    }

    #[test]
    fn apply_original_restores_textures_and_removes_wrapper() {
        let install = unique_temp_dir("apply-orig-ws-cycle");
        write_file(&install.join("localres.pff"), b"ORIGINAL");
        // Pre-existing user file to exercise the launcherbak backup path.
        write_file(&install.join("d3d8.dll"), b"USER-D3D8");
        let pack_dir = unique_temp_dir("pack-ws-cycle");
        let pack = pack_dir.join("localres.pff");
        write_file(&pack, b"HD");
        let bundle = unique_temp_dir("wrapper-bundle-ws-cycle");
        for rel in DFBHD_WRAPPER_MANAGED_FILES {
            write_file(&bundle.join(rel), b"BHD-WRAPPER");
        }

        let mut s = LauncherState::default();
        {
            let g = s.games.iter_mut().find(|g| g.id == "dfbhd").unwrap();
            g.source_linked = true;
            g.install_dir = Some(install.to_string_lossy().to_string());
            g.active_profile_id = "hd_widescreen_1080p".to_string();
            g.log.clear();
        }
        s.apply_dfbhd_hd_widescreen_with_paths(
            &install.to_string_lossy(),
            &pack.to_string_lossy(),
            &bundle.to_string_lossy(),
        );
        // Combined apply installed the wrapper; user file was backed up.
        assert_eq!(std::fs::read(install.join("localres.pff")).unwrap(), b"HD");
        assert!(install.join("d3d8.dll").is_file());
        assert!(install.join("d3d8.dll.launcherbak").is_file());

        // Apply "original": textures restore and wrapper cleanup runs.
        {
            let g = s.games.iter_mut().find(|g| g.id == "dfbhd").unwrap();
            g.active_profile_id = "original".to_string();
            g.log.clear();
        }
        s.apply_dfbhd_profile_with_pack_path("original", &install.to_string_lossy(), "");

        // Texture restored to original.
        assert_eq!(std::fs::read(install.join("localres.pff")).unwrap(), b"ORIGINAL");
        // User d3d8.dll restored from launcherbak; bak removed.
        assert_eq!(std::fs::read(install.join("d3d8.dll")).unwrap(), b"USER-D3D8");
        assert!(!install.join("d3d8.dll.launcherbak").exists());
        // Remaining managed wrapper files removed (no other backups present).
        for rel in DFBHD_WRAPPER_MANAGED_FILES
            .iter()
            .filter(|rel| **rel != "d3d8.dll")
        {
            assert!(
                !install.join(rel).exists(),
                "managed wrapper path should be gone: {rel}"
            );
        }
        // Managed scripts dir is now empty and removed.
        assert!(!install.join("scripts").exists());

        let g = s.games.iter().find(|g| g.id == "dfbhd").unwrap();
        let log_text = g.log.join("\n").to_lowercase();
        assert!(
            log_text.contains("restor") || log_text.contains("original"),
            "expected restore log, got: {log_text}"
        );
        assert!(
            log_text.contains("wrapper"),
            "expected wrapper cleanup log, got: {log_text}"
        );

        // Idempotent: a second original apply still succeeds.
        s.apply_dfbhd_profile_with_pack_path("original", &install.to_string_lossy(), "");
        assert_eq!(std::fs::read(install.join("localres.pff")).unwrap(), b"ORIGINAL");

        let _ = std::fs::remove_dir_all(&install);
        let _ = std::fs::remove_dir_all(&pack_dir);
        let _ = std::fs::remove_dir_all(&bundle);
    }

    #[test]
    fn apply_hd_textures_does_not_install_or_remove_wrapper() {
        let install = unique_temp_dir("apply-hd-tw-wrapper");
        write_file(&install.join("localres.pff"), b"ORIGINAL");
        let pack_dir = unique_temp_dir("pack-hd-tw-wrapper");
        let pack = pack_dir.join("localres.pff");
        write_file(&pack, b"HD");

        let mut s = LauncherState::default();
        {
            let g = s.games.iter_mut().find(|g| g.id == "dfbhd").unwrap();
            g.source_linked = true;
            g.install_dir = Some(install.to_string_lossy().to_string());
            g.active_profile_id = "hd_textures".to_string();
            g.log.clear();
        }
        s.apply_dfbhd_profile_with_pack_path(
            "hd_textures",
            &install.to_string_lossy(),
            &pack.to_string_lossy(),
        );
        assert_eq!(std::fs::read(install.join("localres.pff")).unwrap(), b"HD");
        assert_eq!(
            std::fs::read(install.join("localres.pff.original")).unwrap(),
            b"ORIGINAL"
        );
        for rel in DFBHD_WRAPPER_MANAGED_FILES {
            assert!(
                !install.join(rel).exists(),
                "texture-only apply must not install wrapper path: {rel}"
            );
        }

        // If the wrapper is already present, texture-only apply must not remove it.
        write_file(&install.join("d3d8.dll"), b"USER-D3D8");
        write_file(&install.join("d3d8.dll.launcherbak"), b"OLDER-D3D8");
        write_file(&install.join("scripts/BHD_Asi.asi"), b"USER-ASI");
        let mut s2 = LauncherState::default();
        {
            let g = s2.games.iter_mut().find(|g| g.id == "dfbhd").unwrap();
            g.source_linked = true;
            g.install_dir = Some(install.to_string_lossy().to_string());
            g.active_profile_id = "hd_textures".to_string();
            g.log.clear();
        }
        s2.apply_dfbhd_profile_with_pack_path(
            "hd_textures",
            &install.to_string_lossy(),
            &pack.to_string_lossy(),
        );
        assert_eq!(std::fs::read(install.join("d3d8.dll")).unwrap(), b"USER-D3D8");
        assert!(install.join("d3d8.dll.launcherbak").exists());
        assert_eq!(
            std::fs::read(install.join("scripts/BHD_Asi.asi")).unwrap(),
            b"USER-ASI"
        );
        assert_eq!(std::fs::read(install.join("localres.pff")).unwrap(), b"HD");

        let _ = std::fs::remove_dir_all(&install);
        let _ = std::fs::remove_dir_all(&pack_dir);
    }

    #[test]
    fn select_and_persist_hd_widescreen_profile_id() {
        let mut s = LauncherState::default();
        let g = s.games.iter().find(|g| g.id == "dfbhd").expect("dfbhd");
        assert_eq!(g.active_profile_id, "original");

        s.select_profile("hd_widescreen_1080p");
        assert_eq!(
            s.games
                .iter()
                .find(|g| g.id == "dfbhd")
                .unwrap()
                .active_profile_id,
            "hd_widescreen_1080p"
        );

        let cfg = s.export_config();
        assert_eq!(
            cfg.games
                .get("dfbhd")
                .and_then(|c| c.active_profile_id.as_deref()),
            Some("hd_widescreen_1080p")
        );

        let mut s2 = LauncherState::default();
        s2.apply_loaded_config(&cfg);
        assert_eq!(
            s2.games
                .iter()
                .find(|g| g.id == "dfbhd")
                .unwrap()
                .active_profile_id,
            "hd_widescreen_1080p"
        );

        // Unknown ids remain ignored on selection.
        s2.select_profile("not_a_profile");
        assert_eq!(
            s2.games
                .iter()
                .find(|g| g.id == "dfbhd")
                .unwrap()
                .active_profile_id,
            "hd_widescreen_1080p"
        );
    }
}
