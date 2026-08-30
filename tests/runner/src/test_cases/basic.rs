use super::super::node_builders::{
    container_node, default_page_config, image_node, stack_container, text_node,
};
use k2f_core::{CanvasMode, Manifest, Pt, StackDirection};

pub fn one_page_letter() -> Manifest {
    let p1 = "Dear Valued Customer,\n\nWe are writing to inform you about the recent updates to our terms of service. We believe these changes will improve your experience with our platform and provide greater transparency regarding your data privacy.";
    let p2 = "Please take a moment to review the attached document. If you have any questions, do not hesitate to contact our support team, available 24/7 to assist you.";
    let signature = "Sincerely,\nThe K2F Team";

    Manifest {
        title: "1-Page Letter".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node(
            "root",
            "document",
            vec![
                text_node("p1", p1),
                text_node("p2", p2),
                text_node("signature", signature),
            ],
        ),
        running_blocks: vec![],
    }
}

pub fn financial_report() -> Manifest {
    // Simulating a report with nested "rows"
    let mut children = vec![
        text_node("title", "Q3 Financial Report"),
        text_node("subtitle", "Fiscal Year 2025"),
    ];

    for i in 1..=5 {
        let row = container_node(
            &format!("row_{}", i),
            "row",
            vec![
                text_node(&format!("cell_{}_1", i), &format!("Item {}", i)),
                text_node(&format!("cell_{}_2", i), "$1,000.00"),
                text_node(&format!("cell_{}_3", i), "+5.0%"),
            ],
        );
        children.push(row);
    }

    Manifest {
        title: "Financial Report".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", children),
        running_blocks: vec![],
    }
}

pub fn infinite_canvas() -> Manifest {
    // Multiple nodes to demonstrate infinite canvas growth behavior
    // The page height should expand to accommodate all content on a single page
    let mut children = vec![
        text_node("node1", "Start Here"),
        text_node("node2", "Process A"),
        text_node("node3", "Process B"),
        text_node("node4", "Process C"),
        text_node("node5", "Process D"),
        text_node("node6", "End"),
    ];

    // Add some images to increase content height and demonstrate expansion
    children.push(image_node("img1", Pt(200000), Pt(150000)));
    children.push(text_node("node7", "After image 1"));
    children.push(image_node("img2", Pt(180000), Pt(120000)));
    children.push(text_node("node8", "After image 2"));
    children.push(text_node(
        "node9",
        "Final node - canvas should expand to fit all content",
    ));

    Manifest {
        title: "Infinite Canvas".to_string(),
        canvas_mode: CanvasMode::Infinite,
        page_config: default_page_config(), // Initial page size; height will expand
        root: container_node("root", "canvas", children),
        running_blocks: vec![],
    }
}

pub fn typography_stress() -> Manifest {
    let fonts = vec!["Helvetica", "Times New Roman", "Courier"];
    let sizes = vec![10, 12, 14, 18, 24, 36, 48, 72];
    let mut children = vec![];

    for font in &fonts {
        for size in &sizes {
            let text = format!(
                "{} {}pt - The quick brown fox jumps over the lazy dog.",
                font, size
            );
            let mut node = text_node(&format!("{}_{}", font.replace(' ', "_"), size), &text);

            // Apply deterministic style overrides via built-in modifier types.
            // Font families are resolved via theme `font_aliases` (these map to "default" in the test theme).
            use k2f_core::Modifier;
            node.modifiers.push(Modifier {
                range: [0, text.len()],
                mod_type: "emphasis".to_string(),
                intent: font.to_string(),
            });
            node.modifiers.push(Modifier {
                range: [0, text.len()],
                mod_type: "emphasis".to_string(),
                intent: format!("size_{}", size),
            });
            children.push(node);
        }
    }

    Manifest {
        title: "Typography Stress Test".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "document", children),
        running_blocks: vec![],
    }
}

pub fn table_100_rows() -> Manifest {
    let mut rows = vec![text_node("header", "ID | Name | Value | Status")];

    for i in 0..100 {
        let row_text = format!(
            "{:03} | Item Name {:03} | {:.2} | Active",
            i,
            i,
            (i as f64) * 12.34
        );
        rows.push(text_node(&format!("row_{}", i), &row_text));
    }

    Manifest {
        title: "Table 100 Rows".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node("root", "table", rows),
        running_blocks: vec![],
    }
}

pub fn nested_containers(depth: usize) -> Manifest {
    let mut current = text_node("leaf", "Deepest Node");

    for i in 0..depth {
        current = container_node(
            &format!("level_{}", depth - 1 - i),
            "box",
            vec![current], // Wrap previous
        );
    }

    Manifest {
        title: "Nested Containers".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: current,
        running_blocks: vec![],
    }
}

