use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GameLinkConfig {
    pub source_linked: bool,
    pub install_dir: Option<String>,
    pub exe_path: Option<String>,
    pub pff_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LauncherConfig {
    pub games: BTreeMap<String, GameLinkConfig>,
}

#[allow(dead_code)]
pub fn config_path() -> PathBuf {
    let base_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    base_dir.join("launcher-config.json")
}

#[allow(dead_code)]
pub fn load_config() -> LauncherConfig {
    let path = config_path();
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return LauncherConfig::default(),
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

#[allow(dead_code)]
pub fn save_config(cfg: &LauncherConfig) -> Result<(), String> {
    let path = config_path();
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_config_serialization() {
        let mut cfg = LauncherConfig::default();
        cfg.games.insert(
            "dfbhd".to_string(),
            GameLinkConfig {
                source_linked: true,
                install_dir: Some("C:/Games/DFBHD".to_string()),
                exe_path: Some("C:/Games/DFBHD/dfbhd.exe".to_string()),
                pff_paths: vec!["C:/Games/DFBHD/resource.pff".to_string()],
            },
        );
        let json = serde_json::to_string(&cfg).unwrap();
        let back: LauncherConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn load_invalid_json_string_as_default_logic() {
        let bad: Result<LauncherConfig, _> = serde_json::from_str("not json");
        assert!(bad.is_err());
    }
}
