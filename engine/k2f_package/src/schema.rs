use crate::error::PackageError;
use boon::{Compiler, Schemas};
use serde_json::Value;
use std::sync::OnceLock;

const MANIFEST_SCHEMA: &str = include_str!("../schema/manifest.schema.json");
const NODES_SCHEMA: &str = include_str!("../schema/nodes.schema.json");
const STYLES_SCHEMA: &str = include_str!("../schema/styles.schema.json");
const VISUAL_SCHEMA: &str = include_str!("../schema/visual_primitives.schema.json");
const SIGNATURES_SCHEMA: &str = include_str!("../schema/signatures.schema.json");

pub fn bundled_schema_files() -> Vec<(&'static str, &'static str)> {
    crate::paths::FORMAT_SCHEMA_FILES
        .iter()
        .copied()
        .zip([
            MANIFEST_SCHEMA,
            NODES_SCHEMA,
            STYLES_SCHEMA,
            VISUAL_SCHEMA,
            SIGNATURES_SCHEMA,
        ])
        .collect()
}

struct Compiled {
    schemas: Schemas,
    manifest: boon::SchemaIndex,
    nodes: boon::SchemaIndex,
    styles: boon::SchemaIndex,
    signatures: boon::SchemaIndex,
}

fn compiled() -> Result<&'static Compiled, PackageError> {
    static CELL: OnceLock<Result<Compiled, String>> = OnceLock::new();
    match CELL.get_or_init(build) {
        Ok(c) => Ok(c),
        Err(e) => Err(PackageError::Other(format!("schema compiler: {e}"))),
    }
}

fn build() -> Result<Compiled, String> {
    let mut schemas = Schemas::new();
    let mut compiler = Compiler::new();
    for (url, text) in [
        ("k2f://schema/manifest.schema.json", MANIFEST_SCHEMA),
        ("k2f://schema/nodes.schema.json", NODES_SCHEMA),
        ("k2f://schema/styles.schema.json", STYLES_SCHEMA),
        ("k2f://schema/visual_primitives.schema.json", VISUAL_SCHEMA),
        ("k2f://schema/signatures.schema.json", SIGNATURES_SCHEMA),
    ] {
        let value: Value = serde_json::from_str(text).map_err(|e| format!("{url}: {e}"))?;
        compiler
            .add_resource(url, value)
            .map_err(|e| format!("add {url}: {e}"))?;
    }
    let manifest = compiler
        .compile("k2f://schema/manifest.schema.json", &mut schemas)
        .map_err(|e| format!("compile manifest: {e}"))?;
    let nodes = compiler
        .compile("k2f://schema/nodes.schema.json", &mut schemas)
        .map_err(|e| format!("compile nodes: {e}"))?;
    let styles = compiler
        .compile("k2f://schema/styles.schema.json", &mut schemas)
        .map_err(|e| format!("compile styles: {e}"))?;
    let signatures = compiler
        .compile("k2f://schema/signatures.schema.json", &mut schemas)
        .map_err(|e| format!("compile signatures: {e}"))?;
    Ok(Compiled {
        schemas,
        manifest,
        nodes,
        styles,
        signatures,
    })
}

fn validate_against(
    index: boon::SchemaIndex,
    instance: &Value,
    label: &str,
) -> Result<(), PackageError> {
    let compiled = compiled()?;
    compiled
        .schemas
        .validate(instance, index)
        .map_err(|e| PackageError::SchemaInvalid(format!("{label}: {e}")))
}

pub fn validate_manifest_json(v: &Value) -> Result<(), PackageError> {
    validate_against(compiled()?.manifest, v, "manifest.json")
}

pub fn validate_root_json(v: &Value) -> Result<(), PackageError> {
    validate_against(compiled()?.nodes, v, "content/root.json")
}

pub fn validate_content_json(v: &Value, label: &str) -> Result<(), PackageError> {
    validate_against(compiled()?.nodes, v, label)
}

pub fn validate_theme_json(v: &Value) -> Result<(), PackageError> {
    validate_against(compiled()?.styles, v, "styles/theme.json")
}

