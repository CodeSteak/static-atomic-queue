use std::ffi::c_void;

use cocoa::base::id;
use objc::msg_send;

use raw_window_handle::macos::MacOSHandle;
use raw_window_handle::RawWindowHandle;

pub fn get_raw_window_handle(parent: *mut c_void) -> RawWindowHandle {
    let parent_id = parent as id;
    let parent_window = unsafe { msg_send![parent_id, window] };
    RawWindowHandle::MacOS(MacOSHandle {
        ns_window: parent_window,
        ns_view: parent,
        ..MacOSHandle::empty()
    })
}