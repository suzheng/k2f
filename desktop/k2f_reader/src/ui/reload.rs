//! When a watched `.K2F` is stable enough to read again.
//! Stat only — the file bytes are read after [`Decision::Reload`].

use std::fs::Metadata;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

/// Quiet period after the last size/mtime/inode change before a reload.
pub const STABLE_FOR: Duration = Duration::from_millis(400);

/// How often to stat when the watcher misses an event (network volumes).
pub const POLL_EVERY: Duration = Duration::from_secs(1);

/// Identity of the file we last painted, or of a change still settling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileIdentity {
    mtime: SystemTime,
    len: u64,
    inode: u64,
}

impl FileIdentity {
    pub fn from_path(path: &Path) -> Option<Self> {
        let meta = std::fs::metadata(path).ok()?;
        Self::from_meta(&meta)
    }

    fn from_meta(meta: &Metadata) -> Option<Self> {
        Some(Self {
            mtime: meta.modified().ok()?,
            len: meta.len(),
            inode: inode_of(meta),
        })
    }
}

#[cfg(unix)]
fn inode_of(meta: &Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    meta.ino()
}

#[cfg(windows)]
fn inode_of(meta: &Metadata) -> u64 {
    use std::os::windows::fs::MetadataExt;
    meta.file_index().unwrap_or(0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Parked {
    Failed(FileIdentity),
    Held(FileIdentity),
}

#[derive(Clone, Copy, Debug)]
struct Pending {
    identity: FileIdentity,
    since: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Matches the loaded file, or a failed read we will not retry.
    Idle,
    /// Changed, and still inside [`STABLE_FOR`].
    Waiting,
    /// Stable and the form has no unsaved values.
    Reload,
    /// Stable, but unsaved form values must stay on screen.
    Hold,
}

#[derive(Clone, Debug)]
pub struct DiskState {
    loaded: Option<FileIdentity>,
    pending: Option<Pending>,
    parked: Option<Parked>,
}

impl DiskState {
    pub fn new() -> Self {
        Self {
            loaded: None,
            pending: None,
            parked: None,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn note_loaded(&mut self, id: FileIdentity) {
        self.loaded = Some(id);
        self.pending = None;
        self.parked = None;
    }

    pub fn note_failed(&mut self, id: FileIdentity) {
        self.pending = None;
        self.parked = Some(Parked::Failed(id));
    }

    /// Next stat. `None` when nothing is open.
    pub fn wake_at(&self, now: Instant, active: bool) -> Option<Instant> {
        if !active && self.pending.is_none() {
            return None;
        }
        let poll = now + POLL_EVERY;
        match self.pending {
            Some(p) => Some(poll.min(p.since + STABLE_FOR)),
            None => Some(poll),
        }
    }

    pub fn poll(&mut self, now: Instant, current: Option<FileIdentity>, dirty: bool) -> Decision {
        let Some(current) = current else {
            self.pending = None;
            return Decision::Idle;
        };
        if self.loaded == Some(current) {
            self.pending = None;
            self.parked = None;
            return Decision::Idle;
        }
        if self.parked == Some(Parked::Failed(current)) {
            self.pending = None;
            return Decision::Idle;
        }
        if self.parked == Some(Parked::Held(current)) {
            self.pending = None;
            return if dirty {
                Decision::Hold
            } else {
                Decision::Reload
            };
        }
        match self.pending {
            Some(p)
                if p.identity == current
                    && now.saturating_duration_since(p.since) >= STABLE_FOR =>
            {
                self.pending = None;
                if dirty {
                    self.parked = Some(Parked::Held(current));
                    Decision::Hold
                } else {
                    Decision::Reload
                }
            }
            Some(p) if p.identity == current => Decision::Waiting,
            _ => {
                self.pending = Some(Pending {
                    identity: current,
                    since: now,
                });
                Decision::Waiting
            }
        }
    }
}

impl Default for DiskState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("k2f-reader-reload-{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn restat_of_one_file_matches() {
        let path = scratch("same").join("doc.K2F");
        fs::write(&path, b"lock-v1").unwrap();
        let a = FileIdentity::from_path(&path).unwrap();
        let b = FileIdentity::from_path(&path).unwrap();
        assert_eq!(a, b);
        let mut disk = DiskState::new();
        disk.note_loaded(a);
        assert_eq!(disk.poll(Instant::now(), Some(b), false), Decision::Idle);
    }

    #[test]
    fn rewrite_waits_then_reloads() {
        let path = scratch("wait").join("doc.K2F");
        fs::write(&path, b"one").unwrap();
        let first = FileIdentity::from_path(&path).unwrap();
        fs::write(&path, b"two-two").unwrap();
        let second = FileIdentity::from_path(&path).unwrap();
        assert_ne!(first, second);

        let mut disk = DiskState::new();
        disk.note_loaded(first);
        let t0 = Instant::now();
        assert_eq!(disk.poll(t0, Some(second), false), Decision::Waiting);
        assert_eq!(
            disk.poll(t0 + Duration::from_millis(200), Some(second), false),
            Decision::Waiting
        );
        assert_eq!(
            disk.poll(t0 + STABLE_FOR, Some(second), false),
            Decision::Reload
        );
    }

    #[test]
    fn another_write_restarts_the_wait() {
        let dir = scratch("burst");
        let path = dir.join("doc.K2F");
        fs::write(&path, b"a").unwrap();
        let loaded = FileIdentity::from_path(&path).unwrap();
        fs::write(&path, b"bb").unwrap();
        let mid = FileIdentity::from_path(&path).unwrap();
        fs::write(&path, b"ccc").unwrap();
        let done = FileIdentity::from_path(&path).unwrap();
        assert_ne!(mid, done);

        let mut disk = DiskState::new();
        disk.note_loaded(loaded);
        let t0 = Instant::now();
        assert_eq!(disk.poll(t0, Some(mid), false), Decision::Waiting);
        let t1 = t0 + Duration::from_millis(300);
        assert_eq!(disk.poll(t1, Some(done), false), Decision::Waiting);
        assert_eq!(
            disk.poll(t1 + Duration::from_millis(300), Some(done), false),
            Decision::Waiting
        );
        assert_eq!(
            disk.poll(t1 + STABLE_FOR, Some(done), false),
            Decision::Reload
        );
    }

    #[test]
    fn dirty_form_holds_and_stays_held() {
        let path = scratch("hold").join("doc.K2F");
        fs::write(&path, b"one").unwrap();
        let first = FileIdentity::from_path(&path).unwrap();
        fs::write(&path, b"two!").unwrap();
        let second = FileIdentity::from_path(&path).unwrap();
        let mut disk = DiskState::new();
        disk.note_loaded(first);
        let t0 = Instant::now();
        assert_eq!(disk.poll(t0, Some(second), true), Decision::Waiting);
        assert_eq!(
            disk.poll(t0 + STABLE_FOR, Some(second), true),
            Decision::Hold
        );
        assert_eq!(
            disk.poll(t0 + STABLE_FOR + POLL_EVERY, Some(second), true),
            Decision::Hold
        );
        assert_eq!(
            disk.poll(t0 + STABLE_FOR + POLL_EVERY, Some(second), false),
            Decision::Reload
        );
    }

    #[test]
    fn failed_read_is_not_retried_until_the_file_changes() {
        let path = scratch("fail").join("doc.K2F");
        fs::write(&path, b"good").unwrap();
        let good = FileIdentity::from_path(&path).unwrap();
        fs::write(&path, b"bad!").unwrap();
        let bad = FileIdentity::from_path(&path).unwrap();
        let mut disk = DiskState::new();
        disk.note_loaded(good);
        disk.note_failed(bad);
        let now = Instant::now();
        assert_eq!(disk.poll(now, Some(bad), false), Decision::Idle);
        fs::write(&path, b"better").unwrap();
        let next = FileIdentity::from_path(&path).unwrap();
        assert_eq!(disk.poll(now, Some(next), false), Decision::Waiting);
        assert_eq!(
            disk.poll(now + STABLE_FOR, Some(next), false),
            Decision::Reload
        );
    }
}
