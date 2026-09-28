//! Cross-platform wake for the winit loop.
//! macOS menus and the file watcher share one user-event type.

#[derive(Clone, Copy, Debug)]
pub enum Wake {
    Os,
    Disk,
}
