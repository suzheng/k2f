use crate::error::{AgentError, SCHEMA_INVALID};
use boon::{Compiler, Schemas};
use serde_json::Value;
use std::sync::OnceLock;

const AGENT_SCHEMA: &str = include_str!("../profiles/agent_v0.schema.json");

struct Compiled {
    schemas: Schemas,
    agent: boon::SchemaIndex,
}

fn compiled() -> Result<&'static Compiled, AgentError> {
    static CELL: OnceLock<Result<Compiled, String>> = OnceLock::new();
    match CELL.get_or_init(build) {
        Ok(c) => Ok(c),
        Err(e) => Err(AgentError::new(
            SCHEMA_INVALID,
            format!("agent profile compiler: {e}"),
        )),
    }
}

fn build() -> Result<Compiled, String> {
    let mut schemas = Schemas::new();
    let mut compiler = Compiler::new();
    let url = "k2f://sdk/profiles/agent_v0.schema.json";
    let value: Value = serde_json::from_str(AGENT_SCHEMA).map_err(|e| format!("{url}: {e}"))?;
    compiler
        .add_resource(url, value)
        .map_err(|e| format!("add {url}: {e}"))?;
    let agent = compiler
        .compile(url, &mut schemas)
        .map_err(|e| format!("compile agent: {e}"))?;
    Ok(Compiled { schemas, agent })
}

pub fn validate_agent_root_json(v: &Value) -> Result<(), AgentError> {
    let compiled = compiled()?;
    compiled
        .schemas
        .validate(v, compiled.agent)
        .map_err(|e| AgentError::new(SCHEMA_INVALID, format!("agent_v0: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn agent_schema_rejects_color_layout_and_unknown_role() {
        let color = json!({
            "id": "n",
            "role": "body",
            "color": "#ff0000",
            "content": { "type": "text", "value": "x" }
        });
        assert!(validate_agent_root_json(&color).is_err());

        let layout = json!({
            "id": "n",
            "role": "body",
            "layout": { "type": "stack" },
            "content": { "type": "text", "value": "x" }
        });
        assert!(validate_agent_root_json(&layout).is_err());

        let role = json!({
            "id": "n",
            "role": "magic_box",
            "content": { "type": "text", "value": "x" }
        });
        assert!(validate_agent_root_json(&role).is_err());
    }

    #[test]
    fn agent_schema_rejects_image_without_size() {
        let img = json!({
            "id": "logo",
            "role": "body",
            "content": { "type": "image", "value": { "src": "a.png" } }
        });
        assert!(validate_agent_root_json(&img).is_err());
    }
}
