use std::path::Path;

use agent_core::journal::{HISTORY_FILE_NAME, POSITION_FILE_NAME};
use agent_sync::{OBSERVATIONS_FILE_NAME, PENDING_FILE_NAME};

use crate::log::LOG_FILE_NAME;

fn moved_files() -> [String; 6] {
    [
        LOG_FILE_NAME.to_owned(),
        format!("{LOG_FILE_NAME}.old"),
        PENDING_FILE_NAME.to_owned(),
        POSITION_FILE_NAME.to_owned(),
        HISTORY_FILE_NAME.to_owned(),
        OBSERVATIONS_FILE_NAME.to_owned(),
    ]
}

pub fn move_old_files(from: &Path, to: &Path) -> Vec<String> {
    if from == to {
        return Vec::new();
    }

    let mut moved = Vec::new();
    for name in moved_files() {
        let source = from.join(&name);
        let target = to.join(&name);
        if !source.is_file() || target.exists() {
            continue;
        }
        if std::fs::create_dir_all(to).is_err() {
            return moved;
        }
        let done = std::fs::rename(&source, &target).is_ok()
            || (std::fs::copy(&source, &target).is_ok() && std::fs::remove_file(&source).is_ok());
        if done {
            moved.push(name);
        }
    }
    moved
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("elite-agent-relocate-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn old_files_move_out_from_under_the_exe() {
        let root = scratch("move");
        let (exe_dir, profile) = (root.join("Desktop"), root.join("AppData"));
        std::fs::create_dir_all(&exe_dir).unwrap();
        std::fs::write(exe_dir.join(HISTORY_FILE_NAME), "{}").unwrap();
        std::fs::write(exe_dir.join(LOG_FILE_NAME), "old log").unwrap();
        std::fs::write(exe_dir.join("notes.txt"), "mine").unwrap();

        let moved = move_old_files(&exe_dir, &profile);

        assert_eq!(moved.len(), 2);
        assert!(profile.join(HISTORY_FILE_NAME).is_file());
        assert!(!exe_dir.join(HISTORY_FILE_NAME).exists());
        assert!(exe_dir.join("notes.txt").is_file());
    }

    #[test]
    fn newer_file_in_the_profile_is_not_overwritten() {
        let root = scratch("keep");
        let (exe_dir, profile) = (root.join("Desktop"), root.join("AppData"));
        std::fs::create_dir_all(&exe_dir).unwrap();
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::write(exe_dir.join(HISTORY_FILE_NAME), "old").unwrap();
        std::fs::write(profile.join(HISTORY_FILE_NAME), "new").unwrap();

        assert!(move_old_files(&exe_dir, &profile).is_empty());
        assert_eq!(
            std::fs::read_to_string(profile.join(HISTORY_FILE_NAME)).unwrap(),
            "new"
        );
    }

    #[test]
    fn same_folder_is_left_alone() {
        let root = scratch("same");
        std::fs::write(root.join(LOG_FILE_NAME), "log").unwrap();

        assert!(move_old_files(&root, &root).is_empty());
        assert!(root.join(LOG_FILE_NAME).is_file());
    }
}
