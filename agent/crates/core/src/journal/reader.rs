use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::parse::{parse_line, ParsedLine};

pub const JOURNAL_DIR_COMPONENTS: [&str; 2] = ["Frontier Developments", "Elite Dangerous"];

pub fn journal_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<(SystemTime, PathBuf)> = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !is_journal_file(&path) {
            continue;
        }
        let metadata = entry.metadata()?;
        let created = metadata.created().or_else(|_| metadata.modified())?;
        files.push((created, path));
    }

    files.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    Ok(files.into_iter().map(|(_, path)| path).collect())
}

pub fn latest_journal(dir: &Path) -> io::Result<Option<PathBuf>> {
    Ok(journal_files(dir)?.pop())
}

fn is_journal_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    let Some(rest) = name.strip_prefix("Journal.") else {
        return false;
    };
    let Some(rest) = rest.strip_suffix(".log") else {
        return false;
    };
    let Some((stamp, part)) = rest.rsplit_once('.') else {
        return false;
    };

    part.len() == 2 && part.bytes().all(|byte| byte.is_ascii_digit()) && is_journal_stamp(stamp)
}

fn is_journal_stamp(stamp: &str) -> bool {
    stamp.len() >= 12
        && stamp
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'-' || byte == b'T')
        && stamp
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_digit())
}

pub struct JournalTailer {
    path: PathBuf,
    offset: u64,
    pending: Vec<u8>,
}

impl JournalTailer {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            offset: 0,
            pending: Vec::new(),
        }
    }

    pub fn resume_at(path: PathBuf, offset: u64) -> Self {
        Self {
            path,
            offset,
            pending: Vec::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn read_new(&mut self) -> io::Result<Vec<ParsedLine>> {
        let mut file = File::open(&self.path)?;
        let len = file.metadata()?.len();

        if len < self.offset {
            self.offset = 0;
            self.pending.clear();
        }
        if len == self.offset {
            return Ok(Vec::new());
        }

        file.seek(SeekFrom::Start(self.offset))?;
        let mut chunk = Vec::with_capacity((len - self.offset) as usize);
        let read = file.read_to_end(&mut chunk)? as u64;
        self.offset += read;
        self.pending.extend_from_slice(&chunk);

        let mut parsed = Vec::new();
        let mut consumed = 0;
        while let Some(newline) = self.pending[consumed..]
            .iter()
            .position(|byte| *byte == b'\n')
        {
            let end = consumed + newline;
            let line = String::from_utf8_lossy(&self.pending[consumed..end]);
            parsed.push(parse_line(&line));
            consumed = end + 1;
        }
        self.pending.drain(..consumed);

        Ok(parsed)
    }
}

pub struct JournalWatcher {
    dir: PathBuf,
    current: Option<JournalTailer>,
    visited: BTreeMap<PathBuf, u64>,
}

impl JournalWatcher {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            current: None,
            visited: BTreeMap::new(),
        }
    }

    pub fn resume(dir: PathBuf, path: PathBuf, offset: u64) -> Self {
        Self {
            dir,
            current: Some(JournalTailer::resume_at(path.clone(), offset)),
            visited: BTreeMap::from([(path, offset)]),
        }
    }

    pub fn position(&self) -> Option<(&Path, u64)> {
        self.current
            .as_ref()
            .map(|tailer| (tailer.path(), tailer.offset()))
    }

    pub fn poll(&mut self) -> io::Result<Vec<ParsedLine>> {
        let latest = match latest_journal(&self.dir)? {
            Some(path) => path,
            None => return Ok(Vec::new()),
        };

        let mut lines = Vec::new();
        match &mut self.current {
            Some(tailer) if tailer.path() == latest => {
                lines.extend(read_existing(tailer)?);
            }
            Some(tailer) => {
                lines.extend(read_existing(tailer)?);
                self.visited
                    .insert(tailer.path().to_path_buf(), tailer.offset());
                lines.extend(self.follow(latest)?);
            }
            None => {
                lines.extend(self.follow(latest)?);
            }
        }
        if let Some(tailer) = &self.current {
            self.visited
                .insert(tailer.path().to_path_buf(), tailer.offset());
        }
        Ok(lines)
    }

    fn follow(&mut self, path: PathBuf) -> io::Result<Vec<ParsedLine>> {
        let offset = self.visited.get(&path).copied().unwrap_or(0);
        let mut next = JournalTailer::resume_at(path, offset);
        let lines = next.read_new()?;
        self.current = Some(next);
        Ok(lines)
    }
}

fn read_existing(tailer: &mut JournalTailer) -> io::Result<Vec<ParsedLine>> {
    match tailer.read_new() {
        Ok(lines) => Ok(lines),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}
