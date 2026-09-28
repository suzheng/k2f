use super::hud::{ChromeHit, DEFAULT_INNER_H, DEFAULT_INNER_W, MIN_INNER_H, MIN_INNER_W};
use super::input::{accept_key, key_action, Action, KeyBind};
use super::pdf_dialog::PdfDialogHit;
use super::reload::{Decision, DiskState, FileIdentity};
use super::scroll::{line_delta_px, wheel_y_to_scroll};
use super::session::{PointerCursor, Session, StagedSave};
use super::wake::Wake;
use super::watch::SourceWatch;
use super::zoom::{zoom_after_ctrl_wheel, zoom_after_pinch};
use crate::export::{
    ensure_extension, pick_folder_path, pick_open_path, pick_save_path, ExportFormat,
};
use crate::AppState;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{CursorIcon, Window, WindowId};

pub fn run(app: Option<AppState>, source: Option<PathBuf>) -> anyhow::Result<()> {
    let event_loop = EventLoop::<Wake>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    #[cfg(target_os = "macos")]
    super::macos::install(proxy.clone());
    let watch = match SourceWatch::start(proxy) {
        Ok(watch) => Some(watch),
        Err(e) => {
            eprintln!("file watch unavailable: {e:#}");
            None
        }
    };
    let session = match app {
        Some(app) => Session::new(app)?,
        None => Session::empty(),
    };
    let mut gui = Gui {
        session,
        source: source.clone(),
        window: None,
        context: None,
        surface: None,
        modifiers: winit::keyboard::ModifiersState::empty(),
        cursor: (0.0, 0.0),
        clipboard: None,
        ime_on: false,
        disk: DiskState::new(),
        watch,
    };
    gui.session.set_source_path(source);
    gui.bind_loaded_file();
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
    ime_on: bool,
    disk: DiskState,
    watch: Option<SourceWatch>,
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
        if self.session.tick_display_lod() {
            window.request_redraw();
        }
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

    fn begin_open(&mut self) {
        let Some(path) = pick_open_path() else {
            return;
        };
        self.load_document(path);
    }

    fn load_document(&mut self, path: PathBuf) {
        match self.session.load_path(&path) {
            Ok(()) => {
                self.session.set_source_path(Some(path.clone()));
                self.source = Some(path);
                self.bind_loaded_file();
                if let Some(window) = &self.window {
                    window.set_title(&self.session.window_title());
                }
            }
            Err(e) => {
                eprintln!("open {}: {e:#}", path.display());
                self.session.set_open_error(format!("{e:#}"));
            }
        }
        self.redraw();
    }

    fn bind_loaded_file(&mut self) {
        let Some(path) = self.source.clone() else {
            self.disk.clear();
            self.session.set_disk_newer(false);
            return;
        };
        if let Some(id) = FileIdentity::from_path(&path) {
            self.disk.note_loaded(id);
        }
        self.session.set_disk_newer(false);
        if let Some(watch) = &mut self.watch {
            if let Err(e) = watch.retarget(&path) {
                eprintln!("watch {}: {e:#}", path.display());
            }
        }
    }

    fn reload_from_disk(&mut self, manual: bool) {
        let Some(path) = self.source.clone() else {
            return;
        };
        let before = FileIdentity::from_path(&path);
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("reload {}: {e:#}", path.display());
                if let Some(id) = before {
                    self.disk.note_failed(id);
                }
                return;
            }
        };
        let after = FileIdentity::from_path(&path);
        if !manual && before != after {
            return;
        }
        if let Err(e) = self.session.reload_bytes(&bytes) {
            eprintln!("reload {}: {e:#}", path.display());
            if let Some(id) = after.or(before) {
                self.disk.note_failed(id);
            }
            return;
        }
        if let Some(id) = after.or(before) {
            self.disk.note_loaded(id);
        }
        self.session.set_disk_newer(false);
        if let Some(window) = &self.window {
            window.set_title(&self.session.window_title());
        }
        self.sync_ime();
        self.redraw();
    }

    fn poll_disk(&mut self) {
        let Some(path) = self.source.clone() else {
            return;
        };
        if self.session.app().is_none() {
            return;
        }
        let current = FileIdentity::from_path(&path);
        match self
            .disk
            .poll(Instant::now(), current, self.session.is_fill_dirty())
        {
            Decision::Idle => {
                if self.session.disk_newer() {
                    self.session.set_disk_newer(false);
                    self.redraw();
                }
            }
            Decision::Waiting => {}
            Decision::Hold => {
                if !self.session.disk_newer() {
                    self.session.set_disk_newer(true);
                    self.redraw();
                }
            }
            Decision::Reload => self.reload_from_disk(false),
        }
    }

    fn arm_wait(&self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let active = self.source.is_some() && self.session.app().is_some();
        let disk = self.disk.wake_at(now, active);
        let lod = self.session.display_wake_at();
        match (lod, disk) {
            (Some(a), Some(b)) => event_loop.set_control_flow(ControlFlow::WaitUntil(a.min(b))),
            (Some(a), None) => event_loop.set_control_flow(ControlFlow::WaitUntil(a)),
            (None, Some(b)) => event_loop.set_control_flow(ControlFlow::WaitUntil(b)),
            (None, None) => event_loop.set_control_flow(ControlFlow::Wait),
        }
    }

    fn begin_export(&mut self) {
        let Some(app) = self.session.app() else {
            return;
        };
        if app.export_format() == ExportFormat::Pdf {
            self.session.open_pdf_dialog();
            self.redraw();
            return;
        }
        self.finish_export(None);
    }

    fn finish_export(&self, pdf_scale: Option<k2f_pdf::PdfScale>) {
        let Some(app) = self.session.app() else {
            return;
        };
        let format = app.export_format();
        let Some(path) = (if format == ExportFormat::Idml {
            pick_folder_path(self.source.as_deref())
        } else {
            pick_save_path(
                format,
                app.title(),
                app.page_count(),
                self.source.as_deref(),
            )
            .map(|p| ensure_extension(p, format))
        }) else {
            return;
        };
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

    fn save_fill(&mut self) {
        match self.session.save_fill() {
            Ok(_) => self.apply_staged_save(),
            Err(e) => {
                eprintln!("save fill: {e:#}");
                self.session.set_open_error(format!("save failed: {e:#}"));
            }
        }
        self.sync_ime();
    }

    fn save_open_node(&mut self) {
        match self.session.save_node() {
            Ok(_) => self.apply_staged_save(),
            Err(e) => {
                eprintln!("save node: {e:#}");
                self.session.set_edit_error(format!("{e:#}"));
            }
        }
        self.sync_ime();
    }

    fn apply_staged_save(&mut self) {
        let Some(staged) = self.session.take_staged_save() else {
            return;
        };
        match staged {
            StagedSave::Written => self.note_saved(),
            StagedSave::NeedsPath(bytes) => self.save_package_as(bytes),
        }
    }

    fn note_saved(&mut self) {
        self.bind_loaded_file();
        if let Some(window) = &self.window {
            window.set_title(&self.session.window_title());
        }
    }

    fn save_package_as(&mut self, bytes: Vec<u8>) {
        let (title, pages) = {
            let Some(app) = self.session.app() else {
                return;
            };
            (app.title().to_string(), app.page_count())
        };
        let Some(path) = pick_save_path(ExportFormat::K2f, &title, pages, self.source.as_deref())
        else {
            return;
        };
        if let Err(e) = super::persist::replace_file(&path, &bytes) {
            eprintln!("save {}: {e:#}", path.display());
            self.fail_save(format!("save failed: {e:#}"));
            return;
        }
        if let Err(e) = self.session.commit_package(&bytes) {
            eprintln!("save {}: {e:#}", path.display());
            self.fail_save(format!("save failed: {e:#}"));
            return;
        }
        self.session.set_source_path(Some(path.clone()));
        self.source = Some(path);
        self.note_saved();
    }

    fn fail_save(&mut self, msg: String) {
        if self.session.popover_open() {
            self.session.set_edit_error(msg);
        } else {
            self.session.set_open_error(msg);
        }
    }

    fn save_shortcut(&mut self) {
        if self.session.popover_open() {
            self.save_open_node();
        } else {
            self.save_fill();
        }
    }

    fn sync_ime(&self) {
        let Some(window) = &self.window else {
            return;
        };
        if let Some((x, y, w, h)) = self.session.ime_cursor_area() {
            window.set_ime_allowed(true);
            window.set_ime_cursor_area(
                LogicalPosition::new(x, y),
                LogicalSize::new(w.max(1.0), h.max(1.0)),
            );
        } else {
            window.set_ime_allowed(false);
        }
    }

    fn drain_os_open(&mut self) {
        #[cfg(target_os = "macos")]
        {
            if super::macos::take_menu_open() {
                self.begin_open();
            }
            if let Some(format) = super::macos::take_copy_format() {
                if let Some(app) = self.session.app_mut() {
                    app.set_copy_format(format);
                }
                super::macos::sync_copy_format_menu(format);
                self.redraw();
            }
            for path in super::macos::take_open_paths() {
                self.load_document(path);
            }
        }
    }
}

