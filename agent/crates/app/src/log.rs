use std::io::Write;
use std::path::PathBuf;

pub const LOG_FILE_NAME: &str = "elite-agent.log";

const MAX_BYTES: u64 = 1024 * 1024;

pub fn path() -> PathBuf {
    crate::dir::data_dir().join(LOG_FILE_NAME)
}

pub fn append(line: &str) {
    let path = path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    if std::fs::metadata(&path).is_ok_and(|meta| meta.len() > MAX_BYTES) {
        let _ = std::fs::rename(&path, path.with_extension("log.old"));
    }

    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    else {
        return;
    };

    let stamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    let _ = writeln!(file, "{stamp} {line}");
}
