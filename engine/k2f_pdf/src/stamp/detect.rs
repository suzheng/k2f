use k2f_core::{Fill, PaintOp};
use k2f_paint::resolve_fill;

pub fn page_needs_stamp(ops: &[PaintOp]) -> bool {
    ops.iter().any(op_needs_stamp)
}

fn op_needs_stamp(op: &PaintOp) -> bool {
    match op {
        PaintOp::BackdropBlur { .. } => true,
        PaintOp::DrawBox { decoration, .. } => {
            if decoration.shadow.is_some() {
                return true;
            }
            match resolve_fill(decoration) {
                Ok(Some(Fill::LinearGradient { .. })) => true,
                Ok(Some(Fill::Solid { .. }) | None) => false,
                Err(_) => false,
            }
        }
        PaintOp::Unknown => false,
        PaintOp::DrawText { .. }
        | PaintOp::DrawImage { .. }
        | PaintOp::DrawTableReference { .. } => false,
    }
}
