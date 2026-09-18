use crate::coord::{DOM, NS};
use crate::ir::{TableBox, TextBox};
use crate::para_xml;
use crate::table::table_xml;

const XML_DECL: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;

pub fn story_xml(tb: &TextBox, story_self: &str) -> String {
    let mut hts = 0usize;
    let body = para_xml::write_paras(
        tb.align,
        &tb.runs,
        &mut hts,
        tb.no_break,
        tb.first_line_indent_pt,
        tb.left_indent_pt,
        tb.semantic_newlines,
    );
    wrap_story(story_self, &body)
}

pub fn story_table_xml(tbl: &TableBox, story_self: &str, table_self: &str) -> String {
    wrap_story(story_self, &table_xml(tbl, table_self))
}

fn wrap_story(story_self: &str, body: &str) -> String {
    format!(
        r#"{XML_DECL}
<idPkg:Story xmlns:idPkg="{NS}" DOMVersion="{DOM}">
  <Story Self="{story_self}" AppliedTOCStyle="n" TrackChanges="false" StoryTitle="$ID/" AppliedNamedGrid="n">
    <StoryPreference OpticalMarginAlignment="false" OpticalMarginSize="12" FrameType="TextFrameType" StoryOrientation="Horizontal" StoryDirection="LeftToRightDirection"/>
{body}  </Story>
</idPkg:Story>
"#
    )
}
