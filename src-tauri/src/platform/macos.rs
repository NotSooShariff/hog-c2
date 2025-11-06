use core_foundation::base::{CFRelease, TCFType};
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::window::{kCGWindowLayer, kCGWindowListOptionOnScreenOnly, CGWindowListCopyWindowInfo};
use std::ptr;

/// Get information about the currently active window on macOS
pub fn get_active_window_info() -> Option<(String, String)> {
    unsafe {
        // Get list of on-screen windows
        let window_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly,
            0, // kCGNullWindowID
        );

        if window_list.is_null() {
            log::error!("Failed to get window list");
            return None;
        }

        let cf_window_list = core_foundation::array::CFArray::wrap_under_create_rule(window_list);

        // Iterate through windows to find the frontmost application window
        for i in 0..cf_window_list.len() {
            let window_info = cf_window_list.get(i);
            if window_info.is_null() {
                continue;
            }

            let dict = core_foundation::dictionary::CFDictionary::wrap_under_get_rule(
                window_info as core_foundation::dictionary::CFDictionaryRef
            );

            // Get window layer - we want layer 0 (normal windows)
            let layer_key = CFString::from_static_string("kCGWindowLayer");
            if let Some(layer) = dict.find(layer_key.as_CFTypeRef() as *const _) {
                let layer_num = core_foundation::number::CFNumber::wrap_under_get_rule(
                    *layer as core_foundation::number::CFNumberRef
                );
                if let Some(layer_val) = layer_num.to_i32() {
                    if layer_val != kCGWindowLayer as i32 {
                        continue;
                    }
                }
            }

            // Get owner name (application name)
            let owner_key = CFString::from_static_string("kCGWindowOwnerName");
            let owner_name = if let Some(owner) = dict.find(owner_key.as_CFTypeRef() as *const _) {
                let cf_string = CFString::wrap_under_get_rule(*owner as CFStringRef);
                cf_string.to_string()
            } else {
                continue;
            };

            // Get window name (title)
            let name_key = CFString::from_static_string("kCGWindowName");
            let window_name = if let Some(name) = dict.find(name_key.as_CFTypeRef() as *const _) {
                let cf_string = CFString::wrap_under_get_rule(*name as CFStringRef);
                cf_string.to_string()
            } else {
                // Some windows don't have a title, use owner name
                owner_name.clone()
            };

            // Return the first valid window (frontmost)
            if !owner_name.is_empty() {
                return Some((owner_name, window_name));
            }
        }

        None
    }
}
