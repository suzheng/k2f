use crate::encode::apply_inline_markdown;
use k2f_core::{
    ListMarkerType, NodeContent, RunningBlockNode, RunningBlockPosition, SemanticNode,
    TableDataSource, TableSpec,
};

#[derive(Debug, Clone, Copy)]
pub struct MarkdownEmitOptions {
    /// Emit `<!-- k2f: ... -->` roundtrip hints (CLI export). Clipboard uses `false`.
    pub hints: bool,
}

impl Default for MarkdownEmitOptions {
    fn default() -> Self {
        Self { hints: true }
    }
}

impl MarkdownEmitOptions {
    pub fn clipboard() -> Self {
        Self { hints: false }
    }
}

pub fn document_to_markdown(
    root: &SemanticNode,
    running: &[RunningBlockNode],
    opts: MarkdownEmitOptions,
) -> String {
    let mut out = String::new();
    if opts.hints {
        emit_running(running, &mut out);
    }
    if let NodeContent::Container { children } = &root.content {
        emit_nodes(children, opts, &mut out);
    } else {
        emit_node(root, opts, &mut out);
    }
    out
}

pub fn nodes_to_markdown(nodes: &[SemanticNode], opts: MarkdownEmitOptions) -> String {
    let mut out = String::new();
    emit_nodes(nodes, opts, &mut out);
    out
}

fn emit_running(blocks: &[RunningBlockNode], out: &mut String) {
    for rb in blocks {
        let text = match &rb.node.content {
            NodeContent::Text(t) => t.as_str(),
            _ => continue,
        };
        let key = match rb.position {
            RunningBlockPosition::Header => "header",
            RunningBlockPosition::Footer => "footer",
        };
        comment(out, &format!("{key}={}", sanitize_comment(text)));
    }
}

fn emit_nodes(nodes: &[SemanticNode], opts: MarkdownEmitOptions, out: &mut String) {
    let mut i = 0;
    while i < nodes.len() {
        if nodes[i].role == "list_item" {
            i = emit_list(nodes, i, opts, out);
            blank(out);
            continue;
        }
        emit_node(&nodes[i], opts, out);
        i += 1;
    }
}

fn emit_node(node: &SemanticNode, opts: MarkdownEmitOptions, out: &mut String) {
    if opts.hints {
        hints(node, out);
    }
    match &node.content {
        NodeContent::Container { children } => emit_nodes(children, opts, out),
        NodeContent::Image { src, .. } => {
            push(out, &format!("![]({src})"));
            blank(out);
        }
        NodeContent::Table(spec) => {
            emit_table(spec, out);
            blank(out);
        }
        NodeContent::Text(text) => emit_text_node(node, text, opts, out),
        NodeContent::Math(tex) => {
            push(out, &format!("$$\n{tex}\n$$"));
            blank(out);
        }
        _ => {}
    }
}

fn emit_text_node(node: &SemanticNode, text: &str, opts: MarkdownEmitOptions, out: &mut String) {
    let inline = apply_inline_markdown(text, &node.modifiers);
    match node.role.as_str() {
        "h1" => {
            push(out, &format!("# {inline}"));
            blank(out);
        }
        "h2" => {
            push(out, &format!("## {inline}"));
            blank(out);
        }
        "h3" => {
            push(out, &format!("### {inline}"));
            blank(out);
        }
        "h4" => {
            push(out, &format!("#### {inline}"));
            blank(out);
        }
        "rule" => {
            push(out, "---");
            blank(out);
        }
        "code" => {
            let fence = code_fence(text);
            push(out, &format!("{fence}\n{text}\n{fence}"));
            blank(out);
        }
        "quote" => {
            for line in text.split('\n') {
                push(out, &format!("> {line}"));
            }
            blank(out);
        }
        "signature_block" => {
            if opts.hints {
                comment(out, "role=signature_block");
            }
            push(out, &inline);
            blank(out);
        }
        "warning" => {
            if opts.hints {
                comment(out, "role=warning");
            }
            push(out, &inline);
            blank(out);
        }
        _ => {
            push(out, &inline);
            blank(out);
        }
    }
}

