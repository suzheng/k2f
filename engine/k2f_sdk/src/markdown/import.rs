use super::id::IdGen;
use super::inline::{InlineBuf, MAX_MODIFIERS};
use super::{ConversionReport, MarkdownOptions, MarkdownResult};
use crate::builder::MarkdownBuilder;
use crate::error::AgentError;
use crate::nodes::{heading_node, list_item_node, math_node, table_node, text_node};
use k2f_core::{BreakInside, ListMarkerType, Modifier, NodeContent, SemanticNode, TableDataSource};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::fs;
use std::path::Path;

const IMAGE_WIDTH_MM: f64 = 80.0;

struct ListCtx {
    id: String,
    ordered: bool,
    next: usize,
    item: InlineBuf,
    item_open: bool,
}

struct TableBuild {
    header: Vec<MdCell>,
    rows: Vec<Vec<MdCell>>,
    row: Vec<MdCell>,
    cell: InlineBuf,
    in_head: bool,
}

#[derive(Clone, Default)]
struct MdCell {
    text: String,
    modifiers: Vec<Modifier>,
}

pub fn markdown_to_k2f(md: &str, opts: MarkdownOptions) -> Result<MarkdownResult, AgentError> {
    let mut doc = if let Some(bytes) = opts.template_bytes.as_ref() {
        MarkdownBuilder::from_bytes(bytes, &opts.title, opts.page_size)?
    } else {
        MarkdownBuilder::from_dir(&opts.template_dir, &opts.title, opts.page_size)?
    };
    if let Some(bytes) = opts.font_bytes.clone() {
        doc.set_font_bytes(bytes);
    }
    let mut importer = Importer {
        doc,
        ids: IdGen::default(),
        report: ConversionReport::default(),
        opts,
        inline: None,
        lists: Vec::new(),
        table: None,
        code: None,
        quote_depth: 0,
        skip: 0,
        heading_level: 0,
        pending_variant: None,
        pending_keep_with_next: false,
        pending_break_before: false,
        pending_column_span: false,
        pending_role: None,
    };
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_MATH);
    for event in Parser::new_ext(md, options) {
        importer.event(event)?;
    }
    importer.flush_inline()?;
    let report = importer.report;
    let bytes = importer.doc.finish()?;
    Ok(MarkdownResult { bytes, report })
}

struct Importer {
    doc: MarkdownBuilder,
    ids: IdGen,
    report: ConversionReport,
    opts: MarkdownOptions,
    inline: Option<InlineBuf>,
    lists: Vec<ListCtx>,
    table: Option<TableBuild>,
    code: Option<String>,
    quote_depth: u32,
    skip: u32,
    heading_level: u8,
    pending_variant: Option<String>,
    pending_keep_with_next: bool,
    pending_break_before: bool,
    pending_column_span: bool,
    pending_role: Option<String>,
}

impl Importer {
    fn event(&mut self, event: Event<'_>) -> Result<(), AgentError> {
        if self.skip > 0 {
            match event {
                Event::Start(_) => self.skip += 1,
                Event::End(_) => self.skip -= 1,
                _ => {}
            }
            return Ok(());
        }
        match event {
            Event::Start(tag) => self.start(tag)?,
            Event::End(tag) => self.end(tag)?,
            Event::Text(t) => self.text(&t),
            Event::Code(t) => self.code_span(&t),
            Event::SoftBreak => {
                if let Some(b) = self.buf() {
                    b.soft_break();
                }
            }
            Event::HardBreak => {
                if let Some(b) = self.buf() {
                    b.hard_break();
                }
            }
            Event::Html(html) | Event::InlineHtml(html) => self.html(&html)?,
            Event::FootnoteReference(id) => {
                self.report
                    .warn(format!("skipped footnote reference '{id}'"));
            }
            Event::Rule => self.flush_rule()?,
            Event::TaskListMarker(_) => self.report.warn("skipped task list checkbox"),
            Event::InlineMath(s) => {
                if let Some(b) = self.buf() {
                    b.push_math(&s);
                } else {
                    self.report.warn("skipped math");
                }
            }
            Event::DisplayMath(s) => self.flush_display_math(&s)?,
        }
        Ok(())
    }

