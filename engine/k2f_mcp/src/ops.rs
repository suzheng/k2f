use crate::error::ToolError;
use crate::publish::publish_bytes;
use crate::session::{Session, SessionMeta, SessionStore};
use k2f_paint::OpenedDocument;
use k2f_sdk::{markdown_to_k2f, Editor, MarkdownOptions, PageSize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub struct K2fState {
    pub sessions: SessionStore,
    pub publish_origin: String,
}

impl K2fState {
    pub fn new(publish_origin: impl Into<String>) -> Self {
        Self {
            sessions: SessionStore::default(),
            publish_origin: publish_origin.into(),
        }
    }

    pub fn create(
        &mut self,
        title: &str,
        dest_dir: &str,
        template: &str,
        page_size: &str,
    ) -> Result<Value, ToolError> {
        let dest = resolve_path(dest_dir)?;
        if dest.exists() {
            return Err(ToolError::invalid(format!(
                "destination already exists: {}",
                dest.display()
            )));
        }
        let src = resolve_path(template)?;
        if !src.is_dir() {
            return Err(ToolError::invalid(format!(
                "template must be an author directory: {}",
                src.display()
            )));
        }
        copy_dir_all(&src, &dest).map_err(|e| {
            ToolError::invalid(format!("copy {} → {}: {e}", src.display(), dest.display()))
        })?;
        patch_manifest(&dest, title, page_size)?;
        let editor = Editor::open_dir(&dest).map_err(ToolError::from_agent)?;
        let outline = parse_outline(&editor)?;
        let meta = SessionMeta {
            label: dest.display().to_string(),
            source_path: Some(dest),
            kind: "editor",
        };
        let id = self.sessions.insert(Session::editor(meta, editor));
        Ok(json!({
            "session_id": id.to_string(),
            "kind": "editor",
            "outline": outline,
            "dest_dir": dest_dir,
        }))
    }

    pub fn open(&mut self, path: &str) -> Result<Value, ToolError> {
        let path = resolve_path(path)?;
        if path.is_dir() {
            let editor = Editor::open_dir(&path).map_err(ToolError::from_agent)?;
            let outline = parse_outline(&editor)?;
            let meta = SessionMeta {
                label: path.display().to_string(),
                source_path: Some(path),
                kind: "editor",
            };
            let id = self.sessions.insert(Session::editor(meta, editor));
            return Ok(json!({
                "session_id": id.to_string(),
                "kind": "editor",
                "outline": outline,
            }));
        }
        let bytes =
            fs::read(&path).map_err(|e| ToolError::invalid(format!("read {path:?}: {e}")))?;
        let editor = Editor::open(&bytes).map_err(ToolError::from_agent)?;
        let outline = parse_outline(&editor)?;
        let meta = SessionMeta {
            label: path.display().to_string(),
            source_path: Some(path),
            kind: "editor",
        };
        let id = self.sessions.insert(Session::editor(meta, editor));
        Ok(json!({
            "session_id": id.to_string(),
            "kind": "editor",
            "outline": outline,
        }))
    }

    pub fn outline(&self, session_id: &str) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get(&id)?;
        let SessionKind::Editor(ed) = &session.kind;
        Ok(parse_outline(ed)?)
    }

    pub fn get_node(&self, session_id: &str, node_id: &str) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get(&id)?;
        let SessionKind::Editor(ed) = &session.kind;
        let raw = ed.get_node_json(node_id).map_err(ToolError::from_agent)?;
        serde_json::from_str(&raw).map_err(|e| ToolError::internal(e.to_string()))
    }

    pub fn search(&self, session_id: &str, query: &str) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get(&id)?;
        let SessionKind::Editor(ed) = &session.kind;
        Ok(json!(ed.search(query)))
    }

    pub fn replace_text(
        &mut self,
        session_id: &str,
        node_id: &str,
        text: &str,
    ) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get_mut(&id)?;
        session
            .editor_mut()?
            .replace_text(node_id, text)
            .map_err(ToolError::from_agent)?;
        Ok(json!({ "ok": true }))
    }

    pub fn set_role(
        &mut self,
        session_id: &str,
        node_id: &str,
        role: &str,
        variant: Option<&str>,
    ) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get_mut(&id)?;
        session
            .editor_mut()?
            .set_role(node_id, role, variant)
            .map_err(ToolError::from_agent)?;
        Ok(json!({ "ok": true }))
    }

    pub fn insert_node(
        &mut self,
        session_id: &str,
        parent_id: &str,
        index: usize,
        node: &Value,
    ) -> Result<Value, ToolError> {
        if !node.is_object() {
            return Err(ToolError::invalid("node must be a JSON object")
                .with_hint("Pass semantic node JSON without x/y geometry fields"));
        }
        let node_json =
            serde_json::to_string(node).map_err(|e| ToolError::invalid(e.to_string()))?;
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get_mut(&id)?;
        session
            .editor_mut()?
            .insert_node(parent_id, index, &node_json)
            .map_err(ToolError::from_agent)?;
        Ok(json!({ "ok": true }))
    }

    pub fn delete_node(&mut self, session_id: &str, node_id: &str) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get_mut(&id)?;
        session
            .editor_mut()?
            .delete_node(node_id)
            .map_err(ToolError::from_agent)?;
        Ok(json!({ "ok": true }))
    }

    pub fn validate(&self, session_id: &str) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get(&id)?;
        let SessionKind::Editor(ed) = &session.kind;
        ed.validate_package().map_err(ToolError::from_agent)?;
        Ok(json!({
            "ok": true,
            "checks": "semantic_tree_and_theme_vocab",
        }))
    }

    pub fn diff(&self, session_id: &str) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get(&id)?;
        let SessionKind::Editor(ed) = &session.kind;
        let raw = ed.diff_json().map_err(ToolError::from_agent)?;
        serde_json::from_str(&raw).map_err(|e| ToolError::internal(e.to_string()))
    }

    pub fn save(
        &mut self,
        session_id: &str,
        expected_content_hash: Option<&str>,
        path: &str,
    ) -> Result<Value, ToolError> {
        if path.trim().is_empty() {
            return Err(ToolError::path_required("save"));
        }
        let id = SessionStore::parse_id(session_id)?;
        let bytes = {
            let session = self.sessions.get_mut(&id)?;
            session
                .editor_mut()?
                .save_with(expected_content_hash)
                .map_err(ToolError::from_agent)?
        };
        let path = resolve_path(path)?;
        fs::write(&path, &bytes).map_err(|e| ToolError::invalid(format!("write {path:?}: {e}")))?;
        let banner = OpenedDocument::open(&bytes)
            .map(|d| d.banner().as_str().to_string())
            .unwrap_or_else(|_| "UNKNOWN".into());
        Ok(json!({
            "path": path.display().to_string(),
            "size": bytes.len(),
            "banner": banner,
        }))
    }

    pub fn save_dir(&mut self, session_id: &str, dest_dir: &str) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let dest = resolve_path(dest_dir)?;
        let session = self.sessions.get_mut(&id)?;
        session
            .editor_mut()?
            .save_dir(&dest)
            .map_err(ToolError::from_agent)?;
        Ok(json!({
            "path": dest.display().to_string(),
            "kind": "author_source",
        }))
    }

    pub fn markdown_to_k2f(
        &mut self,
        markdown: &str,
        title: &str,
        template: &str,
        page_size: &str,
    ) -> Result<Value, ToolError> {
        let mut opts = MarkdownOptions::new(title, template).map_err(ToolError::from_agent)?;
        opts.page_size = PageSize::parse(page_size).map_err(ToolError::from_agent)?;
        opts.image_base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let result = markdown_to_k2f(markdown, opts).map_err(ToolError::from_agent)?;
        let editor = Editor::open(&result.bytes).map_err(ToolError::from_agent)?;
        let outline = parse_outline(&editor)?;
        let meta = SessionMeta {
            label: format!("markdown:{title}"),
            source_path: None,
            kind: "editor",
        };
        let id = self.sessions.insert(Session::editor(meta, editor));
        Ok(json!({
            "session_id": id.to_string(),
            "kind": "editor",
            "outline": outline,
            "report": { "warnings": result.report.warnings },
        }))
    }

    pub fn export_pdf(
        &self,
        session_id: &str,
        path: &str,
        scale: Option<f32>,
    ) -> Result<Value, ToolError> {
        if path.trim().is_empty() {
            return Err(ToolError::path_required("export_pdf"));
        }
        let scale = match scale {
            Some(s) => k2f_sdk::parse_pdf_scale(s).map_err(ToolError::from_agent)?,
            None => k2f_sdk::PdfScale::DEFAULT,
        };
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get(&id)?;
        let SessionKind::Editor(ed) = &session.kind;
        let pdf = ed
            .export_pdf_bytes_at(scale)
            .map_err(ToolError::from_agent)?;
        let path = resolve_path(path)?;
        fs::write(&path, &pdf).map_err(|e| ToolError::invalid(format!("write {path:?}: {e}")))?;
        Ok(json!({
            "path": path.display().to_string(),
            "size": pdf.len(),
        }))
    }

    pub fn verify(&self, path: Option<&str>, session_id: Option<&str>) -> Result<Value, ToolError> {
        match (path, session_id) {
            (Some(p), _) => self.verify_path(p),
            (None, Some(sid)) => self.verify_session_id(sid),
            (None, None) => Err(ToolError::invalid("path or session_id required")
                .with_hint("Pass path to a .K2F file, or session_id from open")),
        }
    }

    fn verify_session_id(&self, session_id: &str) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let session = self.sessions.get(&id)?;
        if let Some(path) = &session.meta.source_path {
            if path.is_dir() {
                return Err(ToolError::invalid(
                    "author source directory is not a .K2F package; pack first or pass path to .K2F",
                ));
            }
            return self.verify_path(&path.display().to_string());
        }
        Err(
            ToolError::invalid("session has no source file; pass path= to verify saved bytes")
                .with_hint("Call save with path, then verify with that path"),
        )
    }

    fn verify_path(&self, path: &str) -> Result<Value, ToolError> {
        let path = resolve_path(path)?;
        let bytes =
            fs::read(&path).map_err(|e| ToolError::invalid(format!("read {path:?}: {e}")))?;
        self.verify_bytes(&bytes)
    }

    fn verify_bytes(&self, bytes: &[u8]) -> Result<Value, ToolError> {
        let opened = OpenedDocument::open(bytes).map_err(|e| ToolError::invalid(e.to_string()))?;
        Ok(json!({
            "banner": opened.banner().as_str(),
            "status_code": opened.status_code(),
            "hash_code": opened.hash_code(),
            "content_hash": opened.content_hash(),
            "appearance_hash": opened.appearance_hash(),
            "fingerprint": opened.fingerprint(),
            "signed_by": opened.signed_by(),
            "signed_at": opened.signed_at(),
            "generated_by": opened.generated_by(),
        }))
    }

    pub async fn publish(
        &mut self,
        session_id: &str,
        expected_content_hash: Option<&str>,
        dry_run: bool,
    ) -> Result<Value, ToolError> {
        let id = SessionStore::parse_id(session_id)?;
        let bytes = {
            let session = self.sessions.get_mut(&id)?;
            session
                .editor_mut()?
                .save_with(expected_content_hash)
                .map_err(ToolError::from_agent)?
        };
        let opened = OpenedDocument::open(&bytes).map_err(|e| ToolError::invalid(e.to_string()))?;
        if dry_run {
            return Ok(json!({
                "dry_run": true,
                "size": bytes.len(),
                "content_hash": opened.content_hash(),
                "appearance_hash": opened.appearance_hash(),
                "banner": opened.banner().as_str(),
            }));
        }
        let resp = publish_bytes(&self.publish_origin, &bytes)
            .await
            .map_err(|e| {
                ToolError::internal(e).with_hint(
                    "Check K2F_PUBLISH_ORIGIN and that the site /api/publish is reachable",
                )
            })?;
        Ok(json!({
            "appearanceHash": resp.appearance_hash,
            "url": resp.url,
            "iframe": resp.iframe,
            "size": bytes.len(),
        }))
    }
}

