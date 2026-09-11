use super::{request_copy_format, request_menu_open};
use crate::copy::CopyFormat;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{declare_class, msg_send, msg_send_id, mutability, sel, ClassType, DeclaredClass};
use objc2_app_kit::{NSApplication, NSEventModifierFlags, NSMenu, NSMenuItem};
use objc2_foundation::{ns_string, MainThreadMarker, NSObject, NSObjectProtocol};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

declare_class!(
    struct MenuTarget;

    unsafe impl ClassType for MenuTarget {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "K2FReaderMenuTarget";
    }

    impl DeclaredClass for MenuTarget {}

    unsafe impl NSObjectProtocol for MenuTarget {}

    unsafe impl MenuTarget {
        #[method(openDocument:)]
        fn open_document(&self, _sender: Option<&AnyObject>) {
            request_menu_open();
        }

        #[method(copyAsMarkdown:)]
        fn copy_as_markdown(&self, _sender: Option<&AnyObject>) {
            request_copy_format(CopyFormat::Markdown);
            sync_copy_format_checks(CopyFormat::Markdown);
        }

        #[method(copyAsText:)]
        fn copy_as_text(&self, _sender: Option<&AnyObject>) {
            request_copy_format(CopyFormat::Plain);
            sync_copy_format_checks(CopyFormat::Plain);
        }
    }
);

thread_local! {
    static TARGET: RefCell<Option<Retained<MenuTarget>>> = const { RefCell::new(None) };
    static COPY_MD_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
    static COPY_TEXT_ITEM: RefCell<Option<Retained<NSMenuItem>>> = const { RefCell::new(None) };
}

static INSTALLED: AtomicBool = AtomicBool::new(false);
/// 0 = Markdown, 1 = Plain — mirrors AppState for menu checkmarks.
static COPY_FORMAT_CODE: AtomicU8 = AtomicU8::new(0);

fn shared_target(mtm: MainThreadMarker) -> Retained<MenuTarget> {
    TARGET.with(|slot| {
        if slot.borrow().is_none() {
            let this = mtm.alloc().set_ivars(());
            let target: Retained<MenuTarget> = unsafe { msg_send_id![super(this), init] };
            *slot.borrow_mut() = Some(target);
        }
        slot.borrow().as_ref().expect("menu target").clone()
    })
}

fn format_code(format: CopyFormat) -> u8 {
    match format {
        CopyFormat::Markdown => 0,
        CopyFormat::Plain => 1,
    }
}

fn set_menu_item_state(item: &NSMenuItem, on: bool) {
    // NSControlStateValueOn/Off without enabling the NSCell feature.
    let state: isize = if on { 1 } else { 0 };
    unsafe {
        let _: () = msg_send![item, setState: state];
    }
}

fn sync_copy_format_checks(format: CopyFormat) {
    COPY_FORMAT_CODE.store(format_code(format), Ordering::SeqCst);
    let md_on = format == CopyFormat::Markdown;
    COPY_MD_ITEM.with(|slot| {
        if let Some(item) = slot.borrow().as_ref() {
            set_menu_item_state(item, md_on);
        }
    });
    COPY_TEXT_ITEM.with(|slot| {
        if let Some(item) = slot.borrow().as_ref() {
            set_menu_item_state(item, !md_on);
        }
    });
}

/// Keep Settings checkmarks aligned with AppState (e.g. after programmatic set).
pub fn sync_copy_format_menu(format: CopyFormat) {
    sync_copy_format_checks(format);
}

/// Insert File → Open… and Settings → copy-format prefs after winit's app menu.
pub fn install_menus() {
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let Some(menubar) = (unsafe { app.mainMenu() }) else {
        return;
    };
    let target = shared_target(mtm);

    let file_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            ns_string!("File"),
            None,
            ns_string!(""),
        )
    };
    let file_menu = NSMenu::new(mtm);
    unsafe { file_menu.setTitle(ns_string!("File")) };
    let open = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            ns_string!("Open…"),
            Some(sel!(openDocument:)),
            ns_string!("o"),
        )
    };
    open.setKeyEquivalentModifierMask(NSEventModifierFlags::NSEventModifierFlagCommand);
    unsafe {
        let _: () = msg_send![&open, setTarget: &*target];
    }
    file_menu.addItem(&open);
    file_item.setSubmenu(Some(&file_menu));
    unsafe { menubar.insertItem_atIndex(&file_item, 1) };

    let settings_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            ns_string!("Settings"),
            None,
            ns_string!(""),
        )
    };
    let settings_menu = NSMenu::new(mtm);
    unsafe { settings_menu.setTitle(ns_string!("Settings")) };

    let copy_md = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            ns_string!("Copy as Markdown"),
            Some(sel!(copyAsMarkdown:)),
            ns_string!(""),
        )
    };
    let copy_text = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            ns_string!("Copy as Text"),
            Some(sel!(copyAsText:)),
            ns_string!(""),
        )
    };
    unsafe {
        let _: () = msg_send![&copy_md, setTarget: &*target];
        let _: () = msg_send![&copy_text, setTarget: &*target];
    }
    settings_menu.addItem(&copy_md);
    settings_menu.addItem(&copy_text);
    settings_item.setSubmenu(Some(&settings_menu));
    unsafe { menubar.insertItem_atIndex(&settings_item, 2) };

    COPY_MD_ITEM.with(|slot| *slot.borrow_mut() = Some(copy_md));
    COPY_TEXT_ITEM.with(|slot| *slot.borrow_mut() = Some(copy_text));
    let initial = match COPY_FORMAT_CODE.load(Ordering::SeqCst) {
        1 => CopyFormat::Plain,
        _ => CopyFormat::Markdown,
    };
    sync_copy_format_checks(initial);
}
