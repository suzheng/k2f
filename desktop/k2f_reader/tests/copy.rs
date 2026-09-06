use k2f_paint::TextSpan;
use k2f_reader::copy::{
    plain_text_from_points, plain_text_from_spans, selected_spans, slices_at, CopyPayload, RectPt,
    K2F_NODES_MIME,
};

fn span(id: &str, text: &str, x: f64, y: f64, w: f64, h: f64) -> TextSpan {
    TextSpan {
        node_id: id.into(),
        char_start: 0,
        char_end: text.chars().count(),
        text: text.into(),
        x_pt: x,
        y_pt: y,
        width_pt: w,
        height_pt: h,
    }
}

#[test]
fn reading_order_concat() {
    let spans = vec![
        TextSpan {
            node_id: "a".into(),
            char_start: 0,
            char_end: 2,
            text: "ab".into(),
            x_pt: 10.0,
            y_pt: 10.0,
            width_pt: 20.0,
            height_pt: 12.0,
        },
        TextSpan {
            node_id: "b".into(),
            char_start: 0,
            char_end: 2,
            text: "cd".into(),
            x_pt: 40.0,
            y_pt: 10.0,
            width_pt: 20.0,
            height_pt: 12.0,
        },
    ];
    let sel = RectPt::from_drag(0.0, 0.0, 100.0, 30.0);
    assert_eq!(plain_text_from_spans(&spans, sel), "abcd");
}

#[test]
fn from_drag_is_order_independent() {
    let a = RectPt::from_drag(80.0, 40.0, 10.0, 5.0);
    let b = RectPt::from_drag(10.0, 5.0, 80.0, 40.0);
    assert_eq!(a, b);
    assert_eq!(a.x0, 10.0);
    assert_eq!(a.y0, 5.0);
    assert_eq!(a.x1, 80.0);
    assert_eq!(a.y1, 40.0);
}

#[test]
fn miss_yields_empty() {
    let spans = [span("a", "ab", 10.0, 10.0, 20.0, 12.0)];
    let sel = RectPt::from_drag(100.0, 100.0, 120.0, 120.0);
    assert!(plain_text_from_spans(&spans, sel).is_empty());
    assert!(selected_spans(&spans, sel).is_empty());
}

#[test]
fn sorts_top_then_left() {
    let spans = [
        span("later", "bottom", 10.0, 40.0, 20.0, 10.0),
        span("right", "right", 40.0, 10.0, 20.0, 10.0),
        span("left", "left", 10.0, 10.0, 20.0, 10.0),
    ];
    let sel = RectPt::from_drag(0.0, 0.0, 80.0, 60.0);
    assert_eq!(plain_text_from_spans(&spans, sel), "leftright\nbottom");
}

#[test]
fn touching_edge_selects_span() {
    let spans = [span("a", "hit", 10.0, 10.0, 20.0, 12.0)];
    let sel = RectPt::from_drag(30.0, 16.0, 40.0, 20.0);
    assert_eq!(plain_text_from_spans(&spans, sel), "hit");
}

#[test]
fn non_finite_rect_selects_nothing() {
    let spans = [span("a", "ab", 10.0, 10.0, 20.0, 12.0)];
    let sel = RectPt::from_drag(f64::NAN, 0.0, 100.0, 30.0);
    assert!(plain_text_from_spans(&spans, sel).is_empty());
}

#[test]
fn collapsed_drag_selects_nothing() {
    let spans = [span("a", "hit", 10.0, 10.0, 20.0, 12.0)];
    let sel = RectPt::from_drag(15.0, 15.0, 15.0, 15.0);
    assert!(
        plain_text_from_spans(&spans, sel).is_empty(),
        "zero-area click must match web collapsed selection, got {:?}",
        plain_text_from_spans(&spans, sel)
    );
    assert!(selected_spans(&spans, sel).is_empty());
}

#[test]
fn payload_none_when_miss() {
    let spans = [span("a", "ab", 10.0, 10.0, 20.0, 12.0)];
    let sel = RectPt::from_drag(100.0, 100.0, 120.0, 120.0);
    assert!(CopyPayload::from_spans(&spans, sel).is_none());
}

