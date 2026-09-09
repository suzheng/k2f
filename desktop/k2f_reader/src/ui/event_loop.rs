use super::hud::{ChromeHit, DEFAULT_INNER_H, DEFAULT_INNER_W, MIN_INNER_H, MIN_INNER_W};
use super::input::{accept_key, key_action, Action, KeyBind};
use super::scroll::{line_delta_px, wheel_y_to_scroll};
use super::session::{PointerCursor, Session};
use crate::export::{ensure_extension, pick_save_path, ExportFormat};
use crate::AppState;
use super::pdf_dialog::PdfDialogHit;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{CursorIcon, Window, WindowId};

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
        self.session.set_scale(window.scale_factor() as f32);
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

    fn begin_export(&mut self) {
        if self.session.app().export_format() == ExportFormat::Pdf {
            self.session.open_pdf_dialog();
            self.redraw();
            return;
        }
        self.finish_export(None);
    }

    fn finish_export(&self, pdf_scale: Option<k2f_pdf::PdfScale>) {
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
        let result = if format == ExportFormat::Pdf {
            let scale = pdf_scale.unwrap_or(k2f_pdf::PdfScale::DEFAULT);
            app.export_pdf_bytes_at(scale)
                .and_then(|bytes| std::fs::write(&path, bytes).map_err(anyhow::Error::from))
        } else {
            app.export_to(format, &path)
        };
        match result {
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
        let attrs = Window::default_attributes()
            .with_title(self.session.window_title())
            .with_inner_size(LogicalSize::new(
                DEFAULT_INNER_W as f64,
                DEFAULT_INNER_H as f64,
            ))
            .with_min_inner_size(LogicalSize::new(MIN_INNER_W as f64, MIN_INNER_H as f64));
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
                let chrome_changed = self.session.pointer_move(position.x, position.y);
                if let Some(window) = &self.window {
                    window.set_cursor(match self.session.pointer_cursor(position.x, position.y) {
                        PointerCursor::Pointer => CursorIcon::Pointer,
                        PointerCursor::Text => CursorIcon::Text,
                        PointerCursor::Default => CursorIcon::Default,
                    });
                }
                if chrome_changed {
                    self.redraw();
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => {
                    self.session.pointer_down(self.cursor.0, self.cursor.1);
                    self.redraw();
                }
                ElementState::Released => {
                    if let Some(hit) =
                        self.session.take_pdf_dialog_click(self.cursor.0, self.cursor.1)
                    {
                        match hit {
                            PdfDialogHit::Cancel => {
                                self.session.close_pdf_dialog();
                            }
                            PdfDialogHit::Confirm => {
                                let scale = self.session.selected_pdf_scale();
                                self.session.close_pdf_dialog();
                                self.finish_export(Some(scale));
                            }
                            PdfDialogHit::Scale2 | PdfDialogHit::Scale3 | PdfDialogHit::Scale4 => {}
                        }
                        self.redraw();
                        return;
                    }
                    if let Some(hit) = self.session.take_chrome_click(self.cursor.0, self.cursor.1)
                    {
                        match hit {
                            ChromeHit::Export => {
                                self.session.close_export_menu();
                                self.begin_export();
                            }
                            ChromeHit::ExportMenu => self.session.toggle_export_menu(),
                            ChromeHit::ExportItem(i) => {
                                if let Some(format) = ExportFormat::ALL.get(i).copied() {
                                    self.session.close_export_menu();
                                    self.session.app_mut().set_export_format(format);
                                    self.begin_export();
                                }
                            }
                            ChromeHit::Copy => {
                                self.session.close_export_menu();
                                self.session.app_mut().toggle_copy_format();
                            }
                            ChromeHit::ZoomIn => {
                                self.session.close_export_menu();
                                self.session.apply(Action::ZoomIn);
                            }
                            ChromeHit::ZoomOut => {
                                self.session.close_export_menu();
                                self.session.apply(Action::ZoomOut);
                            }
                        }
                        self.redraw();
                        return;
                    }
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
                if matches!(event.logical_key, Key::Named(NamedKey::Escape)) {
                    if self.session.close_pdf_dialog() || self.session.close_export_menu() {
                        self.redraw();
                    }
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
                    self.session.close_export_menu();
                    self.begin_export();
                    return;
                }
                let copied = self.session.apply(action);
                if let Some(payload) = copied {
                    self.copy_to_clipboard(payload.plain);
                }
                self.redraw();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let wheel_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => line_delta_px(y),
                    MouseScrollDelta::PixelDelta(p) => p.y,
                };
                self.session.scroll_by(wheel_y_to_scroll(wheel_y));
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
