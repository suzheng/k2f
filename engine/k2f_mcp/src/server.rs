use crate::catalog::assert_catalog_matches_handlers;
use crate::error::ToolError;
use crate::ops::{default_publish_origin, K2fState};
use crate::resources::{list_session_resources, read_resource, resource_templates};
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{
    ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParams,
    ReadResourceRequestParams, ReadResourceResponse, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, Json, RoleServer,
    ServerHandler, ServiceExt,
};
use serde::Deserialize;
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex};

fn map_ok(result: Result<Value, ToolError>) -> Result<Json<Value>, String> {
    result.map(Json).map_err(|e| e.to_json_string())
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct CreateArgs {
    title: String,
    #[schemars(description = "Output directory for author source (must not exist)")]
    dest_dir: String,
    #[serde(default = "default_template")]
    #[schemars(description = "blank | invoice | legal | clinical_summary | report")]
    template: String,
    #[serde(default = "default_page_size")]
    #[schemars(description = "A4 or Letter")]
    page_size: String,
}

fn default_page_size() -> String {
    "A4".into()
}
fn default_template() -> String {
    "blank".into()
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct OpenArgs {
    #[schemars(description = "Absolute or relative path to a .K2F package")]
    path: String,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct SessionArgs {
    session_id: String,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct GetNodeArgs {
    session_id: String,
    node_id: String,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct SearchArgs {
    session_id: String,
    query: String,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct ReplaceTextArgs {
    session_id: String,
    node_id: String,
    text: String,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct SetRoleArgs {
    session_id: String,
    node_id: String,
    role: String,
    variant: Option<String>,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct InsertNodeArgs {
    session_id: String,
    parent_id: String,
    index: usize,
    #[schemars(description = "Semantic node object (no x/y geometry)")]
    node: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct DeleteNodeArgs {
    session_id: String,
    node_id: String,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct SaveArgs {
    session_id: String,
    #[schemars(description = "Optimistic lock vs content hash at open")]
    expected_content_hash: Option<String>,
    #[schemars(description = "Required local path to write the .K2F package")]
    path: String,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct MarkdownArgs {
    markdown: String,
    title: String,
    #[serde(default = "default_report_template")]
    template: String,
    #[serde(default = "default_page_size")]
    page_size: String,
}

fn default_report_template() -> String {
    "report".into()
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct ExportPdfArgs {
    session_id: String,
    #[schemars(description = "Required local path to write the PDF")]
    path: String,
    #[serde(default)]
    #[schemars(description = "Raster stamp scale for paint ops: 2, 3, or 4 (default 4)")]
    scale: Option<f32>,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct VerifyArgs {
    path: Option<String>,
    session_id: Option<String>,
}

#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
struct PublishArgs {
    session_id: String,
    expected_content_hash: Option<String>,
    #[serde(default)]
    #[schemars(description = "If true, relock and return hashes without POSTing")]
    dry_run: bool,
}

#[derive(Clone)]
pub struct K2fServer {
    state: Arc<Mutex<K2fState>>,
    tool_router: ToolRouter<Self>,
}

impl K2fServer {
    pub fn new() -> Self {
        Self::with_publish_origin(default_publish_origin())
    }

    pub fn with_publish_origin(origin: impl Into<String>) -> Self {
        let server = Self {
            state: Arc::new(Mutex::new(K2fState::new(origin))),
            tool_router: Self::tool_router(),
        };
        assert_catalog_matches_handlers(&[
            "create",
            "open",
            "outline",
            "get_node",
            "search",
            "replace_text",
            "set_role",
            "insert_node",
            "delete_node",
            "validate",
            "diff",
            "save",
            "markdown_to_k2f",
            "export_pdf",
            "verify",
            "publish",
        ]);
        server
    }

    pub fn state(&self) -> Arc<Mutex<K2fState>> {
        self.state.clone()
    }
}

impl Default for K2fServer {
    fn default() -> Self {
        Self::new()
    }
}

// Descriptions must match engine/k2f_mcp/mcp_tools.json exactly (drift test enforces).
#[tool_router]
impl K2fServer {
    #[tool(
        description = "Copy an official template to dest_dir and open an Editor session. dest_dir must not exist.",
        annotations(
            title = "Create from template",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    fn create(&self, Parameters(args): Parameters<CreateArgs>) -> Result<Json<Value>, String> {
        let mut state = self.state.lock().unwrap();
        map_ok(state.create(
            &args.title,
            &args.dest_dir,
            &args.template,
            &args.page_size,
        ))
    }

    #[tool(
        description = "Open a local .K2F into an Editor session. Returns session_id + outline — never lock JSON.",
        annotations(
            title = "Open .K2F",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    fn open(&self, Parameters(args): Parameters<OpenArgs>) -> Result<Json<Value>, String> {
        let mut state = self.state.lock().unwrap();
        map_ok(state.open(&args.path))
    }

    #[tool(
        description = "Editor tree summary: id, role, ~80 char preview, children ids. Requires Editor session.",
        annotations(
            title = "Outline",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn outline(&self, Parameters(args): Parameters<SessionArgs>) -> Result<Json<Value>, String> {
        let state = self.state.lock().unwrap();
        map_ok(state.outline(&args.session_id))
    }

    #[tool(
        description = "Fetch one semantic node by stable id (agent JSON, no layout coordinates).",
        annotations(
            title = "Get node",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn get_node(&self, Parameters(args): Parameters<GetNodeArgs>) -> Result<Json<Value>, String> {
        let state = self.state.lock().unwrap();
        map_ok(state.get_node(&args.session_id, &args.node_id))
    }

    #[tool(
        description = "Search node text; returns matching node ids. Requires Editor session.",
        annotations(
            title = "Search",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn search(&self, Parameters(args): Parameters<SearchArgs>) -> Result<Json<Value>, String> {
        let state = self.state.lock().unwrap();
        map_ok(state.search(&args.session_id, &args.query))
    }

    #[tool(
        description = "Replace text on a Text/Math node (idempotent for same id+text). Requires Editor session.",
        annotations(
            title = "Replace text",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn replace_text(
        &self,
        Parameters(args): Parameters<ReplaceTextArgs>,
    ) -> Result<Json<Value>, String> {
        let mut state = self.state.lock().unwrap();
        map_ok(state.replace_text(&args.session_id, &args.node_id, &args.text))
    }

    #[tool(
        description = "Set semantic role (and optional variant) on a node. Role must exist in the package theme.",
        annotations(
            title = "Set role",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn set_role(&self, Parameters(args): Parameters<SetRoleArgs>) -> Result<Json<Value>, String> {
        let mut state = self.state.lock().unwrap();
        map_ok(state.set_role(
            &args.session_id,
            &args.node_id,
            &args.role,
            args.variant.as_deref(),
        ))
    }

    #[tool(
        description = "Insert a child under parent_id at index. node is a JSON object (not a string) with no x/y. Requires Editor session — not create().",
        annotations(
            title = "Insert node",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    fn insert_node(
        &self,
        Parameters(args): Parameters<InsertNodeArgs>,
    ) -> Result<Json<Value>, String> {
        let mut state = self.state.lock().unwrap();
        let node = Value::Object(args.node);
        map_ok(state.insert_node(
            &args.session_id,
            &args.parent_id,
            args.index,
            &node,
        ))
    }

    #[tool(
        description = "Delete a node by id (cannot delete root). Requires Editor session.",
        annotations(
            title = "Delete node",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    fn delete_node(
        &self,
        Parameters(args): Parameters<DeleteNodeArgs>,
    ) -> Result<Json<Value>, String> {
        let mut state = self.state.lock().unwrap();
        map_ok(state.delete_node(&args.session_id, &args.node_id))
    }

    #[tool(
        description = "Validate author source (semantic tree + theme vocab). Requires Editor session — use diff + save + verify before export.",
        annotations(
            title = "Validate",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn validate(&self, Parameters(args): Parameters<SessionArgs>) -> Result<Json<Value>, String> {
        let state = self.state.lock().unwrap();
        map_ok(state.validate(&args.session_id))
    }

    #[tool(
        description = "Semantic diff vs baseline at open/create time. Requires Editor session.",
        annotations(
            title = "Diff",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn diff(&self, Parameters(args): Parameters<SessionArgs>) -> Result<Json<Value>, String> {
        let state = self.state.lock().unwrap();
        map_ok(state.diff(&args.session_id))
    }

    #[tool(
        description = "Relock the session to an UNSIGNED .K2F at path (required). Strips signatures; appends changelog if diff is non-empty. Returns {path,size,banner} — never package bytes. Pass expected_content_hash for optimistic lock vs the hash at open.",
        annotations(
            title = "Save",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    fn save(&self, Parameters(args): Parameters<SaveArgs>) -> Result<Json<Value>, String> {
        let mut state = self.state.lock().unwrap();
        map_ok(state.save(
            &args.session_id,
            args.expected_content_hash.as_deref(),
            &args.path,
        ))
    }

    #[tool(
        description = "Convert Markdown to an Editor session (mutable). Returns session_id, outline, and report.warnings.",
        annotations(
            title = "Markdown to K2F",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    fn markdown_to_k2f(
        &self,
        Parameters(args): Parameters<MarkdownArgs>,
    ) -> Result<Json<Value>, String> {
        let mut state = self.state.lock().unwrap();
        map_ok(state.markdown_to_k2f(
            &args.markdown,
            &args.title,
            &args.template,
            &args.page_size,
        ))
    }

    #[tool(
        description = "Export PDF from current lock to path (required). Unsaved Editor draws the old lock — save first. Returns {path,size} — never PDF bytes.",
        annotations(
            title = "Export PDF",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    fn export_pdf(
        &self,
        Parameters(args): Parameters<ExportPdfArgs>,
    ) -> Result<Json<Value>, String> {
        let state = self.state.lock().unwrap();
        map_ok(state.export_pdf(&args.session_id, &args.path, args.scale))
    }

    #[tool(
        description = "Integrity banner and trust metadata for a .K2F path or open session with source_path.",
        annotations(
            title = "Verify",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    fn verify(&self, Parameters(args): Parameters<VerifyArgs>) -> Result<Json<Value>, String> {
        let state = self.state.lock().unwrap();
        map_ok(state.verify(args.path.as_deref(), args.session_id.as_deref()))
    }

    #[tool(
        description = "Relock session and POST to K2F publish API; returns /v/{appearance_hash}. Set dry_run=true to preview hashes without POST. Env K2F_PUBLISH_ORIGIN.",
        annotations(
            title = "Publish",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn publish(
        &self,
        Parameters(args): Parameters<PublishArgs>,
    ) -> Result<Json<Value>, String> {
        // Relock under the mutex, then drop before awaiting HTTP.
        let (origin, bytes, dry_run) = {
            let mut state = self.state.lock().unwrap();
            let id = crate::session::SessionStore::parse_id(&args.session_id)
                .map_err(|e| e.to_json_string())?;
            let bytes = match state.sessions.get_mut(&id) {
                Ok(session) => session
                    .editor_mut()
                    .map_err(|e| e.to_json_string())?
                    .save_with(args.expected_content_hash.as_deref())
                    .map_err(|e| ToolError::from_agent(e).to_json_string())?,
                Err(e) => return Err(e.to_json_string()),
            };
            (state.publish_origin.clone(), bytes, args.dry_run)
        };
        if dry_run {
            let opened = k2f_paint::OpenedDocument::open(&bytes)
                .map_err(|e| ToolError::invalid(e.to_string()).to_json_string())?;
            return Ok(Json(serde_json::json!({
                "dry_run": true,
                "size": bytes.len(),
                "content_hash": opened.content_hash(),
                "appearance_hash": opened.appearance_hash(),
                "banner": opened.banner().as_str(),
            })));
        }
        let resp = crate::publish::publish_bytes(&origin, &bytes)
            .await
            .map_err(|e| {
                ToolError::internal(e)
                    .with_hint(
                        "Check K2F_PUBLISH_ORIGIN and that the site /api/publish is reachable",
                    )
                    .to_json_string()
            })?;
        Ok(Json(serde_json::json!({
            "appearanceHash": resp.appearance_hash,
            "url": resp.url,
            "iframe": resp.iframe,
            "size": bytes.len(),
        })))
    }
}

const SERVER_INSTRUCTIONS: &str = "K2F MCP: copy template (create) or open/markdown_to_k2f → Editor. insert_node/replace_text/set_role on Editor. save writes .K2F (relock); save_dir writes author source. export_pdf requires path. sign is not a tool.";

#[tool_handler(router = self.tool_router)]
impl ServerHandler for K2fServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_instructions(SERVER_INSTRUCTIONS)
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        let state = self.state.lock().unwrap();
        let resources = list_session_resources(&state);
        async move { Ok(ListResourcesResult::with_all_items(resources)) }
    }

    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_ {
        async move {
            Ok(ListResourceTemplatesResult::with_all_items(
                resource_templates(),
            ))
        }
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResponse, McpError>> + Send + '_ {
        let uri = request.uri;
        let state = self.state.lock().unwrap();
        let result =
            read_resource(&state, &uri).map_err(|e| McpError::invalid_params(e.to_string(), None));
        async move { result }
    }
}

pub async fn run_stdio() -> anyhow::Result<()> {
    let server = K2fServer::new();
    let running = server.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;
    Ok(())
}
