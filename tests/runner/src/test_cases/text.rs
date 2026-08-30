use super::super::node_builders::{container_node, default_page_config, grid_container, text_node};
use k2f_core::{CanvasMode, GridTrack, Manifest};

pub fn text_wrap_width_matrix() -> Manifest {
    // Test text designed to demonstrate wrapping behavior at different widths
    // Narrow width (80pt): Should wrap frequently, many short lines
    // Medium width (180pt): Moderate wrapping, balanced line lengths
    // Wide width (300pt): Minimal wrapping, longer lines
    let text = "The quick brown fox jumps over the lazy dog. This sentence contains multiple words that will wrap differently depending on the available width constraint. When the width is narrow, words will break more frequently. When the width is wider, fewer line breaks occur and text flows more naturally.";

    let grid = grid_container(
        "wrap_grid",
        "box",
        vec![
            GridTrack::Pt { pt: 80000 },  // Narrow: ~80pt
            GridTrack::Pt { pt: 180000 }, // Medium: ~180pt
            GridTrack::Pt { pt: 300000 }, // Wide: ~300pt
        ],
        vec![GridTrack::Pt { pt: 400000 }], // Tall enough for multiple wrapped lines
        12000,                              // 12pt gap between columns
        vec![
            text_node("narrow", text),
            text_node("medium", text),
            text_node("wide", text),
        ],
    );

    Manifest {
        title: "Text: Wrap Width Matrix (via Grid Cells)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn text_whitespace_newlines_matrix() -> Manifest {
    // Test 1: Leading whitespace - should be trimmed at line start
    let t1 = "   Leading spaces should be trimmed: hello world";

    // Test 2: Trailing whitespace - should be trimmed at line end
    let t2 = "Trailing spaces should be trimmed: hello world   ";

    // Test 3: Multiple spaces between words - should be preserved as rendered (not collapsed)
    let t3 =
        "Multiple    spaces    between    words    should    be    preserved    as    rendered";

    // Test 4: Hard newlines and blank lines
    let t4 = "Line 1\nLine 2\n\nLine 4 after blank line\nLine 5";

    // Test 5: Mixed whitespace (spaces, tabs, newlines)
    let t5 = "Mixed whitespace:    Tab here\nNewline here\n\nBlank line above\n  Leading spaces on this line";

    // Test 6: Whitespace at start and end with wrapping
    let t6 = "   This line has leading spaces and will wrap.   Trailing spaces here   ";

    let grid = grid_container(
        "ws_grid",
        "box",
        vec![
            GridTrack::Pt { pt: 200000 }, // Column 1: Wide enough to show wrapping behavior
            GridTrack::Pt { pt: 200000 }, // Column 2
            GridTrack::Pt { pt: 200000 }, // Column 3
        ],
        vec![
            GridTrack::Pt { pt: 120000 }, // Row 1: Leading/trailing spaces, multiple spaces
            GridTrack::Pt { pt: 200000 }, // Row 2: Newlines, blank lines, mixed whitespace
            GridTrack::Pt { pt: 120000 }, // Row 3: Wrapping with whitespace
        ],
        8000,
        vec![
            text_node("leading_ws", t1),
            text_node("trailing_ws", t2),
            text_node("multiple_spaces", t3),
            text_node("newlines_blank", t4),
            text_node("mixed_ws", t5),
            text_node("wrap_ws", t6),
        ],
    );

    Manifest {
        title: "Text: Whitespace + Newlines Matrix".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn text_long_token_multiple_widths() -> Manifest {
    // Long unbreakable token that exceeds various width constraints
    // Tests behavior when a single token is wider than available space
    let long_token = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    // Text with long token in different positions
    let text_before = format!("Short words before long token: {}", long_token);
    let text_after = format!("{} and more text after the long token.", long_token);
    let text_middle = format!("Text before {} and text after.", long_token);
    let text_only = long_token.to_string();

    let grid = grid_container(
        "long_token_grid",
        "box",
        vec![
            GridTrack::Pt { pt: 80000 },  // Narrow: token will overflow
            GridTrack::Pt { pt: 150000 }, // Medium: token may fit or overflow
            GridTrack::Pt { pt: 300000 }, // Wide: token should fit
        ],
        vec![
            GridTrack::Pt { pt: 200000 }, // Row 1: Token before text
            GridTrack::Pt { pt: 200000 }, // Row 2: Token after text
            GridTrack::Pt { pt: 200000 }, // Row 3: Token in middle
            GridTrack::Pt { pt: 150000 }, // Row 4: Token only
        ],
        12000,
        vec![
            text_node("narrow_before", &text_before),
            text_node("medium_before", &text_before),
            text_node("wide_before", &text_before),
            text_node("narrow_after", &text_after),
            text_node("medium_after", &text_after),
            text_node("wide_after", &text_after),
            text_node("narrow_middle", &text_middle),
            text_node("medium_middle", &text_middle),
            text_node("wide_middle", &text_middle),
            text_node("narrow_only", &text_only),
            text_node("medium_only", &text_only),
            text_node("wide_only", &text_only),
        ],
    );

    Manifest {
        title: "Text: Long Unbreakable Token at Multiple Widths".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", vec![grid]),
        running_blocks: vec![],
    }
}

pub fn text_line_height_modifier() -> Manifest {
    use super::super::node_builders::text_node;
    use k2f_core::Modifier;
    let text = "Line 1 normal\nLine 2 tall\nLine 3 tight";
    let mut node = text_node("root", text);
    node.role = "body".to_string();

    let l1_start = 0;
    let l2_start = text.find("Line 2").unwrap();
    let l3_start = text.find("Line 3").unwrap();
    let end = text.len();

    node.modifiers = vec![
        // Increase line height on line 2
        Modifier {
            range: [l2_start, l3_start.saturating_sub(1)],
            mod_type: "emphasis".to_string(),
            intent: "tall".to_string(),
        },
        // Decrease line height on line 3
        Modifier {
            range: [l3_start, end],
            mod_type: "emphasis".to_string(),
            intent: "tight".to_string(),
        },
        // A no-op baseline modifier on line 1 to ensure multiple modifiers coexist.
        Modifier {
            range: [l1_start, l2_start.saturating_sub(1)],
            mod_type: "emphasis".to_string(),
            intent: "black".to_string(),
        },
    ];

    Manifest {
        title: "Text: Line Height Modifier (Theme-Patched)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: node,
        running_blocks: vec![],
    }
}