fn emit_list(
    nodes: &[SemanticNode],
    start: usize,
    opts: MarkdownEmitOptions,
    out: &mut String,
) -> usize {
    let mut i = start;
    let mut counters: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    while i < nodes.len() && nodes[i].role == "list_item" {
        let n = &nodes[i];
        let depth = n.depth.unwrap_or(0) as usize;
        let indent = "  ".repeat(depth);
        let text = match &n.content {
            NodeContent::Text(t) => apply_inline_markdown(t, &n.modifiers),
            _ => String::new(),
        };
        let ordered = n.marker_type == Some(ListMarkerType::Number);
        let suffix = if opts.hints {
            list_hint_suffix(n)
        } else {
            String::new()
        };
        if ordered {
            let list_id = n.list_id.clone().unwrap_or_default();
            let c = counters.entry(list_id).or_insert(0);
            *c += 1;
            push(out, &format!("{indent}{c}. {text}{suffix}"));
        } else {
            push(out, &format!("{indent}- {text}{suffix}"));
        }
        i += 1;
    }
    i
}

pub(crate) fn emit_table(spec: &TableSpec, out: &mut String) {
    let TableDataSource::Inline { rows } = &spec.data else {
        return;
    };
    if rows.is_empty() {
        return;
    }
    let header: Vec<String> = rows[0].iter().map(cell_text).collect();
    push(out, &row_md(&header));
    push(out, &format!("|{}|", vec![" --- "; header.len()].join("|")));
    for row in rows.iter().skip(spec.header_rows.max(1)) {
        let cells: Vec<String> = row.iter().map(cell_text).collect();
        push(out, &row_md(&cells));
    }
}

fn cell_text(node: &SemanticNode) -> String {
    match &node.content {
        NodeContent::Text(t) => apply_inline_markdown(t, &node.modifiers).replace('|', "\\|"),
        NodeContent::Math(tex) => format!("${}$", tex.replace('|', "\\|")),
        _ => String::new(),
    }
}

fn row_md(cells: &[String]) -> String {
    format!("| {} |", cells.join(" | "))
}

fn hints(node: &SemanticNode, out: &mut String) {
    if node.keep_with_next && !matches!(node.role.as_str(), "h1" | "h2" | "h3" | "h4") {
        comment(out, "keep_with_next=true");
    }
    if node.column_span.is_all() {
        comment(out, "column_span=all");
    }
    if let Some(k2f_core::LayoutHint::Columns { count, gap }) = &node.layout {
        comment(out, &format!("columns={count} gap={gap}"));
    }
    if let Some(v) = &node.variant {
        if node.role != "table_row_cell" {
            comment(out, &format!("variant={v}"));
        }
    }
}

fn list_hint_suffix(node: &SemanticNode) -> String {
    let mut parts = Vec::new();
    if node.keep_with_next {
        parts.push("keep_with_next=true".to_string());
    }
    if let Some(v) = &node.variant {
        parts.push(format!("variant={v}"));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" <!-- k2f: {} -->", parts.join(" "))
    }
}

fn comment(out: &mut String, body: &str) {
    push(out, &format!("<!-- k2f: {body} -->"));
}

fn sanitize_comment(s: &str) -> String {
    s.replace("--", "-").replace('\n', " ")
}

fn code_fence(text: &str) -> String {
    let mut n = 3;
    let mut run = 0;
    for c in text.chars() {
        if c == '`' {
            run += 1;
            n = n.max(run + 1);
        } else {
            run = 0;
        }
    }
    "`".repeat(n)
}

fn push(out: &mut String, line: &str) {
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(line);
    out.push('\n');
}

fn blank(out: &mut String) {
    if !out.ends_with("\n\n") {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
}
