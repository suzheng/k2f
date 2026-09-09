use crate::AppState;

/// Same steps as `sdk/js/viewer/mount.js`, plus 2.5 / 3.0 to match `AppState` clamp.
const ZOOM_STEPS: [f32; 12] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    PrevPage,
    NextPage,
    ZoomIn,
    ZoomOut,
    Copy,
    Export,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyBind {
    Left,
    Right,
    Plus,
    Minus,
    Char(char),
}

/// Key-repeat pages and zooms; it must not spam clipboard copies or save dialogs.
pub fn accept_key(repeat: bool, action: Action) -> bool {
    !repeat || !matches!(action, Action::Copy | Action::Export | Action::Open)
}

pub fn key_action(bind: KeyBind, ctrl: bool, shift: bool, super_key: bool) -> Option<Action> {
    match bind {
        KeyBind::Left => Some(Action::PrevPage),
        KeyBind::Right => Some(Action::NextPage),
        KeyBind::Plus => Some(Action::ZoomIn),
        KeyBind::Minus => Some(Action::ZoomOut),
        KeyBind::Char(c) if c.eq_ignore_ascii_case(&'s') && shift && (ctrl || super_key) => {
            Some(Action::Export)
        }
        KeyBind::Char(c) if c.eq_ignore_ascii_case(&'c') && (ctrl || super_key) && !shift => {
            Some(Action::Copy)
        }
        KeyBind::Char(c) if c.eq_ignore_ascii_case(&'o') && (ctrl || super_key) && !shift => {
            Some(Action::Open)
        }
        _ => None,
    }
}

pub fn step_zoom(app: &mut AppState, zoom_in: bool) {
    let z = app.zoom();
    let next = if zoom_in {
        ZOOM_STEPS
            .iter()
            .copied()
            .find(|&s| s > z + 0.001)
            .unwrap_or(z)
    } else {
        ZOOM_STEPS
            .iter()
            .copied()
            .rev()
            .find(|&s| s < z - 0.001)
            .unwrap_or(z)
    };
    app.set_zoom(next);
}
