use k2f_sdk::Editor;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SessionMeta {
    pub label: String,
    pub source_path: Option<PathBuf>,
    pub kind: &'static str,
}

pub enum SessionKind {
    Editor(Editor),
}

pub struct Session {
    pub meta: SessionMeta,
    pub kind: SessionKind,
}

impl Session {
    pub fn editor(meta: SessionMeta, editor: Editor) -> Self {
        Self {
            meta,
            kind: SessionKind::Editor(editor),
        }
    }

    pub fn editor_mut(&mut self) -> Result<&mut Editor, crate::error::ToolError> {
        match &mut self.kind {
            SessionKind::Editor(ed) => Ok(ed),
        }
    }
}
