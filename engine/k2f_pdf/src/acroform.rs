use k2f_core::FormFieldKind;
use k2f_paint::FormFieldLoc;
use pdf_writer::types::{AnnotationFlags, CheckBoxState, FieldFlags, FieldType};
use pdf_writer::{writers, Content, Name, Pdf, Rect, Ref, Str, TextStr};
use std::collections::HashSet;

use crate::coord::pdf_rect;
use crate::ids::Alloc;

const DA: &[u8] = b"/Helv 11 Tf 0 g";

pub struct AcroAllocs {
    pub fields: Vec<Ref>,
    pub helv: Ref,
    pub check_on: Option<Ref>,
    pub check_off: Option<Ref>,
}

pub fn alloc(alloc: &mut Alloc, fields: &[FormFieldLoc]) -> AcroAllocs {
    let ids: Vec<Ref> = fields.iter().map(|_| alloc.bump()).collect();
    let has_check = fields.iter().any(|f| f.kind.is_checkbox());
    AcroAllocs {
        fields: ids,
        helv: alloc.bump(),
        check_on: has_check.then(|| alloc.bump()),
        check_off: has_check.then(|| alloc.bump()),
    }
}

pub fn write_catalog_form(cat: &mut writers::Catalog<'_>, acro: &AcroAllocs) {
    let mut form = cat.form();
    form.fields(acro.fields.iter().copied());
    form.default_appearance(Str(DA));
    form.default_resources()
        .fonts()
        .pair(Name(b"Helv"), acro.helv);
}

pub fn write_widgets(
    pdf: &mut Pdf,
    acro: &AcroAllocs,
    fields: &[FormFieldLoc],
    page_ids: &[Ref],
    page_heights: &[f64],
) -> Vec<Vec<Ref>> {
    pdf.type1_font(acro.helv).base_font(Name(b"Helvetica"));
    if let (Some(on), Some(off)) = (acro.check_on, acro.check_off) {
        write_checkbox_appearances(pdf, on, off);
    }

    let names = unique_pdf_names(fields.iter().map(|f| f.id.as_str()));
    let mut by_page = vec![Vec::new(); page_ids.len()];
    for (i, loc) in fields.iter().enumerate() {
        if loc.page >= page_ids.len() || loc.page >= page_heights.len() {
            continue;
        }
        let id = acro.fields[i];
        write_field(
            pdf,
            id,
            loc,
            &names[i],
            page_ids[loc.page],
            page_heights[loc.page],
            acro.check_on,
            acro.check_off,
        );
        by_page[loc.page].push(id);
    }
    by_page
}

fn unique_pdf_names<'a>(ids: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut used = HashSet::new();
    ids.into_iter()
        .map(|id| {
            let base = {
                let s = id.replace('.', "_");
                if s.is_empty() {
                    "field".into()
                } else {
                    s
                }
            };
            let mut name = base.clone();
            let mut n = 2u32;
            while !used.insert(name.clone()) {
                name = format!("{base}_{n}");
                n += 1;
            }
            name
        })
        .collect()
}

fn write_checkbox_appearances(pdf: &mut Pdf, on_id: Ref, off_id: Ref) {
    let bbox = Rect::new(0.0, 0.0, 1.0, 1.0);
    let mut on = Content::new();
    on.set_stroke_gray(0.0);
    on.set_line_width(0.12);
    on.move_to(0.2, 0.2);
    on.line_to(0.8, 0.8);
    on.move_to(0.2, 0.8);
    on.line_to(0.8, 0.2);
    on.stroke();
    pdf.form_xobject(on_id, &on.finish()).bbox(bbox);
    pdf.form_xobject(off_id, &Content::new().finish()).bbox(bbox);
}

fn write_field(
    pdf: &mut Pdf,
    id: Ref,
    loc: &FormFieldLoc,
    pdf_name: &str,
    page_id: Ref,
    page_h: f64,
    check_on: Option<Ref>,
    check_off: Option<Ref>,
) {
    let mut field = pdf.form_field(id);
    field.partial_name(TextStr(pdf_name));
    field.alternate_name(TextStr(&loc.id));
    match loc.kind {
        FormFieldKind::Text | FormFieldKind::Multiline => {
            field.field_type(FieldType::Text);
            field.text_value(TextStr(&loc.value));
            field.vartext_default_appearance(Str(DA));
            let mut flags = FieldFlags::empty();
            if loc.kind == FormFieldKind::Multiline {
                flags |= FieldFlags::MULTILINE;
            }
            if loc.required {
                flags |= FieldFlags::REQUIRED;
            }
            if !flags.is_empty() {
                field.field_flags(flags);
            }
            if let Some(max) = loc.max_length {
                field.text_max_len(max as i32);
            }
        }
        FormFieldKind::Checkbox => {
            field.field_type(FieldType::Button);
            let state = if loc.value == "true" {
                CheckBoxState::Yes
            } else {
                CheckBoxState::Off
            };
            field.checkbox_value(state);
            field.checkbox_default_value(CheckBoxState::Off);
        }
    }
    let mut annot = field.into_annotation();
    annot.rect(pdf_rect(page_h, loc.x, loc.y, loc.width, loc.height));
    annot.flags(AnnotationFlags::PRINT);
    annot.border(0.0, 0.0, 0.0, None);
    annot.page(page_id);
    if loc.kind.is_checkbox() {
        if let (Some(on), Some(off)) = (check_on, check_off) {
            let as_name = if loc.value == "true" {
                Name(b"Yes")
            } else {
                Name(b"Off")
            };
            annot.appearance_state(as_name);
            annot.appearance().normal().streams().pairs([
                (Name(b"Yes"), on),
                (Name(b"Off"), off),
            ]);
        }
    }
}
