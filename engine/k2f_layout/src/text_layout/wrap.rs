use crate::LayoutContext;
use k2f_core::Pt;

use super::metrics::{line_height_for_style, measure_text_run_width};
use super::types::{merge_join_tracking, push_or_merge_run, TextLine, TextRun};
use super::wrap_fit::fit_prefix_boundary;
use super::wrap_tokenize::{split_run_for_wrapping, FragKind, Fragment};

#[derive(Debug, Clone)]
struct Piece {
    kind: FragKind, // Text or Whitespace
    run: TextRun,
    width: Pt,
    height: Pt,
}

#[derive(Debug, Clone, Copy)]
struct BreakPoint {
    /// How many pieces are kept on the current line (exclusive), *excluding* trailing whitespace.
    line_len_excluding_ws: usize,
    /// The next piece index to start the next line from (exclusive of the whitespace).
    next_start: usize,
}

pub(crate) fn wrap_runs(
    runs: &[TextRun],
    max_width: Pt,
    base_line_height: Pt,
    first_line_indent: Pt,
    ctx: &LayoutContext,
) -> Result<Vec<TextLine>, String> {
    let mut frags: Vec<Fragment> = Vec::new();
    for r in runs {
        frags.extend(split_run_for_wrapping(r));
    }

    let infinite_width = max_width.0 == i128::MAX;

    let mut out_lines: Vec<TextLine> = Vec::new();

    let mut line: Vec<Piece> = Vec::new();
    let mut line_width = Pt::ZERO;
    let mut line_height = base_line_height;
    let mut last_break: Option<BreakPoint> = None;
    let mut is_first_line = true;

    let line_limit = |first: bool| {
        if first && first_line_indent.0 > 0 && !infinite_width {
            (max_width - first_line_indent).max(Pt::ZERO)
        } else {
            max_width
        }
    };

    let mut i: usize = 0;
    while i < frags.len() {
        let frag = frags[i].clone();
        match frag.kind {
            FragKind::Newline => {
                flush_line(
                    &mut out_lines,
                    &mut line,
                    &mut line_width,
                    &mut line_height,
                    base_line_height,
                );
                is_first_line = false;
                last_break = None;
                i += 1;
            }
            FragKind::Atom => {
                let run = frag.run.expect("run present for atom");
                let tex = run.math_tex.as_deref().unwrap_or("");
                let math = crate::math::layout_inline_tex(tex, run.style.font_size, ctx)?;
                let piece_width = math.width;
                let piece_height = if math.height > line_height_for_style(&run.style) {
                    math.height
                } else {
                    line_height_for_style(&run.style)
                };

                if infinite_width
                    || next_line_width(&line, line_width, piece_width, &run)
                        <= line_limit(is_first_line)
                {
                    push_piece(
                        &mut line,
                        &mut line_width,
                        &mut line_height,
                        Piece {
                            kind: frag.kind,
                            run,
                            width: piece_width,
                            height: piece_height,
                        },
                        base_line_height,
                        &mut last_break,
                    );
                    i += 1;
                    continue;
                }

                if !line.is_empty() {
                    if let Some(bp) = last_break {
                        break_at(
                            bp,
                            &mut out_lines,
                            &mut line,
                            &mut line_width,
                            &mut line_height,
                            base_line_height,
                        );
                        is_first_line = false;
                        last_break = None;
                        continue;
                    }
                    flush_line(
                        &mut out_lines,
                        &mut line,
                        &mut line_width,
                        &mut line_height,
                        base_line_height,
                    );
                    is_first_line = false;
                    last_break = None;
                    continue;
                }

                return Err(format!(
                    "UNSPLITTABLE_OVERFLOW: inline math '{tex}' is wider than the line"
                ));
            }
            FragKind::Whitespace | FragKind::Text => {
                let run = frag.run.expect("run present for text/whitespace");

                // Drop leading whitespace (stable, avoids lines starting with spaces).
                if frag.kind == FragKind::Whitespace && line.is_empty() {
                    i += 1;
                    continue;
                }

                let piece_width = measure_text_run_width(&run.text, &run.style, ctx)?;
                let piece_height = line_height_for_style(&run.style);

                // No wrapping when width is infinite; only hard newlines split lines.
                if infinite_width {
                    push_piece(
                        &mut line,
                        &mut line_width,
                        &mut line_height,
                        Piece {
                            kind: frag.kind,
                            run,
                            width: piece_width,
                            height: piece_height,
                        },
                        base_line_height,
                        &mut last_break,
                    );
                    i += 1;
                    continue;
                }

                // If it fits, just add it.
                if next_line_width(&line, line_width, piece_width, &run)
                    <= line_limit(is_first_line)
                {
                    push_piece(
                        &mut line,
                        &mut line_width,
                        &mut line_height,
                        Piece {
                            kind: frag.kind,
                            run,
                            width: piece_width,
                            height: piece_height,
                        },
                        base_line_height,
                        &mut last_break,
                    );
                    i += 1;
                    continue;
                }

                // Overflow handling.
                if !line.is_empty() {
                    if let Some(bp) = last_break {
                        break_at(
                            bp,
                            &mut out_lines,
                            &mut line,
                            &mut line_width,
                            &mut line_height,
                            base_line_height,
                        );
                        is_first_line = false;
                        last_break = None;
                        // retry this fragment on the next line
                        continue;
                    }

                    // No break point: force a new line before this fragment.
                    flush_line(
                        &mut out_lines,
                        &mut line,
                        &mut line_width,
                        &mut line_height,
                        base_line_height,
                    );
                    is_first_line = false;
                    last_break = None;
                    continue;
                }

                // Line is empty and fragment doesn't fit: force split (at least one char).
                if frag.kind == FragKind::Whitespace {
                    // If whitespace can't fit on an empty line, drop it.
                    i += 1;
                    continue;
                }

                let split_at =
                    fit_prefix_boundary(&run.text, &run.style, line_limit(is_first_line), ctx)?;
                let head_text = run.text[..split_at].to_string();
                let tail_text = run.text[split_at..].to_string();

                let head = TextRun {
                    start: run.start,
                    end: run.start + split_at,
                    style: run.style.clone(),
                    text: head_text,
                    math_tex: None,
                };
                let tail = TextRun {
                    start: run.start + split_at,
                    end: run.end,
                    style: run.style,
                    text: tail_text,
                    math_tex: None,
                };

                // Replace current fragment with head, and insert tail (if any) after it.
                frags[i] = Fragment {
                    kind: FragKind::Text,
                    run: Some(head),
                };
                if !tail.text.is_empty() {
                    frags.insert(
                        i + 1,
                        Fragment {
                            kind: FragKind::Text,
                            run: Some(tail),
                        },
                    );
                }
                // retry (head should now be addable)
            }
        }
    }

    // Flush final line (even if empty, to keep layout stable).
    flush_line(
        &mut out_lines,
        &mut line,
        &mut line_width,
        &mut line_height,
        base_line_height,
    );

    Ok(out_lines)
}

