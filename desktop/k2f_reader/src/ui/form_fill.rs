//! Viewer chrome for filling `form_field` nodes. Dirty values stay here until Save relocks.

use k2f_core::{FormFieldKind, CHECKBOX_CHECKED, CHECKBOX_UNCHECKED};
use k2f_paint::FormFieldLoc;
use std::collections::BTreeMap;

/// Lock millipt → document pt (same split as `PageView::window_to_pt`).
pub fn millipt_to_pt(v: i64) -> f64 {
    v as f64 / 1000.0
}

pub fn field_contains_pt(field: &FormFieldLoc, px: f64, py: f64) -> bool {
    let x0 = millipt_to_pt(field.x);
    let y0 = millipt_to_pt(field.y);
    let x1 = millipt_to_pt(field.x.saturating_add(field.width));
    let y1 = millipt_to_pt(field.y.saturating_add(field.height));
    px >= x0 && px < x1 && py >= y0 && py < y1
}

#[derive(Debug, Clone)]
pub struct ActiveEdit {
    pub id: String,
    pub kind: FormFieldKind,
    pub buffer: String,
    pub preedit: String,
    pub max_length: Option<u32>,
}

impl ActiveEdit {
    fn shown(&self) -> String {
        let mut s = self.buffer.clone();
        s.push_str(&self.preedit);
        s
    }

    fn push_str(&mut self, text: &str) {
        let max = self.max_length.map(|n| n as usize);
        for ch in text.chars() {
            if max.is_some_and(|m| self.buffer.chars().count() >= m) {
                break;
            }
            self.buffer.push(ch);
        }
    }

    fn backspace(&mut self) {
        if !self.preedit.is_empty() {
            return;
        }
        self.buffer.pop();
    }
}

#[derive(Debug, Clone, Default)]
pub struct FillState {
    filling: bool,
    dirty: BTreeMap<String, String>,
    active: Option<ActiveEdit>,
}

impl FillState {
    pub fn is_filling(&self) -> bool {
        self.filling
    }

    pub fn is_dirty(&self) -> bool {
        !self.dirty.is_empty() || self.active.is_some()
    }

    pub fn active(&self) -> Option<&ActiveEdit> {
        self.active.as_ref()
    }

    pub fn dirty(&self) -> &BTreeMap<String, String> {
        &self.dirty
    }

    pub fn display_value(&self, field: &FormFieldLoc) -> String {
        if let Some(a) = self.active.as_ref().filter(|a| a.id == field.id) {
            return a.shown();
        }
        self.dirty
            .get(&field.id)
            .cloned()
            .unwrap_or_else(|| field.value.clone())
    }

    pub fn set_filling(&mut self, on: bool) {
        if !on {
            self.commit_active();
        }
        self.filling = on;
        if !on {
            self.active = None;
        }
    }

    pub fn toggle_filling(&mut self) {
        self.set_filling(!self.filling);
    }

    pub fn reset(&mut self) {
        self.filling = false;
        self.dirty.clear();
        self.active = None;
    }

    /// Apply the current buffer into `dirty` without leaving fill mode.
    pub fn commit_active(&mut self) {
        let Some(active) = self.active.take() else {
            return;
        };
        self.dirty.insert(active.id, active.buffer);
    }

    pub fn click(&mut self, field: &FormFieldLoc) {
        if !self.filling {
            return;
        }
        if field.kind.is_checkbox() {
            self.commit_active();
            self.toggle_checkbox(field);
            return;
        }
        if self.active.as_ref().is_some_and(|a| a.id == field.id) {
            return;
        }
        self.commit_active();
        let buffer = self
            .dirty
            .get(&field.id)
            .cloned()
            .unwrap_or_else(|| field.value.clone());
        self.active = Some(ActiveEdit {
            id: field.id.clone(),
            kind: field.kind,
            buffer,
            preedit: String::new(),
            max_length: field.max_length,
        });
    }

    pub fn blur(&mut self) {
        self.commit_active();
    }

    pub fn insert_text(&mut self, text: &str) {
        let Some(active) = self.active.as_mut() else {
            return;
        };
        if active.kind.is_checkbox() {
            return;
        }
        active.push_str(text);
    }

    pub fn backspace(&mut self) {
        if let Some(active) = self.active.as_mut() {
            active.backspace();
        }
    }

    pub fn insert_newline(&mut self) {
        let multiline = self
            .active
            .as_ref()
            .is_some_and(|a| a.kind == FormFieldKind::Multiline);
        if multiline {
            self.insert_text("\n");
        } else {
            self.commit_active();
        }
    }

    pub fn set_preedit(&mut self, preedit: String) {
        if let Some(active) = self.active.as_mut() {
            active.preedit = preedit;
        }
    }

    pub fn commit_ime(&mut self, text: String) {
        if let Some(active) = self.active.as_mut() {
            active.preedit.clear();
            active.push_str(&text);
        }
    }

    pub fn cancel_ime(&mut self) {
        if let Some(active) = self.active.as_mut() {
            active.preedit.clear();
        }
    }

    fn toggle_checkbox(&mut self, field: &FormFieldLoc) {
        let current = self
            .dirty
            .get(&field.id)
            .cloned()
            .unwrap_or_else(|| field.value.clone());
        let next = if current == CHECKBOX_CHECKED {
            CHECKBOX_UNCHECKED.to_string()
        } else {
            CHECKBOX_CHECKED.to_string()
        };
        self.dirty.insert(field.id.clone(), next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(id: &str, kind: FormFieldKind, value: &str) -> FormFieldLoc {
        FormFieldLoc {
            id: id.into(),
            page: 0,
            x: 0,
            y: 0,
            width: 100000,
            height: 20000,
            kind,
            value: value.into(),
            placeholder: None,
            required: false,
            max_length: Some(8),
        }
    }

    #[test]
    fn form_fill_keeps_dirty_until_reset() {
        let mut s = FillState::default();
        let f = field("n", FormFieldKind::Text, "");
        s.set_filling(true);
        s.click(&f);
        s.insert_text("Alice");
        s.commit_active();
        assert_eq!(s.dirty().get("n").map(String::as_str), Some("Alice"));
        s.reset();
        assert!(!s.is_dirty());
    }

    #[test]
    fn form_fill_checkbox_toggles_true_and_empty() {
        let mut s = FillState::default();
        let f = field("c", FormFieldKind::Checkbox, "");
        s.set_filling(true);
        s.click(&f);
        assert_eq!(s.display_value(&f), CHECKBOX_CHECKED);
        s.click(&f);
        assert_eq!(s.display_value(&f), CHECKBOX_UNCHECKED);
    }

    #[test]
    fn form_fill_max_length_stops_insert() {
        let mut s = FillState::default();
        let f = field("n", FormFieldKind::Text, "");
        s.set_filling(true);
        s.click(&f);
        s.insert_text("abcdefghij");
        assert_eq!(s.display_value(&f), "abcdefgh");
    }
}
