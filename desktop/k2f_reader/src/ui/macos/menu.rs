use super::request_menu_open;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{declare_class, msg_send, msg_send_id, mutability, sel, ClassType, DeclaredClass};
use objc2_app_kit::{NSApplication, NSEventModifierFlags, NSMenu, NSMenuItem};
use objc2_foundation::{ns_string, MainThreadMarker, NSObject, NSObjectProtocol};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

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
    }
);

thread_local! {
    static TARGET: RefCell<Option<Retained<MenuTarget>>> = const { RefCell::new(None) };
}

static INSTALLED: AtomicBool = AtomicBool::new(false);

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

/// Insert File → Open… after winit's default app menu. Safe to call once the window exists.
pub fn install_file_menu() {
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
}
