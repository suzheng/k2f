use crate::theme::Theme;
use crate::{LayoutContext, LayoutEngine};
use k2f_core::{
    canonical_json_string, engine_commit_sha, engine_version, expand_manifest_tables_with_assets,
    hash_appearance_binding, hash_manifest_semantic, normalize_manifest_nfc,
    semantic_tree_needs_assets, validate_manifest_node_ids, validate_manifest_running_blocks,
    validate_semantic_tree, validate_semantic_tree_with_theme_vocab, AppearanceHashInput,
    AssetsMap, CanvasMode, LockFile, Manifest, PageConfig, Pt, SemanticNode, ThemeVocab,
};
use serde::Deserialize;
use std::collections::BTreeMap;

pub fn compile_chunk(
    content_json: &str,
    theme_json: &str,
    font_data: &[u8],
    assets: Option<&AssetsMap>,
) -> Result<String, String> {
    if font_data.is_empty() {
        return Err(
            "Missing embedded font blob (font_data is empty). Deterministic rendering requires embedded fonts."
                .to_string(),
        );
    }
    let mut fonts = BTreeMap::new();
    fonts.insert("default".to_string(), font_data.to_vec());
    compile_chunk_with_fonts(content_json, theme_json, &fonts, assets)
}

pub fn compile_chunk_with_fonts(
    content_json: &str,
    theme_json: &str,
    fonts: &BTreeMap<String, Vec<u8>>,
    assets: Option<&AssetsMap>,
) -> Result<String, String> {
    let manifest = parse_content_json(content_json)?;
    let lock = compile_outcome(manifest, theme_json, fonts, assets)?.lock;
    canonical_json_string(&lock).map_err(|e| format!("Serialization error: {e}"))
}

pub struct CompileOutcome {
    pub lock: LockFile,
    pub diags: Vec<crate::slack::LayoutDiag>,
}

pub fn compile_manifest(
    manifest: Manifest,
    theme_json: &str,
    fonts: &BTreeMap<String, Vec<u8>>,
    assets: Option<&AssetsMap>,
) -> Result<LockFile, String> {
    Ok(compile_outcome(manifest, theme_json, fonts, assets)?.lock)
}

pub fn compile_outcome(
    mut manifest: Manifest,
    theme_json: &str,
    fonts: &BTreeMap<String, Vec<u8>>,
    assets: Option<&AssetsMap>,
) -> Result<CompileOutcome, String> {
    if !fonts.keys().any(|p| k2f_core::is_font_face_path(p)) {
        return Err("FONT_MISSING: package has no embedded fonts under assets/fonts/".to_string());
    }

    let theme_value: serde_json::Value =
        serde_json::from_str(theme_json).map_err(|e| format!("Theme JSON error: {e}"))?;
    let theme: Theme = serde_json::from_value(theme_value.clone())
        .map_err(|e| format!("Theme JSON error: {e}"))?;
    let theme_vocab: ThemeVocab =
        serde_json::from_value(theme_value).map_err(|e| format!("Theme JSON error: {e}"))?;

    if let Some(assets) = assets {
        k2f_core::validate_svg_assets(assets)?;
        manifest = expand_manifest_tables_with_assets(&manifest, assets)?;
    } else if semantic_tree_needs_assets(&manifest.root) {
        return Err(
            "Asset-backed table data requires compile_chunk_with_assets (an explicit assets map)."
                .to_string(),
        );
    }

    normalize_manifest_nfc(&mut manifest);

    validate_semantic_tree(&manifest.root).map_err(|e| format!("Content validation error: {e}"))?;
    validate_semantic_tree_with_theme_vocab(&manifest.root, &theme_vocab)
        .map_err(|e| format!("Content validation error: {e}"))?;
    validate_manifest_running_blocks(&manifest, &theme_vocab)
        .map_err(|e| format!("Content validation error: {e}"))?;
    validate_manifest_node_ids(&manifest).map_err(|e| format!("Content validation error: {e}"))?;

    let font_lib = crate::fonts::load_font_library(fonts)?;
    crate::fonts::validate_theme_fonts(&theme, &font_lib)?;

    let ctx = LayoutContext::new(&font_lib, &theme);
    let layout_result = LayoutEngine::layout(&manifest, &ctx)?;
    let diags = crate::slack::layout_slack_diags(&manifest, &layout_result, &theme);
    let render_plan = crate::render_plan::build_render_plan(&manifest, &layout_result, &theme)?;

    let content_hash = hash_manifest_semantic(&manifest);
    let appearance_hash = hash_appearance_binding(&AppearanceHashInput {
        content_hash: &content_hash,
        theme_json,
        fonts,
        page_config: &manifest.page_config,
        engine_version: engine_version(),
        engine_commit_sha: engine_commit_sha(),
    })?;

    Ok(CompileOutcome {
        lock: LockFile {
            engine_version: engine_version().to_string(),
            engine_commit_sha: engine_commit_sha().to_string(),
            content_hash,
            appearance_hash,
            geometry: layout_result,
            render_plan,
        },
        diags,
    })
}

fn parse_content_json(content_json: &str) -> Result<Manifest, String> {
    let mut deserializer = serde_json::Deserializer::from_str(content_json);
    deserializer.disable_recursion_limit();
    match Manifest::deserialize(&mut deserializer) {
        Ok(m) => Ok(m),
        Err(_) => {
            let mut d2 = serde_json::Deserializer::from_str(content_json);
            d2.disable_recursion_limit();
            let node = SemanticNode::deserialize(&mut d2)
                .map_err(|e| format!("Content JSON error: {e}"))?;
            Ok(Manifest {
                title: "Untitled".to_string(),
                canvas_mode: CanvasMode::Paged,
                page_config: PageConfig {
                    width: Pt(595000),
                    height: Pt(842000),
                    margin: [Pt(72000); 4],
                },
                root: node,
                running_blocks: vec![],
            })
        }
    }
}
