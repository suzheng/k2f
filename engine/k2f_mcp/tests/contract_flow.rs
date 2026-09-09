use k2f_mcp::{load_catalog, mcp_tool_names, tool_descriptions, K2fServer, K2fState};
use k2f_paint::{Banner, OpenedDocument};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn contract_path() -> PathBuf {
    repo_root().join("examples/published/contract.K2F")
}

const CLAUSE_4_AFTER: &str = "本协议自双方签署之日起生效。任何一方终止须提前三十日书面通知对方。终止前须完成已确认工作并结清对应发票。终止不影响本文件中机密、报酬与签署栏条款的效力。";

#[test]
fn mcp_tool_names_match_schema() {
    let names = mcp_tool_names();
    assert_eq!(names.len(), 16);
    assert!(!names.contains(&"sign".to_string()));
}

#[test]
fn catalog_save_and_export_require_path() {
    let catalog = load_catalog();
    for name in ["save", "export_pdf"] {
        let tool = catalog.tools.iter().find(|t| t.name == name).unwrap();
        let required = tool.required_fields();
        assert!(
            required.iter().any(|f| f == "path"),
            "{name} must require path, got {required:?}"
        );
    }
}

#[test]
fn contract_clause_edit_flow_via_state() {
    let mut state = K2fState::new("http://127.0.0.1:0");
    let open_json = state
        .open(contract_path().to_str().unwrap())
        .expect("open contract");
    let session_id = open_json["session_id"].as_str().unwrap().to_string();

    state
        .replace_text(&session_id, "contract.clause_4", CLAUSE_4_AFTER)
        .unwrap();
    state
        .set_role(&session_id, "contract.clause_4", "critical_warning", None)
        .unwrap();

    let diff = state.diff(&session_id).unwrap();
    let diff_arr = diff.as_array().unwrap();
    assert_eq!(diff_arr.len(), 1);
    assert_eq!(diff_arr[0]["id"], "contract.clause_4");

    let out = tempfile_path("contract-edited.K2F");
    let save_json = state
        .save(&session_id, None, out.to_str().unwrap())
        .unwrap();
    assert!(save_json.get("bytes_base64").is_none());
    assert_eq!(save_json["banner"].as_str().unwrap(), "UNSIGNED");
    assert!(save_json["size"].as_u64().unwrap() > 0);
    assert_eq!(
        save_json["path"].as_str().unwrap(),
        out.display().to_string()
    );

    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(
        OpenedDocument::open(&bytes).unwrap().banner(),
        Banner::Unsigned
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn insert_and_delete_roundtrip() {
    let mut state = K2fState::new("http://127.0.0.1:0");
    let open_json = state.open(contract_path().to_str().unwrap()).unwrap();
    let session_id = open_json["session_id"].as_str().unwrap().to_string();
    let node = serde_json::json!({
        "id": "contract.mcp_probe",
        "role": "body",
        "content": { "type": "text", "value": "probe" }
    });
    state.insert_node(&session_id, "root", 1, &node).unwrap();
    state
        .delete_node(&session_id, "contract.mcp_probe")
        .unwrap();
}

#[test]
fn create_from_template_then_insert() {
    let mut state = K2fState::new("http://127.0.0.1:0");
    let dest = tempfile_path("mcp-create-legal");
    let _ = std::fs::remove_dir_all(&dest);
    let created = state
        .create("Empty", dest.to_str().unwrap(), "legal", "A4")
        .unwrap();
    assert_eq!(created["kind"].as_str().unwrap(), "editor");
    let session_id = created["session_id"].as_str().unwrap();
    let node = serde_json::json!({
        "id": "doc.body",
        "role": "body",
        "content": { "type": "text", "value": "hello" }
    });
    state.insert_node(session_id, "root", 0, &node).unwrap();
    let _ = std::fs::remove_dir_all(&dest);
}

#[test]
fn editor_validate_checks_semantic_tree() {
    let mut state = K2fState::new("http://127.0.0.1:0");
    let open_json = state.open(contract_path().to_str().unwrap()).unwrap();
    let session_id = open_json["session_id"].as_str().unwrap();
    let v = state.validate(session_id).unwrap();
    assert_eq!(
        v["checks"].as_str().unwrap(),
        "semantic_tree_and_theme_vocab"
    );
}

#[test]
fn publish_dry_run_skips_http() {
    let mut state = K2fState::new("http://127.0.0.1:9");
    let open_json = state.open(contract_path().to_str().unwrap()).unwrap();
    let session_id = open_json["session_id"].as_str().unwrap().to_string();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let out = rt.block_on(state.publish(&session_id, None, true)).unwrap();
    assert_eq!(out["dry_run"], true);
    assert!(out["appearance_hash"].as_str().unwrap().len() > 10);
    assert!(out.get("url").is_none());
}

#[tokio::test]
async fn tools_list_excludes_sign() {
    use rmcp::ServiceExt;

    let (server_transport, client_transport) = tokio::io::duplex(4096);
    let server = K2fServer::new();
    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });

    let client = ().serve(client_transport).await.expect("client connect");
    let tools = client.list_tools(None).await.expect("list_tools");
    let names: Vec<String> = tools.tools.iter().map(|t| t.name.to_string()).collect();
    assert!(!names.contains(&"sign".to_string()));
    assert_eq!(names.len(), 16);

    let catalog_desc = tool_descriptions();
    for tool in &tools.tools {
        let expected = catalog_desc
            .get(tool.name.as_ref())
            .unwrap_or_else(|| panic!("missing catalog entry for {}", tool.name));
        let got = tool.description.as_deref().unwrap_or("");
        assert_eq!(got, expected, "description drift for tool {}", tool.name);
        assert!(
            tool.output_schema.is_some(),
            "tool {} should advertise outputSchema",
            tool.name
        );
        if tool.name == "save" || tool.name == "export_pdf" {
            let schema = tool.input_schema.as_ref();
            let required = schema
                .get("required")
                .and_then(|r| r.as_array())
                .cloned()
                .unwrap_or_default();
            assert!(
                required.iter().any(|v| v.as_str() == Some("path")),
                "{} inputSchema must require path: {:?}",
                tool.name,
                required
            );
        }
        if tool.name == "insert_node" {
            let node = tool
                .input_schema
                .get("properties")
                .and_then(|p| p.get("node"))
                .expect("insert_node.node");
            let ty = node.get("type").and_then(|t| t.as_str()).or_else(|| {
                // $ref to a definition that is an object
                node.get("$ref").and_then(|_| Some("object"))
            });
            assert!(
                ty == Some("object")
                    || node.get("additionalProperties").is_some()
                    || node.get("$ref").is_some(),
                "insert_node.node must be object-like, got {node}"
            );
        }
    }

    let info = client.peer_info().expect("peer info");
    let instructions = info.instructions.as_deref().unwrap_or("");
    let catalog_instr = load_catalog().instructions.as_deref().unwrap_or("");
    assert_eq!(instructions, catalog_instr);

    client.cancel().await.ok();
    server_handle.await.unwrap().unwrap();
}

#[test]
fn empty_path_is_path_required() {
    let mut state = K2fState::new("http://127.0.0.1:0");
    let open_json = state.open(contract_path().to_str().unwrap()).unwrap();
    let session_id = open_json["session_id"].as_str().unwrap();
    let err = state.save(session_id, None, "").unwrap_err();
    assert_eq!(err.code, "PATH_REQUIRED");
    let err = state.export_pdf(session_id, "  ", None).unwrap_err();
    assert_eq!(err.code, "PATH_REQUIRED");
}

#[test]
fn server_state_is_shareable() {
    let _server = K2fServer::new();
    let shared = Arc::new(Mutex::new(K2fState::new("http://127.0.0.1:0")));
    let _guard = shared.lock().unwrap();
}

fn tempfile_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("k2f-mcp-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir.join(name)
}
