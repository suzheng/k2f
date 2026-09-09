use k2f_package::validate_theme_json;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

const THEMED_TEMPLATES: &[&str] = &["invoice", "legal", "clinical_summary", "report"];

fn templates_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates")
}

fn load_theme_json(id: &str) -> Value {
    let dir = templates_root().join(id);
    let raw = fs::read_to_string(dir.join("styles/theme.json")).unwrap();
    serde_json::from_str(&raw).unwrap()
}

fn minimal_theme(extra_role_deco: Value, extra_primitives: Value) -> Value {
    let mut theme = json!({
        "palette": { "ink": "#111111", "paper": "#FFFFFF" },
        "primitives": extra_primitives,
        "roles": {
            "default": {
                "font_family": "default",
                "font_size": 12000,
                "line_height_mult": 1600,
                "color": "ink"
            },
            "body": {
                "font_family": "default",
                "font_size": 12000,
                "line_height_mult": 1600,
                "color": "ink",
                "box_decoration": extra_role_deco
            }
        }
    });
    if theme["primitives"].as_object().unwrap().is_empty() {
        theme.as_object_mut().unwrap().remove("primitives");
    }
    theme
}

#[test]
fn fixture_template_ids_include_blank_and_four_themes() {
    for id in THEMED_TEMPLATES.iter().chain(["blank"].iter()) {
        assert!(
            templates_root().join(id).is_dir(),
            "missing fixture template {id}"
        );
    }
}

#[test]
fn official_themes_match_styles_schema() {
    for id in THEMED_TEMPLATES {
        let v = load_theme_json(id);
        validate_theme_json(&v).unwrap_or_else(|e| panic!("{id}: {e}"));
    }
}

#[test]
fn official_themes_define_named_visual_atoms() {
    for id in THEMED_TEMPLATES {
        let v = load_theme_json(id);
        let name = id;
        let p = &v["primitives"];
        assert_eq!(p["corners"]["none"], 0, "{name} corner.none");
        assert_eq!(p["corners"]["small"], 4000, "{name} corner.small");
        assert_eq!(p["corners"]["medium"], 12000, "{name} corner.medium");
        assert_eq!(p["corners"]["large"], 24000, "{name} corner.large");
        assert_eq!(p["corners"]["full"], 9999000, "{name} corner.full");
        assert!(
            p["shadows"]["elevation.1"]["layers"].is_array(),
            "{name} elevation.1"
        );
        assert!(
            p["shadows"]["elevation.2"]["layers"].is_array(),
            "{name} elevation.2"
        );
        assert!(
            p["shadows"]["elevation.3"]["layers"].is_array(),
            "{name} elevation.3"
        );
        assert_eq!(p["blurs"]["background"]["radius_pt"], 20000, "{name} blur");
        assert_eq!(
            p["borders"]["subtle"]["width_pt"], 500,
            "{name} border.subtle"
        );
        assert_eq!(
            p["borders"]["contrast"]["width_pt"], 1000,
            "{name} border.contrast"
        );
        assert!(
            p["surfaces"]["glass_light"].is_object(),
            "{name} glass_light"
        );
        assert!(p["surfaces"]["glass_dark"].is_object(), "{name} glass_dark");
        assert!(p["surfaces"]["inverse"].is_object(), "{name} inverse");
        assert!(
            v["palette"]["disabled"].as_str().unwrap().ends_with("61"),
            "{name} disabled"
        );
        assert_eq!(
            v["roles"]["h1"]["letter_spacing_pt"], -500,
            "{name} h1 tracking"
        );
        assert_eq!(
            v["roles"]["warning"]["box_decoration"]["corner_radius"], "small",
            "{name} warning corner"
        );
        assert_eq!(
            v["roles"]["table_header_cell"]["box_decoration"]["corner_radius"], "small",
            "{name} table header corner"
        );
        assert_eq!(
            v["roles"]["card"]["variants"]["glass"]["box_decoration"]["blur"], "background",
            "{name} card.glass"
        );
        assert_eq!(
            v["roles"]["card"]["variants"]["raised"]["box_decoration"]["shadow"], "elevation.1",
            "{name} card.raised"
        );
    }
}

#[test]
fn inline_fill_in_theme_decoration_fails_schema() {
    let v = minimal_theme(
        json!({ "background": { "type": "solid", "color": "#FFFFFF" } }),
        json!({}),
    );
    let err = validate_theme_json(&v).unwrap_err().to_string();
    assert!(
        err.contains("SCHEMA_INVALID") || err.to_lowercase().contains("schema"),
        "{err}"
    );
}

#[test]
fn primitives_colors_fails_schema() {
    let mut v = minimal_theme(json!({}), json!({ "colors": { "x": "#000000" } }));
    v["roles"]["body"]
        .as_object_mut()
        .unwrap()
        .remove("box_decoration");
    let err = validate_theme_json(&v).unwrap_err().to_string();
    assert!(
        err.contains("SCHEMA_INVALID") || err.to_lowercase().contains("schema"),
        "{err}"
    );
}
