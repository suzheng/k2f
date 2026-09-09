use k2f_core::{BreakInside, CanvasMode, Manifest, NodeContent, SemanticNode, StackDirection};

use super::super::node_builders::{default_page_config, stack_container};

fn math_node(id: &str, tex: &str) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "math".to_string(),
        break_inside: BreakInside::Avoid,
        content: NodeContent::Math(tex.to_string()),
        ..Default::default()
    }
}

fn math_manifest(title: &str, tex: &str) -> Manifest {
    Manifest {
        title: title.to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: stack_container(
            "root",
            "document",
            StackDirection::Vertical,
            12000,
            vec![math_node("eq.1", tex)],
        ),
        running_blocks: vec![],
    }
}

pub fn math_display_frac() -> Manifest {
    math_manifest("Math display frac", r"\frac{1}{2}")
}

pub fn math_display_sum() -> Manifest {
    math_manifest("Math display sum", r"\sum_{i=1}^n i")
}

pub fn math_display_left_frac() -> Manifest {
    math_manifest("Math display left frac", r"\left(\frac{1}{2}\right)")
}

pub fn math_display_pmatrix() -> Manifest {
    math_manifest(
        "Math display pmatrix",
        r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
    )
}

pub fn math_unknown_command() -> Manifest {
    math_manifest("Math unknown command", r"\unknown")
}
