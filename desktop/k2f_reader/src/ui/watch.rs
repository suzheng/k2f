//! Parent-directory watch so an atomic replace of the `.K2F` is visible.

use super::wake::Wake;
use notify::Watcher;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoopProxy;

/// True when an event path is the file we have open.
pub fn watches_file(event_paths: &[PathBuf], file: &Path) -> bool {
    let Some(name) = file.file_name() else {
        return false;
    };
    event_paths
        .iter()
        .any(|p| p == file || p.file_name() == Some(name))
}

pub struct SourceWatch {
    watcher: notify::RecommendedWatcher,
    parent: Option<PathBuf>,
    file: Arc<Mutex<PathBuf>>,
}

impl SourceWatch {
    pub fn start(proxy: EventLoopProxy<Wake>) -> anyhow::Result<Self> {
        let file = Arc::new(Mutex::new(PathBuf::new()));
        let file_cb = Arc::clone(&file);
        let watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let Ok(event) = res else {
                return;
            };
            let Ok(current) = file_cb.lock() else {
                return;
            };
            if current.as_os_str().is_empty() || !watches_file(&event.paths, &current) {
                return;
            }
            let _ = proxy.send_event(Wake::Disk);
        })?;
        Ok(Self {
            watcher,
            parent: None,
            file,
        })
    }

    pub fn retarget(&mut self, path: &Path) -> anyhow::Result<()> {
        let parent = parent_dir(path);
        if self.parent.as_deref() != Some(parent.as_path()) {
            if let Some(old) = self.parent.take() {
                let _ = self.watcher.unwatch(&old);
            }
            self.watcher
                .watch(&parent, notify::RecursiveMode::NonRecursive)?;
            self.parent = Some(parent);
        }
        *self.file.lock().expect("watch path") = path.to_path_buf();
        Ok(())
    }
}

fn parent_dir(path: &Path) -> PathBuf {
    match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    #[test]
    fn parent_watch_sees_a_rewrite() {
        let dir = std::env::temp_dir().join(format!("k2f-reader-watch-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("doc.K2F");
        fs::write(&path, b"before").unwrap();

        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let _ = tx.send(res);
        })
        .unwrap();
        watcher
            .watch(path.parent().unwrap(), notify::RecursiveMode::NonRecursive)
            .unwrap();
        fs::write(&path, b"after-rewrite").unwrap();

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut saw = false;
        while Instant::now() < deadline {
            let left = deadline.saturating_duration_since(Instant::now());
            match rx.recv_timeout(left.min(Duration::from_millis(200))) {
                Ok(Ok(event)) if watches_file(&event.paths, &path) => {
                    saw = true;
                    break;
                }
                Ok(_) => {}
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        assert!(saw, "parent watch must report a rewrite of doc.K2F");
    }
}