impl ApplicationHandler<Wake> for Gui {
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
        #[cfg(target_os = "macos")]
        {
            super::macos::install_menus();
            if let Some(app) = self.session.app() {
                super::macos::sync_copy_format_menu(app.copy_format());
            }
        }
        self.drain_os_open();
        self.redraw();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: Wake) {
        match event {
            Wake::Os => self.drain_os_open(),
            Wake::Disk => self.poll_disk(),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.drain_os_open();
        self.poll_disk();
        if self.session.tick_display_lod() {
            self.redraw();
        }
        self.arm_wait(event_loop);
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
                        PointerCursor::NsResize => CursorIcon::NsResize,
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
                    if let Some(hit) = self
                        .session
                        .take_pdf_dialog_click(self.cursor.0, self.cursor.1)
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
                            ChromeHit::Open => {
                                self.session.close_export_menu();
                                self.begin_open();
                            }
                            ChromeHit::Edit => {
                                self.session.close_export_menu();
                                self.session.toggle_edit();
                                self.sync_ime();
                            }
                            ChromeHit::Save => {
                                self.session.close_export_menu();
                                if self.session.is_fill_dirty() {
                                    self.save_fill();
                                }
                            }
                            ChromeHit::Export => {
                                self.session.close_export_menu();
                                self.begin_export();
                            }
                            ChromeHit::ExportMenu => self.session.toggle_export_menu(),
                            ChromeHit::ExportItem(i) => {
                                if let Some(format) = ExportFormat::ALL.get(i).copied() {
                                    self.session.close_export_menu();
                                    if let Some(app) = self.session.app_mut() {
                                        app.set_export_format(format);
                                    }
                                    self.begin_export();
                                }
                            }
                            ChromeHit::Copy => {
                                self.session.close_export_menu();
                                if let Some(app) = self.session.app() {
                                    if let Ok(md) = app.export_markdown() {
                                        self.copy_to_clipboard(md);
                                    }
                                }
                            }
                            ChromeHit::ZoomIn => {
                                self.session.close_export_menu();
                                self.session.apply(Action::ZoomIn);
                            }
                            ChromeHit::ZoomOut => {
                                self.session.close_export_menu();
                                self.session.apply(Action::ZoomOut);
                            }
                            ChromeHit::Reload => {
                                self.session.close_export_menu();
                                self.reload_from_disk(true);
                            }
                        }
                        self.redraw();
                        return;
                    }
                    if let Some(payload) = self.session.pointer_up(self.cursor.0, self.cursor.1) {
                        self.copy_to_clipboard(payload.plain);
                    } else if let Some(json) = self.session.take_node_clipboard() {
                        self.copy_to_clipboard(json);
                    }
                    self.apply_staged_save();
                    self.sync_ime();
                    self.redraw();
                }
            },
            WindowEvent::Ime(ime) => {
                match ime {
                    Ime::Enabled => self.ime_on = true,
                    Ime::Disabled => {
                        self.ime_on = false;
                        self.session.fill_cancel_ime();
                    }
                    Ime::Preedit(text, _) => self.session.fill_set_preedit(text),
                    Ime::Commit(text) => self.session.fill_commit_ime(text),
                }
                self.sync_ime();
                self.redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !event.state.is_pressed() {
                    return;
                }
                if self.session.discard_prompt_open()
                    && !matches!(event.logical_key, Key::Named(NamedKey::Escape))
                {
                    return;
                }
                if matches!(event.logical_key, Key::Named(NamedKey::Escape)) {
                    if self.session.edit_escape()
                        || self.session.close_pdf_dialog()
                        || self.session.close_export_menu()
                    {
                        self.sync_ime();
                        self.redraw();
                    }
                    return;
                }
                if self.session.popover_open() || self.session.fill_editing() {
                    if matches!(event.logical_key, Key::Named(NamedKey::Tab))
                        && self.session.popover_open()
                    {
                        self.session.edit_tab();
                        self.redraw();
                        return;
                    }
                    let ctrl = self.modifiers.control_key();
                    let super_key = self.modifiers.super_key();
                    if let Some(bind) = bind_key(&event.logical_key) {
                        if let Some(action) =
                            key_action(bind, ctrl, self.modifiers.shift_key(), super_key)
                        {
                            if action == Action::Save && accept_key(event.repeat, Action::Save) {
                                self.save_shortcut();
                                self.redraw();
                                return;
                            }
                            if action == Action::Reload && accept_key(event.repeat, Action::Reload)
                            {
                                self.reload_from_disk(true);
                                return;
                            }
                            if matches!(action, Action::Save | Action::Reload) {
                                return;
                            }
                        }
                    }
                    if !self.ime_on {
                        match &event.logical_key {
                            Key::Named(NamedKey::Backspace) => self.session.fill_backspace(),
                            Key::Named(NamedKey::Enter) => self.session.fill_newline(),
                            Key::Named(NamedKey::ArrowLeft) if self.session.popover_open() => {
                                self.session.edit_arrow(-1);
                            }
                            Key::Named(NamedKey::ArrowRight) if self.session.popover_open() => {
                                self.session.edit_arrow(1);
                            }
                            Key::Named(NamedKey::ArrowUp) if self.session.popover_open() => {
                                self.session.edit_line(-1);
                            }
                            Key::Named(NamedKey::ArrowDown) if self.session.popover_open() => {
                                self.session.edit_line(1);
                            }
                            _ => {
                                if let Some(text) = event.text.as_ref() {
                                    if !text.is_empty()
                                        && !ctrl
                                        && !super_key
                                        && !text.chars().all(|c| c.is_control())
                                    {
                                        self.session.fill_insert(text);
                                    }
                                }
                            }
                        }
                    }
                    self.sync_ime();
                    self.redraw();
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
                if action == Action::Open {
                    #[cfg(target_os = "macos")]
                    {
                        return;
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        self.session.close_export_menu();
                        self.begin_open();
                        return;
                    }
                }
                if action == Action::Export {
                    self.session.close_export_menu();
                    self.begin_export();
                    return;
                }
                if action == Action::Save {
                    self.save_shortcut();
                    self.redraw();
                    return;
                }
                if action == Action::Reload {
                    self.reload_from_disk(true);
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
                if self.modifiers.control_key() {
                    let z = self.session.app().map(|a| a.zoom()).unwrap_or(1.0);
                    let next = zoom_after_ctrl_wheel(z, wheel_y);
                    self.session.set_zoom_about(next, Some(self.cursor.1));
                } else if !self
                    .session
                    .scroll_text_field(self.cursor.0, self.cursor.1, wheel_y)
                {
                    self.session.scroll_by(wheel_y_to_scroll(wheel_y));
                }
                self.redraw();
            }
            WindowEvent::PinchGesture { delta, .. } => {
                let z = self.session.app().map(|a| a.zoom()).unwrap_or(1.0);
                let next = zoom_after_pinch(z, delta);
                self.session.set_zoom_about(next, Some(self.cursor.1));
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