    fn start(&mut self, tag: Tag<'_>) -> Result<(), AgentError> {
        match tag {
            Tag::Paragraph => {
                if self.in_item() || self.table.is_some() || self.heading_level != 0 {
                    return Ok(());
                }
                self.flush_inline()?;
                self.inline = Some(InlineBuf::default());
            }
            Tag::Heading { level, .. } => {
                self.flush_inline()?;
                self.heading_level = heading_u8(level);
                if self.heading_level > 4 {
                    self.report
                        .warn(format!("heading level {} mapped to h4", self.heading_level));
                    self.heading_level = 4;
                }
                self.inline = Some(InlineBuf::default());
            }
            Tag::BlockQuote(_) => {
                self.flush_inline()?;
                self.quote_depth += 1;
            }
            Tag::CodeBlock(_) => {
                self.flush_inline()?;
                self.code = Some(String::new());
            }
            Tag::List(start) => {
                if self.in_item() {
                    self.flush_item()?;
                } else {
                    self.flush_inline()?;
                }
                self.lists.push(ListCtx {
                    id: self.ids.list(start.is_some()),
                    ordered: start.is_some(),
                    next: 0,
                    item: InlineBuf::default(),
                    item_open: false,
                });
            }
            Tag::Item => {
                if let Some(list) = self.lists.last_mut() {
                    list.item = InlineBuf::default();
                    list.item_open = true;
                }
            }
            Tag::Table(_) => {
                self.flush_inline()?;
                self.table = Some(TableBuild {
                    header: Vec::new(),
                    rows: Vec::new(),
                    row: Vec::new(),
                    cell: InlineBuf::default(),
                    in_head: false,
                });
            }
            Tag::TableHead => {
                if let Some(t) = &mut self.table {
                    t.in_head = true;
                }
            }
            Tag::TableRow => {}
            Tag::TableCell => {
                if let Some(t) = &mut self.table {
                    t.cell = InlineBuf::default();
                }
            }
            Tag::Emphasis => {
                if let Some(b) = self.buf() {
                    b.start_emphasis();
                }
            }
            Tag::Strong => {
                if let Some(b) = self.buf() {
                    b.start_strong();
                }
            }
            Tag::Strikethrough => {
                if let Some(b) = self.buf() {
                    b.start_strikethrough();
                }
            }
            Tag::Link { dest_url, .. } => {
                if let Some(b) = self.buf() {
                    b.start_link(dest_url.into_string());
                }
            }
            Tag::Image { dest_url, .. } => {
                self.flush_inline()?;
                self.add_image(&dest_url)?;
                self.skip = 1;
            }
            Tag::FootnoteDefinition(id) => {
                self.report
                    .warn(format!("skipped footnote definition '{id}'"));
                self.skip = 1;
            }
            Tag::MetadataBlock(_) => {
                self.report.warn("skipped metadata block");
                self.skip = 1;
            }
            _ => {}
        }
        Ok(())
    }

