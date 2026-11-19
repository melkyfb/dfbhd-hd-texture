use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedInstall {
    pub install_dir: String,
    pub exe_path: String,
    pub pff_paths: Vec<String>,
}

pub fn scan_install_dir(dir: &Path) -> Result<DetectedInstall, String> {
    if !dir.exists() {
        return Err(format!(
            "Install directory does not exist: {}",
            dir.to_string_lossy()
        ));
    }
    if !dir.is_dir() {
        return Err(format!(
            "Install path is not a directory: {}",
            dir.to_string_lossy()
        ));
    }

    let mut exe_path: Option<String> = None;
    let root_entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read install directory {}: {e}", dir.to_string_lossy()))?;
    for entry in root_entries {
        let entry = entry.map_err(|e| {
            format!(
                "Failed to read entry in install directory {}: {e}",
                dir.to_string_lossy()
            )
        })?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.eq_ignore_ascii_case("dfbhd.exe") {
                    exe_path = Some(path.to_string_lossy().to_string());
                    break;
                }
            }
        }
    }

    let exe_path = exe_path.ok_or_else(|| {
        format!(
            "dfbhd.exe not found in selected install directory: {}",
            dir.to_string_lossy()
        )
    })?;

    let mut pff_paths = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current).map_err(|e| {
            format!(
                "Failed to read directory {}: {e}",
                current.to_string_lossy()
            )
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| {
                format!(
                    "Failed to read entry in directory {}: {e}",
                    current.to_string_lossy()
                )
            })?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let is_pff = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("pff"))
                    .unwrap_or(false);
                if is_pff {
                    pff_paths.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    pff_paths.sort();

    Ok(DetectedInstall {
        install_dir: dir.to_string_lossy().to_string(),
        exe_path,
        pff_paths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "launcher-gui-detect-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn detects_exe_case_insensitively_and_collects_pffs_recursively() {
        let dir = unique_temp_dir("ok");
        let subdir = dir.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(dir.join("DFBHD.EXE"), b"exe").unwrap();
        fs::write(subdir.join("a.pff"), b"pff").unwrap();
        fs::write(dir.join("B.PFF"), b"pff").unwrap();
        fs::write(dir.join("notes.txt"), b"ignore").unwrap();

        let result = scan_install_dir(&dir).expect("should detect install");
        assert_eq!(result.install_dir, dir.to_string_lossy().to_string());
        assert_eq!(result.exe_path, dir.join("DFBHD.EXE").to_string_lossy().to_string());

        let mut expected_pffs = vec![
            subdir.join("a.pff").to_string_lossy().to_string(),
            dir.join("B.PFF").to_string_lossy().to_string(),
        ];
        expected_pffs.sort();
        assert_eq!(result.pff_paths, expected_pffs);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn errors_when_exe_missing() {
        let dir = unique_temp_dir("noexe");
        fs::write(dir.join("data.pff"), b"pff").unwrap();

        let err = scan_install_dir(&dir).unwrap_err();
        assert!(
            err.to_lowercase().contains("dfbhd.exe") || err.to_lowercase().contains("exe"),
            "error should mention missing exe, got: {err}"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn errors_when_install_dir_missing() {
        let dir = std::env::temp_dir().join(format!(
            "launcher-gui-detect-missing-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);

        let err = scan_install_dir(&dir).unwrap_err();
        assert!(
            err.to_lowercase().contains("does not exist"),
            "error should mention missing directory, got: {err}"
        );
    }

    #[test]
    fn errors_when_install_path_is_not_directory() {
        let dir = unique_temp_dir("notdir");
        let file_path = dir.join("file.txt");
        fs::write(&file_path, b"not a dir").unwrap();

        let err = scan_install_dir(&file_path).unwrap_err();
        assert!(
            err.to_lowercase().contains("not a directory"),
            "error should mention non-directory path, got: {err}"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