use crate::session::SessionKind;

fn patch_manifest(dest: &Path, title: &str, page_size: &str) -> Result<(), ToolError> {
    let manifest_path = dest.join("manifest.json");
    let raw = fs::read_to_string(&manifest_path)
        .map_err(|e| ToolError::invalid(format!("read manifest: {e}")))?;
    let mut manifest: Value =
        serde_json::from_str(&raw).map_err(|e| ToolError::invalid(e.to_string()))?;
    manifest["title"] = json!(title);
    if let Ok(ps) = PageSize::parse(page_size) {
        manifest["page_config"] = serde_json::to_value(ps.page_config())
            .map_err(|e| ToolError::internal(e.to_string()))?;
    }
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).map_err(|e| ToolError::internal(e.to_string()))?
            + "\n",
    )
    .map_err(|e| ToolError::invalid(format!("write manifest: {e}")))?;
    Ok(())
}

fn parse_outline(ed: &Editor) -> Result<Value, ToolError> {
    let raw = ed.outline_json().map_err(ToolError::from_agent)?;
    Ok(serde_json::from_str(&raw).unwrap_or(json!([])))
}

fn resolve_path(path: &str) -> Result<PathBuf, ToolError> {
    let p = Path::new(path);
    let resolved = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| ToolError::internal(e.to_string()))?
            .join(p)
    };
    Ok(resolved)
}

fn copy_dir_all(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

pub fn default_publish_origin() -> String {
    std::env::var("K2F_PUBLISH_ORIGIN").unwrap_or_else(|_| "http://127.0.0.1:3000".into())
}
