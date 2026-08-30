use crate::resolved_style::{padding_for_role_variant, resolve_box_decoration};
use crate::theme::resolve_theme_decoration;
use crate::Theme;
use k2f_core::{
    BlurRef, BoxDecoration, Fill, FillRef, GeometryNode, LayoutResult, Manifest, NodeContent, Page,
    PageRenderPlan, PaintOp, Pt, Rect, RenderPlan, TableDataSource,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum NodeContentInfo {
    Text,
    Math,
    Image { src: String },
    TableReference { source: String, view_mode: String },
    Container,
}

#[derive(Debug, Clone)]
struct SemanticInfo {
    role: String,
    variant: Option<String>,
    content: NodeContentInfo,
}

pub fn build_render_plan(
    manifest: &Manifest,
    layout: &LayoutResult,
    theme: &Theme,
) -> Result<RenderPlan, String> {
    let mut info_by_id: HashMap<String, SemanticInfo> = HashMap::new();
    index_semantic_tree(&manifest.root, &mut info_by_id);
    for rb in &manifest.running_blocks {
        index_semantic_tree(&rb.node, &mut info_by_id);
    }

    let mut pages = Vec::with_capacity(layout.pages.len());
    for page in &layout.pages {
        let mut ops: Vec<PaintOp> = Vec::new();
        append_page_background_op(manifest, page, theme, &mut ops)?;
        for child in &page.root.children {
            append_ops_for_geometry(child, &info_by_id, theme, &mut ops)?;
        }
        pages.push(PageRenderPlan {
            index: page.index,
            ops,
        });
    }

    Ok(RenderPlan {
        compositing: Default::default(),
        pages,
    })
}

fn append_page_background_op(
    manifest: &Manifest,
    page: &Page,
    theme: &Theme,
    ops: &mut Vec<PaintOp>,
) -> Result<(), String> {
    let Some(mut named) =
        resolve_box_decoration(&manifest.root.role, manifest.root.variant.as_deref(), theme)
    else {
        return Ok(());
    };

    named.padding_pt = None;
    if !named.has_paint_refs() {
        return Ok(());
    }
    let mut decoration = resolve_theme_decoration(&named, theme)?;

    if let Some(blur) = inline_blur(&decoration) {
        ops.push(PaintOp::BackdropBlur {
            node_id: format!("{}::page_{}::background", manifest.root.id, page.index),
            rect: Rect {
                x: Pt::ZERO,
                y: Pt::ZERO,
                width: page.width,
                height: page.height,
            },
            radius_pt: blur.radius_pt,
            corner_radius_pt: decoration.corner_radius_pt,
        });
        decoration.blur = None;
    }

    if !has_paint_decoration(&decoration) {
        return Ok(());
    }

    ops.push(PaintOp::DrawBox {
        node_id: format!("{}::page_{}::background", manifest.root.id, page.index),
        rect: Rect {
            x: Pt::ZERO,
            y: Pt::ZERO,
            width: page.width,
            height: page.height,
        },
        decoration,
    });
    Ok(())
}

fn index_semantic_tree(node: &k2f_core::SemanticNode, out: &mut HashMap<String, SemanticInfo>) {
    let content = match &node.content {
        NodeContent::Text(_) => NodeContentInfo::Text,
        NodeContent::CodeBlock(_) => NodeContentInfo::Text,
        NodeContent::Math(_) => NodeContentInfo::Math,
        NodeContent::Image { src, .. } => NodeContentInfo::Image { src: src.clone() },
        NodeContent::TableReference {
            source, view_mode, ..
        } => NodeContentInfo::TableReference {
            source: source.clone(),
            view_mode: view_mode.clone(),
        },
        NodeContent::Container { .. } => NodeContentInfo::Container,
        // v1: treat native tables as container-like for painting (table decor paints on the table node;
        // cell content paints via cell nodes once semantic indexing is extended to recurse into cells).
        NodeContent::Table(_) => NodeContentInfo::Container,
    };

    out.insert(
        node.id.clone(),
        SemanticInfo {
            role: node.role.clone(),
            variant: node.variant.clone(),
            content,
        },
    );

    match &node.content {
        NodeContent::Container { children } => {
            for c in children {
                index_semantic_tree(c, out);
            }
        }
        NodeContent::Table(spec) => {
            // Recurse into table cells for semantic indexing and painting
            if let TableDataSource::Inline { rows } = &spec.data {
                for row in rows {
                    for cell in row {
                        index_semantic_tree(cell, out);
                    }
                }
            }
        }
        _ => {}
    }
}

fn append_ops_for_geometry(
    geo: &GeometryNode,
    info_by_id: &HashMap<String, SemanticInfo>,
    theme: &Theme,
    ops: &mut Vec<PaintOp>,
) -> Result<(), String> {
    let Some(info) = info_by_id.get(&geo.id) else {
        for child in &geo.children {
            append_ops_for_geometry(child, info_by_id, theme, ops)?;
        }
        return Ok(());
    };

    let rect = Rect {
        x: geo.x,
        y: geo.y,
        width: geo.width,
        height: geo.height,
    };

    if let Some(mut named) = resolve_box_decoration(&info.role, info.variant.as_deref(), theme) {
        named.padding_pt = None;
        if named.has_paint_refs() {
            let mut decoration = resolve_theme_decoration(&named, theme)?;
            if let Some(blur) = inline_blur(&decoration) {
                ops.push(PaintOp::BackdropBlur {
                    node_id: geo.id.clone(),
                    rect: Rect {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    },
                    radius_pt: blur.radius_pt,
                    corner_radius_pt: decoration.corner_radius_pt,
                });
                decoration.blur = None;
            }

            if has_paint_decoration(&decoration) {
                ops.push(PaintOp::DrawBox {
                    node_id: geo.id.clone(),
                    rect: Rect {
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                    },
                    decoration,
                });
            }
        }
    }

    for child in &geo.children {
        append_ops_for_geometry(child, info_by_id, theme, ops)?;
    }

    match &info.content {
        NodeContentInfo::Container => {}
        NodeContentInfo::Text => {
            emit_math_rules(geo, info, theme, ops);
            ops.push(PaintOp::DrawText {
                node_id: geo.id.clone(),
                rect,
                runs: geo.text_runs.clone(),
            });
        }
        NodeContentInfo::Math => {
            emit_math_rules(geo, info, theme, ops);
            ops.push(PaintOp::DrawText {
                node_id: geo.id.clone(),
                rect,
                runs: geo.text_runs.clone(),
            });
        }
        NodeContentInfo::Image { src } => {
            let image_rect = if let Ok(padding) =
                padding_for_role_variant(&info.role, info.variant.as_deref(), theme)
            {
                inset_rect(rect, padding)
            } else {
                rect
            };
            ops.push(PaintOp::DrawImage {
                node_id: geo.id.clone(),
                rect: image_rect,
                src: src.clone(),
            });
        }
        NodeContentInfo::TableReference { source, view_mode } => {
            ops.push(PaintOp::DrawTableReference {
                node_id: geo.id.clone(),
                rect,
                source: source.clone(),
                view_mode: view_mode.clone(),
            });
        }
    }
    Ok(())
}

fn emit_math_rules(geo: &GeometryNode, info: &SemanticInfo, theme: &Theme, ops: &mut Vec<PaintOp>) {
    if geo.fill_rects.is_empty() {
        return;
    }
    let color = resolve_color_ref_for_plan(
        &crate::resolved_style::resolve_text_style(&info.role, info.variant.as_deref(), theme)
            .color,
        theme,
    );
    for (i, r) in geo.fill_rects.iter().enumerate() {
        ops.push(PaintOp::DrawBox {
            node_id: format!("{}::rule_{i}", geo.id),
            rect: Rect {
                x: r.x,
                y: r.y,
                width: r.width,
                height: r.height,
            },
            decoration: BoxDecoration {
                background: Some(FillRef::Inline(Fill::Solid {
                    color: color.clone(),
                })),
                ..Default::default()
            },
        });
    }
}

fn has_paint_decoration(decoration: &BoxDecoration) -> bool {
    decoration.background.is_some()
        || decoration.border.is_some()
        || decoration.corner_radius_pt.is_some()
        || decoration.shadow.is_some()
        || decoration.blur.is_some()
}

fn inline_blur(decoration: &BoxDecoration) -> Option<k2f_core::Blur> {
    match decoration.blur.as_ref()? {
        BlurRef::Inline(b) => Some(b.clone()),
        BlurRef::Ref(_) => None,
    }
}

fn inset_rect(rect: Rect, padding: crate::resolved_style::EdgeInsets) -> Rect {
    let width = (rect.width - padding.horizontal()).max(Pt::ZERO);
    let height = (rect.height - padding.vertical()).max(Pt::ZERO);
    Rect {
        x: rect.x + padding.left,
        y: rect.y + padding.top,
        width,
        height,
    }
}

pub(crate) fn resolve_color_ref_for_plan(color: &str, theme: &Theme) -> String {
    crate::theme::resolve_palette_color(color, theme)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k2f_core::{
        CanvasMode, GeometryNode, LayoutResult, Manifest, NodeContent, PageConfig,
        RunningBlockNode, RunningBlockPosition, SemanticNode,
    };

    #[test]
    fn page_background_op_is_emitted_first_when_root_has_decoration() {
        let manifest = Manifest {
            title: "Background".to_string(),
            canvas_mode: CanvasMode::Paged,
            page_config: PageConfig {
                width: Pt(100_000),
                height: Pt(200_000),
                margin: [Pt::ZERO; 4],
            },
            root: SemanticNode {
                id: "root".to_string(),
                role: "document".to_string(),
                variant: Some("with_background".to_string()),
                preserve_whitespace: None,
                list_id: None,
                depth: None,
                marker_type: None,
                content: NodeContent::Container {
                    children: vec![SemanticNode {
                        id: "child".to_string(),
                        role: "body".to_string(),
                        variant: None,
                        preserve_whitespace: None,
                        list_id: None,
                        depth: None,
                        marker_type: None,
                        content: NodeContent::Text("hi".to_string()),
                        modifiers: vec![],
                        layout: None,
                        ..Default::default()
                    }],
                },
                modifiers: vec![],
                layout: None,
                ..Default::default()
            },
            running_blocks: vec![],
        };

        let theme: Theme = serde_json::from_str(
            r##"
{
  "palette": { "black": "#000000" },
  "primitives": { "surfaces": { "global_background": { "type": "solid", "color": "#FFFFFF" } } },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "document": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black",
      "variants": {
        "with_background": { "box_decoration": { "background": "global_background" } }
      }
    }
  }
}
"##,
        )
        .unwrap();

        let layout = LayoutResult {
            pages: vec![Page {
                index: 0,
                width: Pt(100_000),
                height: Pt(200_000),
                root: GeometryNode {
                    id: "root_page_0".to_string(),
                    x: Pt::ZERO,
                    y: Pt::ZERO,
                    width: Pt(100_000),
                    height: Pt(200_000),
                    glyphs: vec![],
                    text_runs: vec![],
                    fill_rects: vec![],
                    children: vec![GeometryNode {
                        id: "child".to_string(),
                        x: Pt(10_000),
                        y: Pt(10_000),
                        width: Pt(50_000),
                        height: Pt(20_000),
                        glyphs: vec![],
                        text_runs: vec![],
                        fill_rects: vec![],
                        children: vec![],
                    }],
                },
            }],
        };

        let plan = build_render_plan(&manifest, &layout, &theme).unwrap();
        let ops = &plan.pages[0].ops;

        assert!(!ops.is_empty());
        match &ops[0] {
            PaintOp::DrawBox {
                node_id,
                rect,
                decoration,
            } => {
                assert_eq!(node_id, "root::page_0::background");
                assert_eq!(rect.width, Pt(100_000));
                assert_eq!(rect.height, Pt(200_000));
                assert!(decoration.background.is_some());
            }
            other => panic!("expected first op to be DrawBox, got {other:?}"),
        }
    }

    #[test]
    fn overlay_stacking_preserves_source_order_in_ops() {
        let manifest = Manifest {
            title: "Overlay".to_string(),
            canvas_mode: CanvasMode::Paged,
            page_config: PageConfig {
                width: Pt(100_000),
                height: Pt(200_000),
                margin: [Pt::ZERO; 4],
            },
            root: SemanticNode {
                id: "root".to_string(),
                role: "document".to_string(),
                variant: None,
                preserve_whitespace: None,
                list_id: None,
                depth: None,
                marker_type: None,
                content: NodeContent::Container {
                    children: vec![SemanticNode {
                        id: "scene".to_string(),
                        role: "document".to_string(),
                        variant: None,
                        preserve_whitespace: None,
                        list_id: None,
                        depth: None,
                        marker_type: None,
                        content: NodeContent::Container {
                            children: vec![
                                SemanticNode {
                                    id: "img".to_string(),
                                    role: "body".to_string(),
                                    variant: None,
                                    preserve_whitespace: None,
                                    list_id: None,
                                    depth: None,
                                    marker_type: None,
                                    content: NodeContent::Image {
                                        src: "asset://img".to_string(),
                                        width: Pt(50_000),
                                        height: Pt(50_000),
                                    },
                                    modifiers: vec![],
                                    layout: None,
                                    ..Default::default()
                                },
                                SemanticNode {
                                    id: "card".to_string(),
                                    role: "card".to_string(),
                                    variant: Some("glass".to_string()),
                                    preserve_whitespace: None,
                                    list_id: None,
                                    depth: None,
                                    marker_type: None,
                                    content: NodeContent::Container {
                                        children: vec![SemanticNode {
                                            id: "card_text".to_string(),
                                            role: "body".to_string(),
                                            variant: None,
                                            preserve_whitespace: None,
                                            list_id: None,
                                            depth: None,
                                            marker_type: None,
                                            content: NodeContent::Text("hi".to_string()),
                                            modifiers: vec![],
                                            layout: None,
                                            ..Default::default()
                                        }],
                                    },
                                    modifiers: vec![],
                                    layout: None,
                                    ..Default::default()
                                },
                            ],
                        },
                        modifiers: vec![],
                        layout: None,
                        ..Default::default()
                    }],
                },
                modifiers: vec![],
                layout: None,
                ..Default::default()
            },
            running_blocks: vec![],
        };

        let theme: Theme = serde_json::from_str(
            r##"
{
  "palette": { "black": "#000000", "white": "#FFFFFF" },
  "primitives": { "surfaces": { "glass_surface": { "type": "solid", "color": "#FFFFFFCC" } } },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "document": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "card": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black",
      "variants": {
        "glass": { "box_decoration": { "background": "glass_surface" } }
      }
    }
  }
}
"##,
        )
        .unwrap();

        let layout = LayoutResult {
            pages: vec![Page {
                index: 0,
                width: Pt(100_000),
                height: Pt(200_000),
                root: GeometryNode {
                    id: "root_page_0".to_string(),
                    x: Pt::ZERO,
                    y: Pt::ZERO,
                    width: Pt(100_000),
                    height: Pt(200_000),
                    glyphs: vec![],
                    text_runs: vec![],
                    fill_rects: vec![],
                    children: vec![GeometryNode {
                        id: "scene".to_string(),
                        x: Pt::ZERO,
                        y: Pt::ZERO,
                        width: Pt(60_000),
                        height: Pt(60_000),
                        glyphs: vec![],
                        text_runs: vec![],
                        fill_rects: vec![],
                        children: vec![
                            GeometryNode {
                                id: "img".to_string(),
                                x: Pt::ZERO,
                                y: Pt::ZERO,
                                width: Pt(50_000),
                                height: Pt(50_000),
                                glyphs: vec![],
                                text_runs: vec![],
                                fill_rects: vec![],
                                children: vec![],
                            },
                            GeometryNode {
                                id: "card".to_string(),
                                x: Pt::ZERO,
                                y: Pt::ZERO,
                                width: Pt(40_000),
                                height: Pt(30_000),
                                glyphs: vec![],
                                text_runs: vec![],
                                fill_rects: vec![],
                                children: vec![GeometryNode {
                                    id: "card_text".to_string(),
                                    x: Pt::ZERO,
                                    y: Pt::ZERO,
                                    width: Pt(10_000),
                                    height: Pt(10_000),
                                    glyphs: vec![],
                                    text_runs: vec![],
                                    fill_rects: vec![],
                                    children: vec![],
                                }],
                            },
                        ],
                    }],
                },
            }],
        };

        let plan = build_render_plan(&manifest, &layout, &theme).unwrap();
        let ops = &plan.pages[0].ops;

        let img_op_idx = ops
            .iter()
            .position(|op| matches!(op, PaintOp::DrawImage { node_id, .. } if node_id == "img"))
            .unwrap();
        let card_box_idx = ops
            .iter()
            .position(|op| matches!(op, PaintOp::DrawBox { node_id, .. } if node_id == "card"))
            .unwrap();
        assert!(
            img_op_idx < card_box_idx,
            "expected image to paint before glass card box"
        );
    }

    #[test]
    fn running_block_semantics_are_indexed_for_painting() {
        let manifest = Manifest {
            title: "RB".to_string(),
            canvas_mode: CanvasMode::Paged,
            page_config: PageConfig {
                width: Pt(100_000),
                height: Pt(200_000),
                margin: [Pt::ZERO; 4],
            },
            root: SemanticNode {
                id: "root".to_string(),
                role: "document".to_string(),
                variant: None,
                preserve_whitespace: None,
                list_id: None,
                depth: None,
                marker_type: None,
                content: NodeContent::Container { children: vec![] },
                modifiers: vec![],
                layout: None,
                ..Default::default()
            },
            running_blocks: vec![RunningBlockNode {
                position: RunningBlockPosition::Footer,
                node: SemanticNode {
                    id: "rb".to_string(),
                    role: "body".to_string(),
                    variant: None,
                    preserve_whitespace: None,
                    list_id: None,
                    depth: None,
                    marker_type: None,
                    content: NodeContent::Text("hello".to_string()),
                    modifiers: vec![],
                    layout: None,
                    ..Default::default()
                },
            }],
        };

        let theme: Theme = serde_json::from_str(
            r##"
{
  "palette": { "black": "#000000" },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "document": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" }
  }
}
"##,
        )
        .unwrap();

        let layout = LayoutResult {
            pages: vec![Page {
                index: 0,
                width: Pt(100_000),
                height: Pt(200_000),
                root: GeometryNode {
                    id: "root_page_0".to_string(),
                    x: Pt::ZERO,
                    y: Pt::ZERO,
                    width: Pt(100_000),
                    height: Pt(200_000),
                    glyphs: vec![],
                    text_runs: vec![],
                    fill_rects: vec![],
                    children: vec![GeometryNode {
                        id: "rb".to_string(),
                        x: Pt(10_000),
                        y: Pt(10_000),
                        width: Pt(50_000),
                        height: Pt(20_000),
                        glyphs: vec![],
                        text_runs: vec![],
                        fill_rects: vec![],
                        children: vec![],
                    }],
                },
            }],
        };

        let plan = build_render_plan(&manifest, &layout, &theme).unwrap();
        let ops = &plan.pages[0].ops;
        assert!(
            ops.iter()
                .any(|op| matches!(op, PaintOp::DrawText { node_id, .. } if node_id == "rb")),
            "expected running block geometry id 'rb' to produce a DrawText op"
        );
    }
}
