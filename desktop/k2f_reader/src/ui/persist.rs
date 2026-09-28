//! Replace a `.K2F` in place. The temp file sits beside the target so
//! `rename` is atomic on Unix and a crash cannot truncate the open package.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::Path;

pub fn replace_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?;
    let mut tmp_name = OsString::from(".");
    tmp_name.push(name);
    tmp_name.push(".tmp");
    let tmp = path.with_file_name(tmp_name);
    fs::write(&tmp, bytes)?;
    if let Err(err) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("k2f-reader-persist-{}-{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn replace_overwrites_and_drops_the_temp_file() {
        let dir = scratch("ok");
        let path = dir.join("doc.K2F");
        fs::write(&path, b"old").unwrap();
        replace_file(&path, b"new-package").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"new-package");
        assert!(!dir.join(".doc.K2F.tmp").exists());
    }

    #[test]
    fn replace_onto_a_directory_leaves_it_alone() {
        let dir = scratch("dir");
        let path = dir.join("doc.K2F");
        fs::create_dir(&path).unwrap();
        let err = replace_file(&path, b"nope").unwrap_err();
        assert!(path.is_dir(), "{err}");
        assert!(!dir.join(".doc.K2F.tmp").exists());
    }
}
