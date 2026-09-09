//! Attach `application:openURLs:` / `application:openFile:` to winit's delegate class.
//! Do not call `setDelegate` — winit 0.30 panics if the delegate is replaced.

use super::push_paths;
use crate::ui::open_path::path_from_open_string;
use objc2::ffi::{self, BOOL, YES};
use objc2::runtime::{AnyClass, AnyObject, Sel};
use objc2::sel;
use objc2_foundation::{NSArray, NSString, NSURL};
use std::ffi::CStr;
use std::sync::atomic::{AtomicBool, Ordering};

static INSTALLED: AtomicBool = AtomicBool::new(false);

pub fn install() {
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let Some(cls) = AnyClass::get("WinitApplicationDelegate") else {
        return;
    };
    unsafe {
        add_method(
            cls,
            sel!(application:openURLs:),
            application_open_urls as unsafe extern "C" fn(_, _, _, _),
            c"v@:@@",
        );
        add_method(
            cls,
            sel!(application:openFile:),
            application_open_file as unsafe extern "C" fn(_, _, _, _) -> BOOL,
            c"c@:@@",
        );
    }
}

unsafe fn add_method<F>(cls: &AnyClass, sel: Sel, func: F, types: &CStr)
where
    F: Copy,
{
    let imp = std::mem::transmute_copy(&func);
    let _ = ffi::class_addMethod(
        cls as *const AnyClass as *mut ffi::objc_class,
        sel.as_ptr(),
        Some(imp),
        types.as_ptr(),
    );
}

unsafe extern "C" fn application_open_urls(
    _this: *mut AnyObject,
    _cmd: Sel,
    _app: *mut AnyObject,
    urls: *mut AnyObject,
) {
    if urls.is_null() {
        return;
    }
    let urls = unsafe { &*urls.cast::<NSArray<NSURL>>() };
    let mut paths = Vec::new();
    for i in 0..urls.count() {
        let url = unsafe { urls.objectAtIndex(i) };
        if !unsafe { url.isFileURL() } {
            continue;
        }
        let Some(path) = (unsafe { url.path() }) else {
            continue;
        };
        if let Some(p) = path_from_open_string(&path.to_string()) {
            paths.push(p);
        }
    }
    push_paths(paths);
}

unsafe extern "C" fn application_open_file(
    _this: *mut AnyObject,
    _cmd: Sel,
    _app: *mut AnyObject,
    filename: *mut AnyObject,
) -> BOOL {
    if filename.is_null() {
        return YES;
    }
    let name = unsafe { &*filename.cast::<NSString>() };
    if let Some(p) = path_from_open_string(&name.to_string()) {
        push_paths(vec![p]);
    }
    YES
}
