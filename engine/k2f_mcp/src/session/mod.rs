mod kinds;

pub use kinds::{Session, SessionKind, SessionMeta};

use crate::error::ToolError;
use std::collections::HashMap;
use uuid::Uuid;

pub struct SessionStore {
    sessions: HashMap<Uuid, Session>,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

impl SessionStore {
    pub fn insert(&mut self, session: Session) -> Uuid {
        let id = Uuid::new_v4();
        self.sessions.insert(id, session);
        id
    }

    pub fn get(&self, id: &Uuid) -> Result<&Session, ToolError> {
        self.sessions
            .get(id)
            .ok_or_else(|| ToolError::not_found(format!("unknown session_id {id}")))
    }

    pub fn get_mut(&mut self, id: &Uuid) -> Result<&mut Session, ToolError> {
        self.sessions
            .get_mut(id)
            .ok_or_else(|| ToolError::not_found(format!("unknown session_id {id}")))
    }

    pub fn ids(&self) -> Vec<Uuid> {
        self.sessions.keys().copied().collect()
    }

    pub fn parse_id(raw: &str) -> Result<Uuid, ToolError> {
        Uuid::parse_str(raw).map_err(|e| ToolError::invalid(format!("session_id: {e}")))
    }
}