pub fn validate_signatures_json(v: &Value) -> Result<(), PackageError> {
    validate_against(compiled()?.signatures, v, "signatures/v1.json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extra_field_on_node_is_schema_invalid() {
        let node = json!({
            "id": "root",
            "role": "body",
            "content": { "type": "text", "value": "hi" },
            "bogus": true
        });
        let err = validate_root_json(&node).unwrap_err();
        assert!(
            err.to_string().contains(crate::CODE_SCHEMA_INVALID),
            "got {err}"
        );
    }

    #[test]
    fn hyphenated_node_id_is_schema_invalid() {
        let node = json!({
            "id": "clause-4",
            "role": "body",
            "content": { "type": "text", "value": "hi" }
        });
        let err = validate_root_json(&node).unwrap_err();
        assert!(
            err.to_string().contains(crate::CODE_SCHEMA_INVALID),
            "got {err}"
        );
    }

    #[test]
    fn include_stub_in_container_children_is_valid() {
        let node = json!({
            "id": "root",
            "role": "document",
            "content": {
                "type": "container",
                "value": {
                    "children": [
                        { "include": "content/ch01.json" }
                    ]
                }
            }
        });
        validate_root_json(&node).unwrap();
    }

    #[test]
    fn bundled_schemas_are_format_five_only() {
        let names: Vec<_> = bundled_schema_files().iter().map(|(p, _)| *p).collect();
        assert_eq!(names, crate::paths::FORMAT_SCHEMA_FILES.to_vec());
        assert!(!names.iter().any(|p| p.contains("agent")));
        assert!(!crate::paths::is_schema_path("schema/agent_v0.schema.json"));
        assert!(!crate::paths::is_schema_path(
            "schema/agent_invoice.schema.json"
        ));
        assert!(crate::paths::is_schema_path("schema/nodes.schema.json"));
    }

    #[test]
    fn grid_optional_null_gaps_and_cell_align_are_valid() {
        let node = json!({
            "id": "root",
            "role": "document",
            "content": {
                "type": "container",
                "value": {
                    "children": [{
                        "id": "root.grid",
                        "role": "body",
                        "content": {
                            "type": "container",
                            "value": { "children": [] }
                        },
                        "layout": {
                            "type": "grid",
                            "columns": [{ "pt": 100000 }],
                            "rows": [{ "pt": 100000 }],
                            "gap": 0,
                            "row_gap": null,
                            "column_gap": null,
                            "cell_align": null
                        }
                    }]
                }
            }
        });
        validate_root_json(&node).unwrap();
    }

    #[test]
    fn grid_omitted_gaps_are_valid() {
        let node = json!({
            "id": "root",
            "role": "document",
            "content": {
                "type": "container",
                "value": {
                    "children": [{
                        "id": "root.grid",
                        "role": "body",
                        "content": {
                            "type": "container",
                            "value": { "children": [] }
                        },
                        "layout": {
                            "type": "grid",
                            "columns": [{ "fr": 1 }],
                            "rows": [{ "pt": 50000 }],
                            "gap": 8000
                        }
                    }]
                }
            }
        });
        validate_root_json(&node).unwrap();
    }

    #[test]
    fn grid_optional_width_height_is_valid() {
        let node = json!({
            "id": "root",
            "role": "document",
            "content": {
                "type": "container",
                "value": {
                    "children": [{
                        "id": "root.grid",
                        "role": "body",
                        "content": {
                            "type": "container",
                            "value": { "children": [] }
                        },
                        "layout": {
                            "type": "grid",
                            "columns": [{ "fr": 1 }],
                            "rows": [{ "fr": 1 }, { "pt": 40000 }],
                            "gap": 0,
                            "height": 540000,
                            "width": 960000
                        }
                    }]
                }
            }
        });
        validate_root_json(&node).unwrap();
    }

    #[test]
    fn signatures_schema_rejects_version_two_extra_field_and_empty_list() {
        let valid = json!({
            "version": 1,
            "signatures": [{
                "alg": "ed25519",
                "public_key": "aa".repeat(32),
                "signature": "bb".repeat(64),
                "signed_at": 1_704_067_200
            }]
        });
        validate_signatures_json(&valid).unwrap();

        let mut v2 = valid.clone();
        v2["version"] = json!(2);
        assert!(validate_signatures_json(&v2).is_err());

        let mut extra = valid.clone();
        extra["signatures"][0]["note"] = json!("x");
        assert!(validate_signatures_json(&extra).is_err());

        let empty = json!({ "version": 1, "signatures": [] });
        assert!(validate_signatures_json(&empty).is_err());
    }
}
