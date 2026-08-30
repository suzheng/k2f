use k2f_sdk::AgentError;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Serialize)]
pub struct ToolError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl ToolError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn from_agent(err: AgentError) -> Self {
        Self {
            code: err.code.to_string(),
            message: err.message,
            hint: None,
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("INVALID_ARGUMENT", message)
    }

    pub fn path_required(tool: &str) -> Self {
        Self::new(
            "PATH_REQUIRED",
            format!("{tool} requires path (writes file; never returns package bytes)"),
        )
        .with_hint(format!("Call {tool} again with path set to a local file path"))
    }

    pub fn wrong_session_kind(need: &str, how: &str) -> Self {
        Self::new(
            "WRONG_SESSION_KIND",
            format!("{need} session required"),
        )
        .with_hint(how.to_string())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    pub fn forbidden(name: &str) -> Self {
        Self::new(
            "FORBIDDEN_TOOL",
            format!("tool '{name}' is not exposed via MCP"),
        )
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL", message)
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            json!({
                "code": self.code,
                "message": self.message,
            })
            .to_string()
        })
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_json_string())
    }
}

impl std::error::Error for ToolError {}