pub fn rtl_ltr_mix() -> Manifest {
    use k2f_core::{NodeContent, SemanticNode};
    Manifest {
        title: "RTL/LTR Mixed".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: SemanticNode {
            id: "root".to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text(
                "LTR + RTL mix: English שלום 123 مرحبا world (end)".to_string(),
            ),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        },
        running_blocks: vec![],
    }
}

pub fn wrapping_stress_long_token() -> Manifest {
    use k2f_core::{NodeContent, SemanticNode};
    Manifest {
        title: "Wrapping Stress (Long Token)".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: SemanticNode {
            id: "root".to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Text(
                "Wrapping stress: short words then a very-long-unbreakable token: AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA and more text after."
                    .to_string(),
            ),
            modifiers: vec![],
            layout: None,
            ..Default::default()
        },
        running_blocks: vec![],
    }
}

pub fn unicode_stress() -> Manifest {
    use k2f_core::{
        Align, FixedSizeHint, JustifyContent, LayoutHint, NodeContent, SemanticNode, StackDirection,
    };
    let combining = SemanticNode {
        id: "combining".to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text(
            "Combining marks: e\u{0301} a\u{0308} o\u{0302} (accent sequences)".to_string(),
        ),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };
    let whitespace = SemanticNode {
        id: "whitespace".to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text(
            "Whitespace + newlines:\nLine1  with   multiple spaces\nLine2".to_string(),
        ),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    Manifest {
        title: "Unicode Stress".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: SemanticNode {
            id: "root".to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Container {
                children: vec![combining, whitespace],
            },
            modifiers: vec![],
            layout: Some(LayoutHint::Stack {
                direction: StackDirection::Vertical,
                gap: 6000, // 6pt
                align_items: Align::default(),
                justify_content: JustifyContent::default(),
                size: FixedSizeHint::default(),
            }),
            ..Default::default()
        },
        running_blocks: vec![],
    }
}

pub fn grid_stack_nesting() -> Manifest {
    use k2f_core::{
        Align, CellAlign, FixedSizeHint, GridTrack, JustifyContent, LayoutHint, NodeContent,
        SemanticNode, StackDirection,
    };
    let cell_1 = SemanticNode {
        id: "cell_1".to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![
                SemanticNode {
                    id: "c1_t1".to_string(),
                    role: "body".to_string(),
                    variant: None,
                    preserve_whitespace: None,
                    list_id: None,
                    depth: None,
                    marker_type: None,
                    content: NodeContent::Text("Cell(1,1) line 1".to_string()),
                    modifiers: vec![],
                    layout: None,
                    ..Default::default()
                },
                SemanticNode {
                    id: "c1_t2".to_string(),
                    role: "body".to_string(),
                    variant: None,
                    preserve_whitespace: None,
                    list_id: None,
                    depth: None,
                    marker_type: None,
                    content: NodeContent::Text("Cell(1,1) line 2".to_string()),
                    modifiers: vec![],
                    layout: None,
                    ..Default::default()
                },
            ],
        },
        modifiers: vec![],
        layout: Some(LayoutHint::Stack {
            direction: StackDirection::Vertical,
            gap: 4000, // 4pt
            align_items: Align::default(),
            justify_content: JustifyContent::default(),
            size: FixedSizeHint::default(),
        }),
        ..Default::default()
    };

    let cell_2 = SemanticNode {
        id: "cell_2".to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![
                SemanticNode {
                    id: "c2_a".to_string(),
                    role: "body".to_string(),
                    variant: None,
                    preserve_whitespace: None,
                    list_id: None,
                    depth: None,
                    marker_type: None,
                    content: NodeContent::Text("A".to_string()),
                    modifiers: vec![],
                    layout: None,
                    ..Default::default()
                },
                SemanticNode {
                    id: "c2_b".to_string(),
                    role: "body".to_string(),
                    variant: None,
                    preserve_whitespace: None,
                    list_id: None,
                    depth: None,
                    marker_type: None,
                    content: NodeContent::Text("B".to_string()),
                    modifiers: vec![],
                    layout: None,
                    ..Default::default()
                },
                SemanticNode {
                    id: "c2_c".to_string(),
                    role: "body".to_string(),
                    variant: None,
                    preserve_whitespace: None,
                    list_id: None,
                    depth: None,
                    marker_type: None,
                    content: NodeContent::Text("C".to_string()),
                    modifiers: vec![],
                    layout: None,
                    ..Default::default()
                },
            ],
        },
        modifiers: vec![],
        layout: Some(LayoutHint::Stack {
            direction: StackDirection::Horizontal,
            gap: 8000, // 8pt
            align_items: Align::default(),
            justify_content: JustifyContent::default(),
            size: FixedSizeHint::default(),
        }),
        ..Default::default()
    };

    let cell_3 = SemanticNode {
        id: "cell_3".to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Container {
            children: vec![SemanticNode {
                id: "c3_inner".to_string(),
                role: "body".to_string(),
                variant: None,
                preserve_whitespace: None,
                list_id: None,
                depth: None,
                marker_type: None,
                content: NodeContent::Text("Nested stack inside grid".to_string()),
                modifiers: vec![],
                layout: None,
                ..Default::default()
            }],
        },
        modifiers: vec![],
        layout: Some(LayoutHint::Stack {
            direction: StackDirection::Vertical,
            gap: 0,
            align_items: Align::default(),
            justify_content: JustifyContent::default(),
            size: FixedSizeHint::default(),
        }),
        ..Default::default()
    };

    let cell_4 = SemanticNode {
        id: "cell_4".to_string(),
        role: "body".to_string(),
        variant: None,
        preserve_whitespace: None,
        list_id: None,
        depth: None,
        marker_type: None,
        content: NodeContent::Text("Bottom-right".to_string()),
        modifiers: vec![],
        layout: None,
        ..Default::default()
    };

    Manifest {
        title: "Grid + Stack Nesting".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: SemanticNode {
            id: "root".to_string(),
            role: "body".to_string(),
            variant: None,
            preserve_whitespace: None,
            list_id: None,
            depth: None,
            marker_type: None,
            content: NodeContent::Container {
                children: vec![cell_1, cell_2, cell_3, cell_4],
            },
            modifiers: vec![],
            layout: Some(LayoutHint::Grid {
                columns: vec![GridTrack::Fr { fr: 1 }, GridTrack::Fr { fr: 2 }],
                rows: vec![GridTrack::Pt { pt: 100000 }, GridTrack::Pt { pt: 120000 }],
                gap: 12000, // 12pt
                row_gap: None,
                column_gap: None,
                cell_align: Some(CellAlign::default()),
                size: Default::default(),
            }),
            ..Default::default()
        },
        running_blocks: vec![],
    }
}

