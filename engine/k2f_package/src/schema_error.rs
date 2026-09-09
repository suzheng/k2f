//! Compact SCHEMA_INVALID messages. Boon's default Display walks every `oneOf`
//! branch (content types, layout kinds, insets), which dumps hundreds of lines.

use boon::{ErrorKind, ValidationError};

const MAX_CHARS: usize = 800;
const MAX_LEAVES: usize = 4;

pub fn format_schema_error(label: &str, err: &ValidationError<'_, '_>) -> String {
    let mut parts = Vec::new();
    collect_leaves(err, &mut parts);
    if parts.is_empty() {
        return truncate(&format!("{label}: {err}"));
    }
    parts.dedup();
    let body = parts
        .into_iter()
        .take(MAX_LEAVES)
        .collect::<Vec<_>>()
        .join("; ");
    truncate(&format!("{label}: {body}"))
}

fn collect_leaves(err: &ValidationError<'_, '_>, out: &mut Vec<String>) {
    match &err.kind {
        ErrorKind::OneOf(_) | ErrorKind::AnyOf | ErrorKind::AllOf => {
            if let Some(best) = err.causes.iter().max_by_key(|c| score_branch(c)) {
                collect_leaves(best, out);
            } else {
                out.push(leaf_msg(err));
            }
        }
        ErrorKind::Group | ErrorKind::Schema { .. } | ErrorKind::Reference { .. } => {
            if err.causes.is_empty() {
                out.push(leaf_msg(err));
            } else {
                for cause in &err.causes {
                    collect_leaves(cause, out);
                }
            }
        }
        _ => {
            if err.causes.is_empty() {
                out.push(leaf_msg(err));
            } else {
                for cause in &err.causes {
                    collect_leaves(cause, out);
                }
            }
        }
    }
}

fn score_branch(err: &ValidationError<'_, '_>) -> i32 {
    let mut score = 0i32;
    walk(err, &mut |leaf| match &leaf.kind {
        ErrorKind::AdditionalProperties { got } => {
            score += 200 + (got.len() as i32) * 10;
        }
        ErrorKind::Required { want } => {
            score += 150 + (want.len() as i32) * 10;
        }
        ErrorKind::Const { .. } | ErrorKind::Enum { .. } => score -= 50,
        ErrorKind::Type { .. } => score -= 20,
        _ => score += 1,
    });
    score
}

fn walk(err: &ValidationError<'_, '_>, visit: &mut impl FnMut(&ValidationError<'_, '_>)) {
    if err.causes.is_empty() {
        visit(err);
        return;
    }
    for cause in &err.causes {
        walk(cause, visit);
    }
}

fn leaf_msg(err: &ValidationError<'_, '_>) -> String {
    let loc = err.instance_location.to_string();
    let loc = if loc.is_empty() { "/".to_string() } else { loc };
    format!("at '{loc}': {}", err.kind)
}

fn truncate(s: &str) -> String {
    if s.len() <= MAX_CHARS {
        return s.to_string();
    }
    let mut end = MAX_CHARS.saturating_sub(1);
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}