fn next_line_width(line: &[Piece], line_width: Pt, piece_width: Pt, run: &TextRun) -> Pt {
    let join = line
        .last()
        .map(|p| merge_join_tracking(&p.run, run))
        .unwrap_or(Pt::ZERO);
    line_width + join + piece_width
}

fn pieces_advance(line: &[Piece]) -> Pt {
    let mut w = Pt::ZERO;
    for (i, p) in line.iter().enumerate() {
        if i > 0 {
            w = w + merge_join_tracking(&line[i - 1].run, &p.run);
        }
        w = w + p.width;
    }
    w
}

fn push_piece(
    line: &mut Vec<Piece>,
    line_width: &mut Pt,
    line_height: &mut Pt,
    piece: Piece,
    base_line_height: Pt,
    last_break: &mut Option<BreakPoint>,
) {
    *line_width = next_line_width(line, *line_width, piece.width, &piece.run);
    if piece.height > *line_height {
        *line_height = piece.height;
    }
    if *line_height < base_line_height {
        *line_height = base_line_height;
    }

    line.push(piece);

    // A whitespace piece creates a deterministic break opportunity *after* it.
    if matches!(line.last().map(|p| p.kind), Some(FragKind::Whitespace)) {
        let len = line.len();
        // Exclude the whitespace itself from the line if we break here.
        *last_break = Some(BreakPoint {
            line_len_excluding_ws: len.saturating_sub(1),
            next_start: len,
        });
    }
}

fn break_at(
    bp: BreakPoint,
    out_lines: &mut Vec<TextLine>,
    line: &mut Vec<Piece>,
    line_width: &mut Pt,
    line_height: &mut Pt,
    base_line_height: Pt,
) {
    // remainder = pieces after whitespace
    let mut remainder = if bp.next_start <= line.len() {
        line.split_off(bp.next_start)
    } else {
        Vec::new()
    };
    // Drop whitespace segment itself by truncating the kept part.
    line.truncate(bp.line_len_excluding_ws);

    flush_line(out_lines, line, line_width, line_height, base_line_height);

    // Start next line with remainder, dropping leading whitespace for stability.
    while matches!(
        remainder.first().map(|p| p.kind),
        Some(FragKind::Whitespace)
    ) {
        remainder.remove(0);
    }

    *line = remainder;
    recompute_line_metrics(line, line_width, line_height, base_line_height);
}

fn flush_line(
    out_lines: &mut Vec<TextLine>,
    line: &mut Vec<Piece>,
    line_width: &mut Pt,
    line_height: &mut Pt,
    base_line_height: Pt,
) {
    // Trim trailing whitespace.
    while matches!(line.last().map(|p| p.kind), Some(FragKind::Whitespace)) {
        line.pop();
    }
    *line_width = pieces_advance(line);

    let mut runs: Vec<TextRun> = Vec::new();
    for p in line.iter() {
        push_or_merge_run(&mut runs, p.run.clone());
    }

    let width = *line_width;
    let height = if runs.is_empty() {
        base_line_height
    } else {
        (*line_height).max(base_line_height)
    };

    out_lines.push(TextLine {
        runs,
        width,
        height,
    });

    line.clear();
    *line_width = Pt::ZERO;
    *line_height = base_line_height;
}

fn recompute_line_metrics(line: &[Piece], width: &mut Pt, height: &mut Pt, base_line_height: Pt) {
    let mut h = base_line_height;
    for p in line {
        if p.height > h {
            h = p.height;
        }
    }
    *width = pieces_advance(line);
    *height = h.max(base_line_height);
}
