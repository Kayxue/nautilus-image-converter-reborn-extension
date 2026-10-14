//! Utilities to detect the parent Nautilus GTK4 window, obtain its
//! window handle (X11 XID or Wayland XDG Foreign exported token), or display
//! GTK dialogs within the Nautilus process.

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_void, CStr};

/// Attempts to retrieve the active `GtkWindow` pointer of the host Nautilus application.
///
/// # Safety
///
/// Must be called from the main GTK thread within the Nautilus host process.
pub unsafe fn get_active_nautilus_window() -> Option<*mut c_void> {
    let app = gio::ffi::g_application_get_default();
    if app.is_null() {
        return None;
    }

    let get_active_window: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
        lookup_symbol(b"gtk_application_get_active_window\0")?;
    let mut window = get_active_window(app as *mut c_void);

    if window.is_null() {
        let get_windows: unsafe extern "C" fn(*mut c_void) -> *mut glib::ffi::GList =
            lookup_symbol(b"gtk_application_get_windows\0")?;
        let list = get_windows(app as *mut c_void);
        if list.is_null() || (*list).data.is_null() {
            return None;
        }
        window = (*list).data;
    }

    if window.is_null() {
        None
    } else {
        Some(window)
    }
}

/// Attempts to retrieve the active `GdkSurface` pointer of the host Nautilus window.
///
/// # Safety
///
/// Must be called from the main GTK thread within the Nautilus host process.
pub unsafe fn get_active_nautilus_surface() -> Option<*mut c_void> {
    let window = get_active_nautilus_window()?;
    let get_surface: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
        lookup_symbol(b"gtk_native_get_surface\0")?;
    let surface = get_surface(window);
    if surface.is_null() {
        None
    } else {
        Some(surface)
    }
}

/// Displays a GTK dialog notifying the user that `nautilus-image-converter` is not installed.
///
/// # Safety
///
/// Must be called from the main GTK thread within the Nautilus host process.
pub unsafe fn show_missing_app_dialog(parent_window: Option<*mut c_void>) {
    let parent = parent_window.unwrap_or(std::ptr::null_mut());

    let title = b"Image Converter Not Installed\0";
    let message = b"The companion application 'nautilus-image-converter' could not be found.\n\nPlease install 'nautilus-image-converter-reborn-app' to resize, rotate, and convert images.\0";

    // 1. Try modern GTK4 GtkAlertDialog (GTK 4.10+)
    type AlertDialogNewFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_void;
    type AlertDialogSetDetailFn = unsafe extern "C" fn(*mut c_void, *const c_char);
    type AlertDialogShowFn = unsafe extern "C" fn(*mut c_void, *mut c_void);

    if let (Some(new_fn), Some(show_fn)) = (
        lookup_symbol::<AlertDialogNewFn>(b"gtk_alert_dialog_new\0"),
        lookup_symbol::<AlertDialogShowFn>(b"gtk_alert_dialog_show\0"),
    ) {
        let fmt = b"%s\0";
        let dialog = new_fn(fmt.as_ptr() as *const c_char, title.as_ptr() as *const c_char);
        if !dialog.is_null() {
            if let Some(set_detail_fn) =
                lookup_symbol::<AlertDialogSetDetailFn>(b"gtk_alert_dialog_set_detail\0")
            {
                set_detail_fn(dialog, message.as_ptr() as *const c_char);
            }
            show_fn(dialog, parent);
            glib::gobject_ffi::g_object_unref(dialog as *mut glib::gobject_ffi::GObject);
            return;
        }
    }

    // 2. Fallback: GtkMessageDialog for earlier GTK4
    type MessageDialogNewFn = unsafe extern "C" fn(
        *mut c_void,
        u32, // GtkDialogFlags (MODAL = 1)
        u32, // GtkMessageType (WARNING = 2)
        u32, // GtkButtonsType (CLOSE = 2)
        *const c_char,
        *const c_char,
    ) -> *mut c_void;
    type WindowPresentFn = unsafe extern "C" fn(*mut c_void);

    if let (Some(new_msg_fn), Some(present_fn)) = (
        lookup_symbol::<MessageDialogNewFn>(b"gtk_message_dialog_new\0"),
        lookup_symbol::<WindowPresentFn>(b"gtk_window_present\0"),
    ) {
        let fmt = b"%s\0";
        let dialog = new_msg_fn(
            parent,
            1, // GTK_DIALOG_MODAL
            2, // GTK_MESSAGE_WARNING
            2, // GTK_BUTTONS_CLOSE
            fmt.as_ptr() as *const c_char,
            message.as_ptr() as *const c_char,
        );
        if !dialog.is_null() {
            present_fn(dialog);
            return;
        }
    }

    eprintln!(
        "[nautilus-image-converter-reborn] nautilus-image-converter is not installed. \
         Please install 'nautilus-image-converter-reborn-app'."
    );
}

