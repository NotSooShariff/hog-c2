use std::ffi::CStr;
use std::ptr;
use x11::xlib;

/// Get information about the currently active window on Linux using X11
pub fn get_active_window_info() -> Option<(String, String)> {
    unsafe {
        // Open X11 display
        let display = xlib::XOpenDisplay(ptr::null());
        if display.is_null() {
            log::error!("Failed to open X11 display");
            return None;
        }

        let root = xlib::XDefaultRootWindow(display);

        // Get the _NET_ACTIVE_WINDOW property
        let active_window_atom = xlib::XInternAtom(
            display,
            b"_NET_ACTIVE_WINDOW\0".as_ptr() as *const i8,
            xlib::False,
        );

        let mut actual_type = 0;
        let mut actual_format = 0;
        let mut nitems = 0;
        let mut bytes_after = 0;
        let mut prop: *mut u8 = ptr::null_mut();

        let status = xlib::XGetWindowProperty(
            display,
            root,
            active_window_atom,
            0,
            1,
            xlib::False,
            xlib::XA_WINDOW,
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut prop,
        );

        if status != 0 || prop.is_null() {
            xlib::XCloseDisplay(display);
            log::warn!("Failed to get active window property");
            return None;
        }

        let active_window = *(prop as *const xlib::Window);
        xlib::XFree(prop as *mut _);

        if active_window == 0 {
            xlib::XCloseDisplay(display);
            return None;
        }

        // Get window title (_NET_WM_NAME or WM_NAME)
        let window_title = get_window_title(display, active_window);

        // Get process name by getting WM_CLASS
        let process_name = get_window_class(display, active_window);

        xlib::XCloseDisplay(display);

        if let (Some(title), Some(proc)) = (window_title, process_name) {
            Some((proc, title))
        } else {
            None
        }
    }
}

unsafe fn get_window_title(display: *mut xlib::Display, window: xlib::Window) -> Option<String> {
    // Try _NET_WM_NAME first (UTF-8)
    let net_wm_name_atom = xlib::XInternAtom(
        display,
        b"_NET_WM_NAME\0".as_ptr() as *const i8,
        xlib::False,
    );

    let utf8_string_atom = xlib::XInternAtom(
        display,
        b"UTF8_STRING\0".as_ptr() as *const i8,
        xlib::False,
    );

    let mut actual_type = 0;
    let mut actual_format = 0;
    let mut nitems = 0;
    let mut bytes_after = 0;
    let mut prop: *mut u8 = ptr::null_mut();

    let status = xlib::XGetWindowProperty(
        display,
        window,
        net_wm_name_atom,
        0,
        1024,
        xlib::False,
        utf8_string_atom,
        &mut actual_type,
        &mut actual_format,
        &mut nitems,
        &mut bytes_after,
        &mut prop,
    );

    if status == 0 && !prop.is_null() {
        let title = CStr::from_ptr(prop as *const i8)
            .to_string_lossy()
            .into_owned();
        xlib::XFree(prop as *mut _);
        return Some(title);
    }

    // Fallback to WM_NAME
    let mut text_prop: xlib::XTextProperty = std::mem::zeroed();
    if xlib::XGetWMName(display, window, &mut text_prop) != 0 && !text_prop.value.is_null() {
        let title = CStr::from_ptr(text_prop.value as *const i8)
            .to_string_lossy()
            .into_owned();
        xlib::XFree(text_prop.value as *mut _);
        return Some(title);
    }

    None
}

unsafe fn get_window_class(display: *mut xlib::Display, window: xlib::Window) -> Option<String> {
    let mut class_hint: xlib::XClassHint = std::mem::zeroed();

    if xlib::XGetClassHint(display, window, &mut class_hint) != 0 {
        let class_name = if !class_hint.res_class.is_null() {
            CStr::from_ptr(class_hint.res_class)
                .to_string_lossy()
                .into_owned()
        } else if !class_hint.res_name.is_null() {
            CStr::from_ptr(class_hint.res_name)
                .to_string_lossy()
                .into_owned()
        } else {
            String::from("Unknown")
        };

        // Free the class hint strings
        if !class_hint.res_name.is_null() {
            xlib::XFree(class_hint.res_name as *mut _);
        }
        if !class_hint.res_class.is_null() {
            xlib::XFree(class_hint.res_class as *mut _);
        }

        return Some(class_name);
    }

    None
}
