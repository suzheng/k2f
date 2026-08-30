use k2f_core::{
    CanvasMode, CodeBlockValue, GridTrack, Manifest, Modifier, NodeContent, SemanticNode,
    StackDirection,
};

use super::super::node_builders::{default_page_config, grid_container, stack_container};

fn code_block_node(
    id: &str,
    value: CodeBlockValue,
    variant: Option<&str>,
    modifiers: Vec<Modifier>,
) -> SemanticNode {
    SemanticNode {
        id: id.to_string(),
        role: "code_block".to_string(),
        variant: variant.map(|v| v.to_string()),
        // Explicitly set true for clarity; validation also accepts None as "treated as true".
        preserve_whitespace: Some(true),
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::CodeBlock(value),
        modifiers,
        layout: None,
        ..Default::default()
    }
}

fn syntax_highlight(range: [usize; 2], intent: &str) -> Modifier {
    Modifier {
        range,
        mod_type: "syntax_highlight".to_string(),
        intent: intent.to_string(),
    }
}

fn unique_range(haystack: &str, needle: &str) -> [usize; 2] {
    let start = haystack
        .find(needle)
        .unwrap_or_else(|| panic!("Expected needle not found in code: '{needle}'"));
    let end = start + needle.len();
    if haystack[start + 1..].contains(needle) {
        panic!("Expected needle to be unique in code: '{needle}'");
    }
    [start, end]
}

pub fn semantic_code_blocks_basic_verbatim() -> Manifest {
    // Contains indentation, trailing spaces, and an empty line. These must be preserved verbatim.
    let code = "fn main() {\n    println!(\"hi\");  \n\n}\n";
    let node = code_block_node(
        "code.basic",
        CodeBlockValue::Text(code.to_string()),
        None,
        vec![],
    );

    Manifest {
        title: "Semantic Code Blocks: Basic Verbatim".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: stack_container(
            "root",
            "document",
            StackDirection::Vertical,
            12000,
            vec![node],
        ),
        running_blocks: vec![],
    }
}

pub fn semantic_code_blocks_lines_vs_string_equivalence() -> Manifest {
    let lines = vec![
        "let x = 1;".to_string(),
        "let y = x + 2;".to_string(),
        "println!(\"{y}\");".to_string(),
    ];
    let as_text = lines.join("\n");

    let left = code_block_node("code.text", CodeBlockValue::Text(as_text), None, vec![]);
    let right = code_block_node("code.lines", CodeBlockValue::Lines(lines), None, vec![]);

    // Put both variants into a 2-column grid so they can be visually compared side-by-side.
    let grid = grid_container(
        "grid",
        "document",
        vec![GridTrack::Pt { pt: 220000 }, GridTrack::Pt { pt: 220000 }],
        vec![GridTrack::Fr { fr: 1 }],
        12000,
        vec![left, right],
    );

    Manifest {
        title: "Semantic Code Blocks: Lines vs String Equivalence".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: grid,
        running_blocks: vec![],
    }
}

pub fn semantic_code_blocks_no_soft_wrap_long_line() -> Manifest {
    // A single long line that should exceed the available width; the code block must not soft-wrap.
    let long = "let extremely_long_identifier_name_that_should_not_soft_wrap_even_in_a_narrow_column = 1234567890; // end";
    let node = code_block_node(
        "code.long",
        CodeBlockValue::Text(long.to_string()),
        None,
        vec![],
    );

    // Narrow fixed column to force overflow without introducing soft wraps.
    let grid = grid_container(
        "grid",
        "document",
        vec![GridTrack::Pt { pt: 180000 }],
        vec![GridTrack::Fr { fr: 1 }],
        0,
        vec![node],
    );

    Manifest {
        title: "Semantic Code Blocks: No Soft Wrap Long Line".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: grid,
        running_blocks: vec![],
    }
}

pub fn semantic_code_blocks_syntax_highlight_ranges() -> Manifest {
    let code = "let x: i32 = 1;\n// comment\nprintln!(\"hi\");\n";

    let mods = vec![
        // keyword: "let"
        syntax_highlight([0, 3], "keyword"),
        // type: "i32"
        syntax_highlight(unique_range(code, "i32"), "type"),
        // comment: "// comment"
        syntax_highlight(unique_range(code, "// comment"), "comment"),
        // string: "\"hi\""
        syntax_highlight(unique_range(code, "\"hi\""), "string"),
    ];

    let node = code_block_node(
        "code.syntax",
        CodeBlockValue::Text(code.to_string()),
        None,
        mods,
    );

    Manifest {
        title: "Semantic Code Blocks: Syntax Highlight Ranges".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: stack_container(
            "root",
            "document",
            StackDirection::Vertical,
            12000,
            vec![node],
        ),
        running_blocks: vec![],
    }
}

pub fn semantic_code_blocks_variant_dark_mode() -> Manifest {
    let code = "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
    let node = code_block_node(
        "code.dark",
        CodeBlockValue::Text(code.to_string()),
        Some("dark_mode"),
        vec![],
    );

    Manifest {
        title: "Semantic Code Blocks: Variant Dark Mode".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: stack_container(
            "root",
            "document",
            StackDirection::Vertical,
            12000,
            vec![node],
        ),
        running_blocks: vec![],
    }
}
