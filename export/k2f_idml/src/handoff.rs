use crate::export::{export_opened, looks_like_idml};
use crate::idml;
use crate::IdmlError;
use k2f_core::is_font_face_path;
use k2f_paint::OpenedDocument;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const DOCUMENT_FONTS: &str = "Document Fonts";

#[derive(Clone, Debug)]
pub struct IdmlHandoff {
    pub stem: String,
    pub idml: Vec<u8>,
    pub fonts: BTreeMap<String, Vec<u8>>,
}

pub fn export_handoff(doc: &OpenedDocument) -> Result<IdmlHandoff, IdmlError> {
    Ok(IdmlHandoff {
        stem: sanitize_stem(doc.title()),
        idml: export_opened(doc)?,
        fonts: collect_faces(doc),
    })
}

pub fn export_handoff_bytes(package_bytes: &[u8]) -> Result<IdmlHandoff, IdmlError> {
    if looks_like_idml(package_bytes) {
        return Err(IdmlError::NotASource);
    }
    export_handoff(&OpenedDocument::open(package_bytes)?)
}

pub fn export_package_zip(package_bytes: &[u8]) -> Result<Vec<u8>, IdmlError> {
    export_handoff_bytes(package_bytes)?.to_zip_bytes()
}

impl IdmlHandoff {
    pub fn with_stem(mut self, stem: &str) -> Self {
        self.stem = sanitize_stem(stem);
        self
    }

    pub fn write_dir(&self, dir: &Path) -> Result<(), IdmlError> {
        fs::create_dir_all(dir)
            .map_err(|e| IdmlError::Write(format!("mkdir {}: {e}", dir.display())))?;
        let fonts_dir = dir.join(DOCUMENT_FONTS);
        fs::create_dir_all(&fonts_dir)
            .map_err(|e| IdmlError::Write(format!("mkdir {}: {e}", fonts_dir.display())))?;
        for (name, bytes) in &self.fonts {
            let path = fonts_dir.join(name);
            fs::write(&path, bytes)
                .map_err(|e| IdmlError::Write(format!("write {}: {e}", path.display())))?;
        }
        let idml_path = dir.join(format!("{}.idml", self.stem));
        fs::write(&idml_path, &self.idml)
            .map_err(|e| IdmlError::Write(format!("write {}: {e}", idml_path.display())))?;
        Ok(())
    }

    pub fn to_zip_bytes(&self) -> Result<Vec<u8>, IdmlError> {
        let mut files = BTreeMap::new();
        files.insert(format!("{}/{}.idml", self.stem, self.stem), self.idml.clone());
        for (name, bytes) in &self.fonts {
            files.insert(
                format!("{}/{DOCUMENT_FONTS}/{name}", self.stem),
                bytes.clone(),
            );
        }
        idml::write_files_zip(&files)
    }
}

/// Default: package directory. `.zip` → zip of that tree. `--idml-only` → lone IDML file.
/// `-o foo.idml` without `--idml-only` writes directory `foo/`.
pub fn write_handoff_output(
    handoff: &IdmlHandoff,
    output: &Path,
    idml_only: bool,
) -> Result<PathBuf, IdmlError> {
    if idml_only {
        write_file(output, &handoff.idml)?;
        return Ok(output.to_path_buf());
    }
    if ext_is(output, "zip") {
        let pkg = handoff.clone().with_stem(&stem_of(output));
        write_file(output, &pkg.to_zip_bytes()?)?;
        return Ok(output.to_path_buf());
    }
    let dir = package_dir_from_output(output);
    let pkg = handoff.clone().with_stem(&stem_of(&dir));
    pkg.write_dir(&dir)?;
    Ok(dir)
}

pub fn package_dir_from_output(output: &Path) -> PathBuf {
    if ext_is(output, "idml") {
        output.with_extension("")
    } else {
        output.to_path_buf()
    }
}

pub fn sanitize_stem(raw: &str) -> String {
    let s: String = raw
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '-' => c,
            _ => '_',
        })
        .collect();
    let s = s.trim_matches('_');
    if s.is_empty() {
        "document".into()
    } else {
        s.to_string()
    }
}

fn collect_faces(doc: &OpenedDocument) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    for (key, bytes) in doc.fonts() {
        if key.eq_ignore_ascii_case("default") {
            continue;
        }
        let norm = key.replace('\\', "/");
        if norm
            .split('/')
            .any(|part| part.eq_ignore_ascii_case("licenses"))
        {
            continue;
        }
        if !is_font_face_path(key) {
            continue;
        }
        let Some(base) = Path::new(key).file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if base.is_empty() || base.starts_with('.') {
            continue;
        }
        out.entry(base.to_string()).or_insert_with(|| bytes.clone());
    }
    out
}

fn stem_of(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(sanitize_stem)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "document".into())
}

fn ext_is(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), IdmlError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|e| IdmlError::Write(format!("mkdir {}: {e}", parent.display())))?;
        }
    }
    fs::write(path, bytes).map_err(|e| IdmlError::Write(format!("write {}: {e}", path.display())))
}
