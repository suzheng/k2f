//! macOS File menu and Finder open-document events.
//! Does not replace winit's NSApplicationDelegate (that panics on 0.30).

use std::path::PathBuf;
use std::sync::Mutex;

use winit::event_loop::EventLoopProxy;

mod menu;
mod open_docs;

pub use menu::install_file_menu;

#[derive(Clone, Copy, Debug)]
pub struct Wake;

static PROXY: Mutex<Option<EventLoopProxy<Wake>>> = Mutex::new(None);
static OPEN_PATHS: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
static MENU_OPEN: Mutex<bool> = Mutex::new(false);

pub fn install(proxy: EventLoopProxy<Wake>) {
    *PROXY.lock().expect("open proxy") = Some(proxy);
    open_docs::install();
}

pub fn take_open_paths() -> Vec<PathBuf> {
    std::mem::take(&mut *OPEN_PATHS.lock().expect("open paths"))
}

pub fn take_menu_open() -> bool {
    let mut flag = MENU_OPEN.lock().expect("menu open");
    let was = *flag;
    *flag = false;
    was
}

fn wake() {
    if let Some(proxy) = PROXY.lock().expect("open proxy").as_ref() {
        let _ = proxy.send_event(Wake);
    }
}

fn push_paths(paths: Vec<PathBuf>) {
    if paths.is_empty() {
        return;
    }
    OPEN_PATHS.lock().expect("open paths").extend(paths);
    wake();
}

fn request_menu_open() {
    *MENU_OPEN.lock().expect("menu open") = true;
    wake();
}
