use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

pub const MCP_TOOL_SCHEMA_JSON: &str = include_str!("../mcp_tools.json");

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub forbidden: Vec<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub rules: Vec<CatalogRule>,
    pub tools: Vec<CatalogTool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CatalogRule {
    pub id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct CatalogTool {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub mcp: bool,
    #[serde(default)]
    pub input_schema: Option<Value>,
    #[serde(rename = "inputSchema")]
    pub input_schema_camel: Option<Value>,
    #[serde(default)]
    pub op: Option<String>,
}

impl CatalogTool {
    pub fn input_schema(&self) -> Option<&Value> {
        self.input_schema_camel
            .as_ref()
            .or(self.input_schema.as_ref())
    }

    pub fn required_fields(&self) -> Vec<String> {
        let Some(schema) = self.input_schema() else {
            return Vec::new();
        };
        schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(MCP_TOOL_SCHEMA_JSON).expect("mcp_tools.json must parse")
    })
}

pub fn load_catalog() -> &'static Catalog {
    catalog()
}

pub fn mcp_tool_names() -> Vec<String> {
    catalog()
        .tools
        .iter()
        .filter(|t| t.mcp)
        .map(|t| t.name.clone())
        .collect()
}

pub fn tool_descriptions() -> HashMap<String, String> {
    catalog()
        .tools
        .iter()
        .filter(|t| t.mcp)
        .map(|t| (t.name.clone(), t.description.clone()))
        .collect()
}

pub fn assert_catalog_matches_handlers(handler_names: &[&str]) {
    let mut expected = mcp_tool_names();
    expected.sort();
    let mut got: Vec<String> = handler_names.iter().map(|s| s.to_string()).collect();
    got.sort();
    if expected != got {
        panic!(
            "MCP tool handlers {:?} do not match mcp_tools.json mcp=true tools {:?}",
            got, expected
        );
    }
    for name in &catalog().forbidden {
        if handler_names.contains(&name.as_str()) {
            panic!("forbidden tool {name} must not be registered");
        }
    }
}
