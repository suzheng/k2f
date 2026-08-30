use crate::export::{ensure_extension, pick_save_path};
use super::hud::{copy_format_hit, export_format_hit, export_hit};
use super::input::{accept_key, key_action, Action, KeyBind};
use super::scroll::line_delta_px;
use super::session::Session;
use crate::AppState;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

pub fn run(app: AppState, source: Option<PathBuf>) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut gui = Gui {
        session: Session::new(app)?,
        source,
        window: None,
        context: None,
        surface: None,
        modifiers: winit::keyboard::ModifiersState::empty(),
        cursor: (0.0, 0.0),
        clipboard: None,
    };
    event_loop.run_app(&mut gui)?;
    Ok(())
}

struct Gui {
    session: Session,
    source: Option<PathBuf>,
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    modifiers: winit::keyboard::ModifiersState,
    cursor: (f64, f64),
    clipboard: Option<arboard::Clipboard>,
}

impl Gui {
    fn redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn apply_window_size(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        let (w, h) = self.session.scaled_size();
        let _ = window.request_inner_size(PhysicalSize::new(w, h));
        window.set_title(&self.session.window_title());
        self.redraw();
    }

    fn present(&mut self) {
        let (Some(window), Some(surface)) = (&self.window, &mut self.surface) else {
            return;
        };
        let size = window.inner_size();
        let (Some(nw), Some(nh)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return;
        };
        if surface.resize(nw, nh).is_err() {
            return;
        }
        self.session.set_window_size(size.width, size.height);
        let frame = self.session.compose_frame(size.width, size.height);
        window.pre_present_notify();
        let Ok(mut buf) = surface.buffer_mut() else {
            return;
        };
        let n = buf.len().min(frame.len());
        buf[..n].copy_from_slice(&frame[..n]);
        let _ = buf.present();
    }

    fn copy_to_clipboard(&mut self, plain: String) {
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok();
        }
        if let Some(cb) = &mut self.clipboard {
            let _ = cb.set_text(plain);
        }
    }

    fn window_width(&self) -> u32 {
        self.window
            .as_ref()
            .map(|w| w.inner_size().width)
            .unwrap_or(1)
    }

    fn window_height(&self) -> u32 {
        self.window
            .as_ref()
            .map(|w| w.inner_size().height)
            .unwrap_or(1)
    }

    fn save_export(&self) {
        let app = self.session.app();
        let format = app.export_format();
        let Some(path) = pick_save_path(
            format,
            app.title(),
            app.page_count(),
            self.source.as_deref(),
        ) else {
            return;
        };
        let path = ensure_extension(path, format);
        match app.export_to(format, &path) {
            Ok(()) => eprintln!("wrote {}", path.display()),
            Err(e) => eprintln!("export {:?}: {e:#}", format),
        }
    }
}

impl ApplicationHandler for Gui {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let (w, h) = self.session.scaled_size();
        let attrs = Window::default_attributes()
            .with_title(self.session.window_title())
            .with_inner_size(PhysicalSize::new(w, h));
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        let context = Context::new(window.clone()).expect("softbuffer context");
        let surface = Surface::new(&context, window.clone()).expect("softbuffer surface");
        self.context = Some(context);
        self.surface = Some(surface);
        self.window = Some(window);
        self.redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(|w| w.id()) != Some(window_id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) | WindowEvent::RedrawRequested => self.present(),
            WindowEvent::ModifiersChanged(m) => self.modifiers = m.state(),
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                self.session.pointer_move(position.x, position.y);
                if self.session.is_dragging() {
                    self.redraw();
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => {
                    let w = self.window_width();
                    let h = self.window_height();
                    if export_hit(w, h, self.cursor.0, self.cursor.1) {
                        self.save_export();
                        return;
                    }
                    let copy_label = self.session.app().copy_format().hud_label();
                    let format_label = self.session.app().export_format().hud_label();
                    if export_format_hit(w, h, format_label, self.cursor.0, self.cursor.1) {
                        self.session.app_mut().toggle_export_format();
                        self.redraw();
                        return;
                    }
                    if copy_format_hit(w, h, copy_label, format_label, self.cursor.0, self.cursor.1) {
                        self.session.app_mut().toggle_copy_format();
                        self.redraw();
                        return;
                    }
                    self.session.pointer_down(self.cursor.0, self.cursor.1);
                    self.redraw();
                }
                ElementState::Released => {
                    if let Some(payload) = self.session.pointer_up(self.cursor.0, self.cursor.1) {
                        self.copy_to_clipboard(payload.plain);
                    }
                    self.redraw();
                }
            },
            WindowEvent::KeyboardInput { event, .. } => {
                if !event.state.is_pressed() {
                    return;
                }
                let Some(bind) = bind_key(&event.logical_key) else {
                    return;
                };
                let Some(action) = key_action(
                    bind,
                    self.modifiers.control_key(),
                    self.modifiers.shift_key(),
                    self.modifiers.super_key(),
                ) else {
                    return;
                };
                if !accept_key(event.repeat, action) {
                    return;
                }
                if action == Action::Export {
                    self.save_export();
                    return;
                }
                let copied = self.session.apply(action);
                if let Some(payload) = copied {
                    self.copy_to_clipboard(payload.plain);
                }
                if matches!(action, Action::ZoomIn | Action::ZoomOut) {
                    self.apply_window_size();
                }
                self.redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => line_delta_px(y),
                    MouseScrollDelta::PixelDelta(p) => p.y,
                };
                self.session.scroll_by(dy);
                self.redraw();
            }
            _ => {}
        }
    }
}

fn bind_key(key: &Key) -> Option<KeyBind> {
    match key {
        Key::Named(NamedKey::ArrowLeft) => Some(KeyBind::Left),
        Key::Named(NamedKey::ArrowRight) => Some(KeyBind::Right),
        Key::Character(c) => {
            let s = c.as_str();
            if s == "+" || s == "=" {
                Some(KeyBind::Plus)
            } else if s == "-" {
                Some(KeyBind::Minus)
            } else {
                let mut chars = s.chars();
                let ch = chars.next()?;
                if chars.next().is_some() {
                    None
                } else {
                    Some(KeyBind::Char(ch))
                }
            }
        }
        _ => None,
    }
}