#[test]
fn payload_omits_geometry_and_matches_web_mime() {
    let spans = [
        span("a", "ab", 10.0, 10.0, 20.0, 12.0),
        span("b", "cd", 40.0, 10.0, 20.0, 12.0),
    ];
    let payload =
        CopyPayload::from_spans(&spans, RectPt::from_drag(0.0, 0.0, 100.0, 30.0)).expect("hits");
    assert_eq!(payload.plain, "abcd");
    assert_eq!(payload.nodes.len(), 2);
    assert_eq!(payload.nodes[0].node_id, "a");
    assert_eq!(payload.nodes[0].char_start, 0);
    assert_eq!(payload.nodes[0].char_end, 2);
    assert_eq!(payload.nodes[0].text, "ab");
    assert_eq!(K2F_NODES_MIME, "application/x-k2f-nodes+json");
    let json = payload.nodes_json();
    assert!(
        json.contains(r#""node_id":"a""#) || json.contains(r#""node_id": "a""#),
        "web MIME body is node objects, got {json}"
    );
    assert!(
        !json.contains("x_pt") && !json.contains("y_pt") && !json.contains("width_pt"),
        "clipboard JSON must omit lock geometry, got {json}"
    );
}

#[test]
fn payload_merges_spans_of_the_same_node() {
    let mut first = span("para", "hello", 10.0, 10.0, 40.0, 12.0);
    first.char_start = 0;
    first.char_end = 5;
    let mut second = span("para", "world", 10.0, 30.0, 40.0, 12.0);
    second.char_start = 6;
    second.char_end = 11;
    let payload =
        CopyPayload::from_spans(&[first, second], RectPt::from_drag(0.0, 0.0, 80.0, 50.0))
            .expect("hits");
    assert_eq!(payload.plain, "hello\nworld");
    assert_eq!(payload.nodes.len(), 1);
    assert_eq!(payload.nodes[0].node_id, "para");
    assert_eq!(payload.nodes[0].char_start, 0);
    assert_eq!(payload.nodes[0].char_end, 11);
    assert_eq!(payload.nodes[0].text, "helloworld");
}

#[test]
fn horizontal_points_select_a_substring() {
    let spans = [span("a", "ABCDEF", 0.0, 0.0, 60.0, 10.0)];
    let slices = slices_at(&spans, 0.0, 5.0, 30.0, 5.0);
    assert_eq!(slices.len(), 1);
    assert_eq!(slices[0].text, "ABC");
    assert_eq!(slices[0].char_start, 0);
    assert_eq!(slices[0].char_end, 3);
    assert_eq!(plain_text_from_points(&spans, 0.0, 5.0, 30.0, 5.0), "ABC");
}

#[test]
fn reverse_horizontal_drag_is_the_same_range() {
    let spans = [span("a", "ABCDEF", 0.0, 0.0, 60.0, 10.0)];
    assert_eq!(
        plain_text_from_points(&spans, 30.0, 5.0, 0.0, 5.0),
        plain_text_from_points(&spans, 0.0, 5.0, 30.0, 5.0)
    );
}

#[test]
fn collapsed_points_select_nothing() {
    let spans = [span("a", "ABCDEF", 0.0, 0.0, 60.0, 10.0)];
    assert!(slices_at(&spans, 15.0, 5.0, 15.0, 5.0).is_empty());
    assert!(plain_text_from_points(&spans, 15.0, 5.0, 15.0, 5.0).is_empty());
    assert!(CopyPayload::from_points(&spans, 15.0, 5.0, 15.0, 5.0).is_none());
}

#[test]
fn caret_range_inserts_newline_between_stacked_lines() {
    let spans = [
        span("left", "left", 10.0, 10.0, 20.0, 10.0),
        span("later", "bottom", 10.0, 40.0, 20.0, 10.0),
    ];
    assert_eq!(
        plain_text_from_points(&spans, 10.0, 15.0, 30.0, 45.0),
        "left\nbottom"
    );
}
