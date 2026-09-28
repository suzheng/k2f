//! Edit/Done mode and the surgical node popover. View mode stays the default.

use super::form_fill::TextBuffer;
use k2f_core::Selection;

/// Role, Variant, and Copy node stay in the save and clipboard path.
/// Flip this to show them in the popover again.
pub const SHOW_META_FIELDS: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverField {
    Text,
    Role,
    Variant,
}

#[derive(Debug, Clone)]
pub struct Popover {
    pub id: String,
    pub page: usize,
    text_enabled: bool,
    text: TextBuffer,
    original_text: String,
    role: TextBuffer,
    variant: TextBuffer,
    focus: PopoverField,
}

impl Popover {
    pub fn from_selection(sel: &Selection, page: usize) -> Self {
        let text_enabled = sel.text.is_some();
        let original_text = sel.text.clone().unwrap_or_default();
        Self {
            id: sel.id.clone(),
            page,
            text_enabled,
            text: TextBuffer::new(original_text.clone(), None),
            original_text,
            role: TextBuffer::new(sel.role.clone(), None),
            variant: TextBuffer::new(sel.variant.clone().unwrap_or_default(), None),
            focus: if text_enabled {
                PopoverField::Text
            } else if SHOW_META_FIELDS {
                PopoverField::Role
            } else {
                PopoverField::Text
            },
        }
    }

    /// Text differs from the value captured when the popover opened.
    pub fn text_dirty(&self) -> bool {
        self.text_enabled && self.text.shown() != self.original_text
    }

    pub fn text_enabled(&self) -> bool {
        self.text_enabled
    }

    pub fn focus(&self) -> PopoverField {
        self.focus
    }

    /// Caret in the focused field, as a char index into `shown()`.
    pub fn caret(&self) -> usize {
        self.buf(self.focus).caret()
    }

    pub fn place_caret(&mut self, field: PopoverField, index: usize) {
        if field == PopoverField::Text && !self.text_enabled {
            return;
        }
        self.commit_fields();
        self.set_focus(field);
        self.buf_mut().set_cursor(index);
    }

    pub fn move_caret(&mut self, delta: isize) {
        if !self.accepts_keys() {
            return;
        }
        let buf = self.buf_mut();
        buf.commit_preedit();
        let n = buf.buffer().chars().count() as isize;
        let cur = buf.cursor() as isize;
        buf.set_cursor((cur + delta).clamp(0, n) as usize);
    }

    pub fn set_focus(&mut self, field: PopoverField) {
        if field == PopoverField::Text && !self.text_enabled {
            return;
        }
        self.focus = field;
    }

    pub fn tab(&mut self) {
        if !SHOW_META_FIELDS {
            return;
        }
        self.focus = match self.focus {
            PopoverField::Text => PopoverField::Role,
            PopoverField::Role => PopoverField::Variant,
            PopoverField::Variant if self.text_enabled => PopoverField::Text,
            PopoverField::Variant => PopoverField::Role,
        };
    }

    pub fn shown(&self, field: PopoverField) -> String {
        self.buf(field).shown()
    }

    pub fn role_text(&self) -> &str {
        self.role.buffer()
    }

    pub fn variant_text(&self) -> &str {
        self.variant.buffer()
    }

    /// Committed text, when the node has text. Includes a pending preedit.
    pub fn saved_text(&mut self) -> Option<String> {
        if !self.text_enabled {
            return None;
        }
        self.text.commit_preedit();
        Some(self.text.buffer().to_string())
    }

    pub fn commit_fields(&mut self) {
        self.role.commit_preedit();
        self.variant.commit_preedit();
        self.text.commit_preedit();
    }

    pub fn insert(&mut self, text: &str) {
        if self.accepts_keys() {
            self.buf_mut().push_str(text);
        }
    }

    pub fn backspace(&mut self) {
        if self.accepts_keys() {
            self.buf_mut().backspace();
        }
    }

    pub fn newline(&mut self) {
        if self.focus == PopoverField::Text {
            self.insert("\n");
        }
    }

    pub fn set_preedit(&mut self, preedit: String) {
        if self.accepts_keys() {
            self.buf_mut().set_preedit(preedit);
        }
    }

    pub fn commit_ime(&mut self, text: String) {
        if self.accepts_keys() {
            self.buf_mut().commit_ime(text);
        }
    }

    pub fn cancel_ime(&mut self) {
        self.buf_mut().cancel_ime();
    }

    fn accepts_keys(&self) -> bool {
        match self.focus {
            PopoverField::Text => self.text_enabled,
            PopoverField::Role | PopoverField::Variant => SHOW_META_FIELDS,
        }
    }

    fn buf(&self, field: PopoverField) -> &TextBuffer {
        match field {
            PopoverField::Text => &self.text,
            PopoverField::Role => &self.role,
            PopoverField::Variant => &self.variant,
        }
    }

    fn buf_mut(&mut self) -> &mut TextBuffer {
        match self.focus {
            PopoverField::Text => &mut self.text,
            PopoverField::Role => &mut self.role,
            PopoverField::Variant => &mut self.variant,
        }
    }
}

#[derive(Debug, Default)]
pub struct EditState {
    editing: bool,
    popover: Option<Popover>,
    error: Option<String>,
}

impl EditState {
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn popover(&self) -> Option<&Popover> {
        self.popover.as_ref()
    }

    pub fn popover_mut(&mut self) -> Option<&mut Popover> {
        self.popover.as_mut()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn set_error(&mut self, msg: String) {
        self.error = Some(msg);
    }

    pub fn enter(&mut self) {
        self.editing = true;
        self.error = None;
    }

    pub fn leave(&mut self) {
        self.editing = false;
        self.popover = None;
        self.error = None;
    }

    pub fn clear_popover(&mut self) {
        self.popover = None;
    }

    pub fn show(&mut self, sel: &Selection, page: usize) {
        self.popover = Some(Popover::from_selection(sel, page));
        self.error = None;
    }
}
