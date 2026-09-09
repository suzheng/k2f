/// Detect SVG bytes that contain a real text-bearing element.
///
/// Comments (`<!-- ... -->`) and CDATA (`<![CDATA[ ... ]]>`) are stripped first
/// so explanatory notes such as `<!-- <text> converted to path -->` do not fail closed.
/// `<textPath>` and `<foreignObject>` are rejected too: they need host fonts and
/// would otherwise drop silently (resvg is built without the `text` feature).

pub const SVG_TEXT_FORBIDDEN_MSG: &str =
    "SVG contains <text> / <tspan> / <textPath> / <foreignObject> — remove those tags (no system fonts). Put labels in a K2F text node beside the image, or convert glyphs to <path>";

pub fn looks_like_svg(bytes: &[u8]) -> bool {
    let s = String::from_utf8_lossy(bytes);
    let trimmed = s.trim_start();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("<svg") || (lower.starts_with("<?xml") && lower.contains("<svg"))
}

pub fn svg_image_path(path: &str) -> bool {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let ext = name.rsplit('.').next().unwrap_or("");
    ext.eq_ignore_ascii_case("svg")
}

pub fn svg_bytes_contain_text_element(bytes: &[u8]) -> bool {
    let lower = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    let stripped = strip_xml_comments_and_cdata(&lower);
    tag_present(&stripped, "<text")
        || tag_present(&stripped, "<tspan")
        || tag_present(&stripped, "<textpath")
        || tag_present(&stripped, "<foreignobject")
}

/// Reject package images that still contain SVG text-bearing elements (compile/pack).
pub fn validate_svg_assets<'a, I, K, V>(assets: I) -> Result<(), String>
where
    I: IntoIterator<Item = (&'a K, &'a V)>,
    K: AsRef<str> + 'a,
    V: AsRef<[u8]> + 'a,
{
    let mut entries: Vec<(&str, &[u8])> = assets
        .into_iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect();
    entries.sort_by_key(|(p, _)| *p);
    for (path, bytes) in entries {
        let normalized = path.replace('\\', "/");
        if !normalized.starts_with("assets/images/") {
            continue;
        }
        if !(svg_image_path(path) || looks_like_svg(bytes)) {
            continue;
        }
        if svg_bytes_contain_text_element(bytes) {
            return Err(format!("IMAGE_SIZE: {path}: {SVG_TEXT_FORBIDDEN_MSG}"));
        }
    }
    Ok(())
}

fn strip_xml_comments_and_cdata(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    loop {
        let comment_at = rest.find("<!--");
        let cdata_at = rest.find("<![cdata[");
        let (at, kind) = match (comment_at, cdata_at) {
            (None, None) => {
                out.push_str(rest);
                break;
            }
            (Some(c), None) => (c, "comment"),
            (None, Some(d)) => (d, "cdata"),
            (Some(c), Some(d)) if c <= d => (c, "comment"),
            (_, Some(d)) => (d, "cdata"),
        };
        out.push_str(&rest[..at]);
        let after_open = if kind == "comment" {
            &rest[at + 4..]
        } else {
            &rest[at + 9..]
        };
        let close = if kind == "comment" { "-->" } else { "]]>" };
        match after_open.find(close) {
            Some(end) => rest = &after_open[end + close.len()..],
            None => break,
        }
    }
    out
}

fn tag_present(haystack: &str, tag: &str) -> bool {
    let mut rest = haystack;
    while let Some(i) = rest.find(tag) {
        let after = rest.get(i + tag.len()..).unwrap_or("");
        if after.starts_with('>')
            || after.starts_with('/')
            || after.starts_with(|c: char| c.is_ascii_whitespace())
        {
            return true;
        }
        rest = after;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_mentioning_text_is_not_an_element() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><!-- <text> converted to path --><rect width="4" height="4" fill="#00f"/></svg>"##;
        assert!(!svg_bytes_contain_text_element(svg));
    }

    #[test]
    fn cdata_mentioning_text_is_not_an_element() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><![CDATA[<text>nope]]><rect width="4" height="4" fill="#00f"/></svg>"##;
        assert!(!svg_bytes_contain_text_element(svg));
    }

    #[test]
    fn real_text_element_is_detected() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><text x="1" y="2">A</text></svg>"#;
        assert!(svg_bytes_contain_text_element(svg));
    }

    #[test]
    fn real_tspan_is_detected() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><tspan>A</tspan></svg>"#;
        assert!(svg_bytes_contain_text_element(svg));
    }

    #[test]
    fn textpath_is_detected() {
        let svg =
            br##"<svg xmlns="http://www.w3.org/2000/svg"><textPath href="#p">A</textPath></svg>"##;
        assert!(svg_bytes_contain_text_element(svg));
    }

    #[test]
    fn foreign_object_is_detected() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg"><foreignObject width="10" height="10">A</foreignObject></svg>"##;
        assert!(svg_bytes_contain_text_element(svg));
    }

    #[test]
    fn comment_mentioning_textpath_is_not_an_element() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg"><!-- <textPath> converted --><path d="M0 0"/></svg>"##;
        assert!(!svg_bytes_contain_text_element(svg));
    }

    #[test]
    fn looks_like_svg_accepts_xml_prolog() {
        let svg = b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
        assert!(looks_like_svg(svg));
        assert!(!looks_like_svg(b"\x89PNG"));
    }

    #[test]
    fn svg_image_path_is_case_insensitive() {
        assert!(svg_image_path("assets/images/mark.SVG"));
        assert!(!svg_image_path("assets/images/mark.png"));
    }

    #[test]
    fn validate_svg_assets_rejects_text_and_allows_comment_only() {
        let mut bad = crate::AssetsMap::new();
        bad.insert(
            "assets/images/fig.svg".into(),
            br#"<svg xmlns="http://www.w3.org/2000/svg"><text>A</text></svg>"#.to_vec(),
        );
        let err = validate_svg_assets(&bad).unwrap_err();
        assert!(err.contains("IMAGE_SIZE"), "{err}");
        assert!(err.contains("<path>"), "{err}");

        let mut ok = crate::AssetsMap::new();
        ok.insert(
            "assets/images/fig.svg".into(),
            br#"<svg xmlns="http://www.w3.org/2000/svg"><!-- <text> converted to path --><path d="M0 0"/></svg>"#.to_vec(),
        );
        validate_svg_assets(&ok).unwrap();

        let mut textpath = crate::AssetsMap::new();
        textpath.insert(
            "assets/images/fig.svg".into(),
            br##"<svg xmlns="http://www.w3.org/2000/svg"><textPath href="#p">A</textPath></svg>"##
                .to_vec(),
        );
        assert!(validate_svg_assets(&textpath).is_err());
    }
}