    fn end(&mut self, tag: TagEnd) -> Result<(), AgentError> {
        match tag {
            TagEnd::Paragraph => {
                if self.in_item() {
                    if let Some(list) = self.lists.last_mut() {
                        if !list.item.is_empty() {
                            list.item.push_text(" ");
                        }
                    }
                    return Ok(());
                }
                if self.table.is_some() || self.heading_level != 0 {
                    return Ok(());
                }
                self.flush_inline()?;
            }
            TagEnd::Heading(_) => self.flush_heading()?,
            TagEnd::BlockQuote(_) => {
                self.flush_inline()?;
                self.quote_depth = self.quote_depth.saturating_sub(1);
            }
            TagEnd::CodeBlock => self.flush_code()?,
            TagEnd::List(_) => {
                self.flush_item()?;
                self.lists.pop();
            }
            TagEnd::Item => self.flush_item()?,
            TagEnd::Table => self.flush_table()?,
            TagEnd::TableHead => {
                if let Some(t) = &mut self.table {
                    t.header = std::mem::take(&mut t.row);
                    t.in_head = false;
                }
            }
            TagEnd::TableRow => {
                if let Some(t) = &mut self.table {
                    if t.in_head {
                        t.header = std::mem::take(&mut t.row);
                    } else if !t.row.is_empty() {
                        t.rows.push(std::mem::take(&mut t.row));
                    }
                }
            }
            TagEnd::TableCell => {
                if let Some(t) = &mut self.table {
                    let (text, modifiers) = t.cell.take_trimmed();
                    t.row.push(MdCell { text, modifiers });
                }
            }
            TagEnd::Emphasis => {
                if let Some(b) = self.buf() {
                    b.end_emphasis();
                }
            }
            TagEnd::Strong => {
                if let Some(b) = self.buf() {
                    b.end_strong();
                }
            }
            TagEnd::Strikethrough => {
                if let Some(b) = self.buf() {
                    b.end_strikethrough();
                }
            }
            TagEnd::Link => {
                if let Some(b) = self.buf() {
                    b.end_link();
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn text(&mut self, s: &str) {
        if let Some(code) = &mut self.code {
            code.push_str(s);
            return;
        }
        if let Some(b) = self.buf() {
            b.push_text(s);
        }
    }

    fn code_span(&mut self, s: &str) {
        if let Some(b) = self.buf() {
            b.push_code(s);
        }
    }

    fn html(&mut self, html: &str) -> Result<(), AgentError> {
        let trimmed = html.trim();
        if let Some(body) = k2f_comment(trimmed) {
            return self.apply_comment(body);
        }
        if !trimmed.is_empty() {
            self.report.warn("skipped raw HTML");
        }
        Ok(())
    }

    fn apply_comment(&mut self, body: &str) -> Result<(), AgentError> {
        let body = body.trim();
        if let Some(v) = body.strip_prefix("header=") {
            return self.doc.set_running_header(v);
        }
        if let Some(v) = body.strip_prefix("footer=") {
            return self.doc.set_running_footer(v);
        }
        for part in body.split_whitespace() {
            if let Some(v) = part.strip_prefix("variant=") {
                self.pending_variant = Some(v.to_string());
            } else if part == "keep_with_next=true" {
                self.pending_keep_with_next = true;
            } else if part == "break_before=page" {
                self.pending_break_before = true;
            } else if part == "column_span=all" {
                self.pending_column_span = true;
            } else if let Some(v) = part.strip_prefix("role=") {
                self.pending_role = Some(v.to_string());
            }
        }
        Ok(())
    }

    fn buf(&mut self) -> Option<&mut InlineBuf> {
        if let Some(t) = self.table.as_mut() {
            return Some(&mut t.cell);
        }
        if let Some(list) = self.lists.last_mut() {
            if list.item_open {
                return Some(&mut list.item);
            }
        }
        self.inline.as_mut()
    }

    fn in_item(&self) -> bool {
        self.lists.last().is_some_and(|l| l.item_open)
    }

    fn flush_inline(&mut self) -> Result<(), AgentError> {
        let Some(mut buf) = self.inline.take() else {
            return Ok(());
        };
        let (text, modifiers) = buf.take_trimmed();
        if text.is_empty() {
            return Ok(());
        }
        if self.quote_depth > 0 {
            let id = self.ids.quote();
            self.push_text(id, "quote", &text, modifiers)
        } else {
            let id = self.ids.para();
            self.push_text(id, "body", &text, modifiers)
        }
    }

    fn flush_heading(&mut self) -> Result<(), AgentError> {
        let level = std::mem::take(&mut self.heading_level).max(1);
        let (text, modifiers) = self
            .inline
            .take()
            .map(|mut b| b.take_trimmed())
            .unwrap_or_default();
        if text.is_empty() {
            return Ok(());
        }
        let id = self.ids.heading(level, &text);
        let mut node = heading_node(&id, &text, level);
        node.modifiers = self.cap_mods(modifiers);
        self.apply_hints(&mut node);
        self.doc.append(node)
    }

    fn flush_code(&mut self) -> Result<(), AgentError> {
        let Some(mut body) = self.code.take() else {
            return Ok(());
        };
        if body.ends_with('\n') {
            body.pop();
        }
        let id = self.ids.code();
        self.push_text(id, "code", &body, vec![])
    }

    fn flush_rule(&mut self) -> Result<(), AgentError> {
        self.flush_inline()?;
        let id = self.ids.rule();
        self.push_text(id, "rule", " ", vec![])
    }

    fn flush_display_math(&mut self, tex: &str) -> Result<(), AgentError> {
        self.flush_inline()?;
        let id = self.ids.math();
        self.doc.append(math_node(&id, tex.trim()))
    }

    fn flush_item(&mut self) -> Result<(), AgentError> {
        let depth = self.lists.len().saturating_sub(1) as u32;
        let Some(list) = self.lists.last_mut() else {
            return Ok(());
        };
        if !list.item_open {
            return Ok(());
        }
        let (text, modifiers) = list.item.take_trimmed();
        list.item_open = false;
        if text.is_empty() {
            return Ok(());
        }
        let index = list.next;
        list.next += 1;
        let id = self.ids.list_item(&list.id, index);
        let list_id = list.id.clone();
        let marker = if list.ordered {
            ListMarkerType::Number
        } else {
            ListMarkerType::Bullet
        };
        let mut node = list_item_node(&id, &list_id, &text, depth, marker);
        node.modifiers = self.cap_mods(modifiers);
        self.apply_hints(&mut node);
        self.doc.append(node)
    }

    fn flush_table(&mut self) -> Result<(), AgentError> {
        let Some(table) = self.table.take() else {
            return Ok(());
        };
        let mut header = table.header;
        let mut rows = table.rows;
        if header.is_empty() && !rows.is_empty() {
            header = rows.remove(0);
        }
        if header.is_empty() {
            self.report.warn("skipped empty table");
            return Ok(());
        }
        for row in &mut rows {
            row.resize(header.len(), MdCell::default());
        }
        let header_text: Vec<String> = header.iter().map(|c| c.text.clone()).collect();
        let row_text: Vec<Vec<String>> = rows
            .iter()
            .map(|r| r.iter().map(|c| c.text.clone()).collect())
            .collect();
        let id = self.ids.table();
        let mut node = table_node(&id, &header_text, &row_text);
        patch_table_mods(&mut node, &header, &rows);
        self.apply_hints(&mut node);
        self.doc.append(node)
    }

    fn add_image(&mut self, dest: &str) -> Result<(), AgentError> {
        if dest.is_empty() {
            self.report.warn("skipped image with empty src");
            return Ok(());
        }
        if dest.starts_with("http://") || dest.starts_with("https://") {
            self.report.warn(format!("skipped remote image '{dest}'"));
            return Ok(());
        }
        let path = if Path::new(dest).is_absolute() {
            Path::new(dest).to_path_buf()
        } else {
            self.opts.image_base.join(dest)
        };
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => {
                self.report.warn(format!("skipped missing image '{dest}'"));
                return Ok(());
            }
        };
        let id = self.ids.image();
        if let Err(e) = self.doc.attach_image(&id, &bytes, IMAGE_WIDTH_MM) {
            self.report.warn(format!("skipped image '{dest}': {e}"));
        }
        Ok(())
    }

    fn push_text(
        &mut self,
        id: String,
        role: &str,
        text: &str,
        modifiers: Vec<Modifier>,
    ) -> Result<(), AgentError> {
        let mut node = text_node(&id, role, text);
        node.modifiers = self.cap_mods(modifiers);
        self.apply_hints(&mut node);
        self.doc.append(node)
    }

    fn cap_mods(&mut self, mut modifiers: Vec<Modifier>) -> Vec<Modifier> {
        if modifiers.len() > MAX_MODIFIERS {
            self.report.warn(format!(
                "truncated modifiers from {} to {MAX_MODIFIERS}",
                modifiers.len()
            ));
            modifiers.truncate(MAX_MODIFIERS);
        }
        modifiers
    }

    fn apply_hints(&mut self, node: &mut SemanticNode) {
        if let Some(role) = self.pending_role.take() {
            apply_comment_role(node, &role);
        }
        if let Some(v) = self.pending_variant.take() {
            node.variant = Some(v);
        }
        if self.pending_keep_with_next {
            node.keep_with_next = true;
            self.pending_keep_with_next = false;
        }
        if self.pending_break_before {
            node.break_before = k2f_core::BreakBefore::Page;
            self.pending_break_before = false;
        }
        if self.pending_column_span {
            node.column_span = k2f_core::ColumnSpan::All;
            self.pending_column_span = false;
        }
    }
}

fn heading_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn k2f_comment(html: &str) -> Option<&str> {
    let inner = html
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();
    inner.strip_prefix("k2f:").map(str::trim)
}

fn apply_comment_role(node: &mut SemanticNode, role: &str) {
    match role {
        "warning" => {
            node.role = role.to_string();
            node.break_inside = BreakInside::Avoid;
        }
        "signature_block" | "code" | "quote" | "body" | "h4" | "rule" => {
            node.role = role.to_string();
        }
        _ => {}
    }
}

fn patch_table_mods(node: &mut SemanticNode, header: &[MdCell], rows: &[Vec<MdCell>]) {
    let NodeContent::Table(spec) = &mut node.content else {
        return;
    };
    let TableDataSource::Inline { rows: tree_rows } = &mut spec.data else {
        return;
    };
    if let Some(hr) = tree_rows.first_mut() {
        for (cell, src) in hr.iter_mut().zip(header) {
            cell.modifiers = src.modifiers.clone();
        }
    }
    for (tr, src_row) in tree_rows.iter_mut().skip(1).zip(rows) {
        for (cell, src) in tr.iter_mut().zip(src_row) {
            cell.modifiers = src.modifiers.clone();
        }
    }
}
