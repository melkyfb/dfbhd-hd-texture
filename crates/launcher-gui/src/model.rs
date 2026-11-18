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

fn push_log(log: &mut Vec<String>, entry: impl Into<String>) {
    log.push(entry.into());
    if log.len() > LOG_CAP {
        let overflow = log.len() - LOG_CAP;
        log.drain(0..overflow);
    }
}

impl Default for LauncherState {
    fn default() -> Self {
        let dfbhd = GameNode {
            id: "dfbhd".to_string(),
            name: "Delta Force: Black Hawk Down".to_string(),
            source_linked: false,
            source_label: "ISO/Install [not linked]".to_string(),
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
            source_label: "[empty / available]".to_string(),
            profiles: Vec::new(),
            active_profile_id: String::new(),
            log: Vec::new(),
        };

        let slot2 = GameNode {
            id: "slot-2".to_string(),
            name: "Empty Slot".to_string(),
            source_linked: false,
            source_label: "[empty / available]".to_string(),
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

    pub fn apply_profile_stub(&mut self) {
        let Some(game) = self.selected_game_mut() else {
            return;
        };
        let profile = if game.active_profile_id.is_empty() {
            "none".to_string()
        } else {
            game.active_profile_id.clone()
        };
        let status = if game.source_linked { "linked" } else { "unlinked" };
        push_log(
            &mut game.log,
            format!("apply profile stub: '{}' (source {})", profile, status),
        );
    }

    pub fn scan_attach_stub(&mut self) {
        let Some(game) = self.selected_game_mut() else {
            return;
        };
        push_log(
            &mut game.log,
            "scan/attach stub: no filesystem scan performed; source not linked".to_string(),
        );
    }

    pub fn launch_stub(&mut self) {
        let Some(game) = self.selected_game_mut() else {
            return;
        };
        push_log(
            &mut game.log,
            "launch blocked: attach/scan and link game source first".to_string(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn log_is_bounded_to_cap_after_repeated_pushes() {
        let mut s = LauncherState::default();
        for _ in 0..(LOG_CAP + 20) {
            s.apply_profile_stub();
        }
        let g = s.selected_game().unwrap();
        assert_eq!(g.log.len(), LOG_CAP);
        let last = g.log.last().unwrap();
        assert!(last.contains("apply profile stub"));
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
}
