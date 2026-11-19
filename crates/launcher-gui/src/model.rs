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

    pub fn apply_dfbhd_profile_with_pack_path(
        &mut self,
        profile_id: &str,
        install_dir: &str,
        hd_pack_path: &str,
    ) {
        let Some(idx) = self.games.iter().position(|g| g.id == "dfbhd") else {
            return;
        };
        let install = Path::new(install_dir);
        let target = install.join("localres.pff");
        let original_backup = install.join("localres.pff.original");
        let bak = install.join("localres.pff.bak");

        match profile_id {
            "original" => {
                let (source, source_label) = if original_backup.exists() {
                    (original_backup.clone(), "localres.pff.original")
                } else if bak.exists() {
                    (bak.clone(), "localres.pff.bak")
                } else {
                    push_log(
                        &mut self.games[idx].log,
                        "profile apply failed: no original backup found to restore (expected localres.pff.original or localres.pff.bak)"
                            .to_string(),
                    );
                    return;
                };
                match std::fs::copy(&source, &target) {
                    Ok(_) => push_log(
                        &mut self.games[idx].log,
                        format!(
                            "restored original localres.pff from '{}' to '{}'",
                            source.display(),
                            target.display()
                        ),
                    ),
                    Err(e) => push_log(
                        &mut self.games[idx].log,
                        format!(
                            "profile apply failed: could not restore original from '{}': {}",
                            source_label, e
                        ),
                    ),
                }
            }
            "hd_textures" => {
                let pack = Path::new(hd_pack_path);
                if !pack.is_file() {
                    push_log(
                        &mut self.games[idx].log,
                        format!(
                            "profile apply failed: HD textures pack not found at '{}'",
                            hd_pack_path
                        ),
                    );
                    return;
                }

                if !original_backup.exists() {
                    let backup_source = if bak.exists() {
                        Some((bak.clone(), "localres.pff.bak"))
                    } else if target.exists() {
                        Some((target.clone(), "localres.pff"))
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
                                        "profile apply failed: could not create original backup from '{}': {}",
                                        label, e
                                    ),
                                );
                                return;
                            }
                        }
                    } else {
                        push_log(
                            &mut self.games[idx].log,
                            "profile apply failed: cannot apply HD textures because no localres.pff, localres.pff.bak, or localres.pff.original was found to preserve as original"
                                .to_string(),
                        );
                        return;
                    }
                }

                match std::fs::copy(pack, &target) {
                    Ok(_) => push_log(
                        &mut self.games[idx].log,
                        format!(
                            "applied HD textures profile: copied '{}' to '{}'",
                            pack.display(),
                            target.display()
                        ),
                    ),
                    Err(e) => push_log(
                        &mut self.games[idx].log,
                        format!(
                            "profile apply failed: could not copy HD textures pack to '{}': {}",
                            target.display(),
                            e
                        ),
                    ),
                }
            }
            _ => {
                push_log(
                    &mut self.games[idx].log,
                    format!("profile apply failed: unknown profile '{}'", profile_id),
                );
            }
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
                self.apply_dfbhd_profile_with_pack_path("original", &install_dir, "");
            }
            "hd_textures" => match resolve_dfbhd_hd_localres_pack_path() {
                Some(pack) => {
                    self.apply_dfbhd_profile_with_pack_path(
                        "hd_textures",
                        &install_dir,
                        &pack.to_string_lossy(),
                    );
                }
                None => {
                    push_log(
                        &mut self.games[idx].log,
                        "profile apply failed: HD textures pack not found (expected assets/profiles/dfbhd/hd_textures/localres.pff relative to executable or current directory)"
                            .to_string(),
                    );
                }
            },
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
}