/// Checks if `surface` is an X11 surface and retrieves its XID.
///
/// # Safety
///
/// `surface` must be a valid pointer to a live `GdkSurface` or null.
pub unsafe fn get_x11_parent_xid(surface: *mut c_void) -> Option<u64> {
    if !is_surface_type(surface, b"gdk_x11_surface_get_type\0") {
        return None;
    }

    let get_xid: unsafe extern "C" fn(*mut c_void) -> u64 =
        lookup_symbol(b"gdk_x11_surface_get_xid\0")?;
    let xid = get_xid(surface);
    if xid == 0 {
        None
    } else {
        Some(xid)
    }
}

/// Exports the Wayland handle for `surface` via XDG Foreign.
///
/// Calls `on_handle` once the handle is received from the Wayland compositor.
///
/// # Safety
///
/// `surface` must be a valid pointer to a live `GdkSurface` or null.
pub unsafe fn export_wayland_parent_handle<F>(surface: *mut c_void, on_handle: F) -> bool
where
    F: FnOnce(Option<String>) + 'static,
{
    if !is_surface_type(surface, b"gdk_wayland_toplevel_get_type\0") {
        return false;
    }

    type ExportFn = unsafe extern "C" fn(
        *mut c_void,
        Option<unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_void)>,
        *mut c_void,
        Option<unsafe extern "C" fn(*mut c_void)>,
    ) -> glib::ffi::gboolean;

    let export_handle: ExportFn = match lookup_symbol(b"gdk_wayland_toplevel_export_handle\0") {
        Some(f) => f,
        None => return false,
    };

    struct ExportCtx {
        callback: Box<dyn FnOnce(Option<String>)>,
        surface: *mut c_void,
    }

    unsafe extern "C" fn on_exported(
        _toplevel: *mut c_void,
        handle: *const c_char,
        user_data: *mut c_void,
    ) {
        let ctx = Box::from_raw(user_data as *mut ExportCtx);
        let handle_str = if handle.is_null() {
            None
        } else {
            CStr::from_ptr(handle).to_str().ok().map(|s| s.to_string())
        };

        (ctx.callback)(handle_str);

        if let Some(drop_fn) = lookup_symbol::<unsafe extern "C" fn(*mut c_void, *const c_char)>(
            b"gdk_wayland_toplevel_drop_exported_handle\0",
        ) && !handle.is_null()
        {
            drop_fn(ctx.surface, handle);
        }
    }

    let user_data = Box::into_raw(Box::new(ExportCtx {
        callback: Box::new(on_handle),
        surface,
    }));

    let ret = export_handle(surface, Some(on_exported), user_data as *mut c_void, None);
    if ret == 0 {
        drop(Box::from_raw(user_data));
        false
    } else {
        true
    }
}

unsafe fn lookup_symbol<T>(symbol_name: &[u8]) -> Option<T> {
    let sym = libc::dlsym(libc::RTLD_DEFAULT, symbol_name.as_ptr() as *const c_char);
    if sym.is_null() {
        None
    } else {
        Some(std::mem::transmute_copy(&sym))
    }
}

unsafe fn is_surface_type(instance: *mut c_void, type_fn_name: &[u8]) -> bool {
    let get_type: unsafe extern "C" fn() -> glib::ffi::GType = match lookup_symbol(type_fn_name) {
        Some(f) => f,
        None => return false,
    };
    let gtype = get_type();
    if gtype == 0 {
        return false;
    }
    glib::gobject_ffi::g_type_check_instance_is_a(
        instance as *mut glib::gobject_ffi::GTypeInstance,
        gtype,
    ) != 0
}