pub fn stack_direction_gap_matrix() -> Manifest {
    let vertical_gap_0 = stack_container(
        "v_gap_0",
        "box",
        StackDirection::Vertical,
        0,
        vec![
            image_node("v0_a", Pt(100000), Pt(40000)),
            image_node("v0_b", Pt(120000), Pt(50000)),
            image_node("v0_c", Pt(140000), Pt(60000)),
        ],
    );
    let vertical_gap_12pt = stack_container(
        "v_gap_12pt",
        "box",
        StackDirection::Vertical,
        12000,
        vec![
            image_node("v12_a", Pt(100000), Pt(40000)),
            image_node("v12_b", Pt(120000), Pt(50000)),
            image_node("v12_c", Pt(140000), Pt(60000)),
        ],
    );
    let horizontal_gap_0 = stack_container(
        "h_gap_0",
        "box",
        StackDirection::Horizontal,
        0,
        vec![
            image_node("h0_a", Pt(80000), Pt(50000)),
            image_node("h0_b", Pt(90000), Pt(60000)),
            image_node("h0_c", Pt(70000), Pt(40000)),
        ],
    );
    let horizontal_gap_8pt = stack_container(
        "h_gap_8pt",
        "box",
        StackDirection::Horizontal,
        8000,
        vec![
            image_node("h8_a", Pt(80000), Pt(50000)),
            image_node("h8_b", Pt(90000), Pt(60000)),
            image_node("h8_c", Pt(70000), Pt(40000)),
        ],
    );

    Manifest {
        title: "Stack: Direction + Gap Matrix".to_string(),
        canvas_mode: CanvasMode::Paged,
        page_config: default_page_config(),
        root: container_node(
            "root",
            "document",
            vec![
                text_node("label_v0", "Vertical stack, gap=0"),
                vertical_gap_0,
                text_node("label_v12", "Vertical stack, gap=12pt"),
                vertical_gap_12pt,
                text_node("label_h0", "Horizontal stack, gap=0"),
                horizontal_gap_0,
                text_node("label_h8", "Horizontal stack, gap=8pt"),
                horizontal_gap_8pt,
            ],
        ),
        running_blocks: vec![],
    }
}
