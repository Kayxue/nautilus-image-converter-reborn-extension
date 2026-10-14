use crate::parent_window;
use std::ffi::c_void;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Operation modes supported by `nautilus-image-converter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageAction {
    /// Resize images.
    Resize,
    /// Rotate images.
    Rotate,
    /// Convert image formats.
    Convert,
}

impl ImageAction {
    /// Positional `mode` string argument accepted by `nautilus-image-converter`.
    pub fn mode_str(&self) -> &'static str {
        match self {
            ImageAction::Resize => "resize",
            ImageAction::Rotate => "rotate",
            ImageAction::Convert => "convert",
        }
    }

    /// Menu item identifier for Nautilus.
    pub fn id_str(&self) -> &'static str {
        match self {
            ImageAction::Resize => "ImageConverter::resize",
            ImageAction::Rotate => "ImageConverter::rotate",
            ImageAction::Convert => "ImageConverter::convert",
        }
    }
}

/// Locates the `nautilus-image-converter` executable in the system.
///
/// Checks environment variables `NAUTILUS_IMAGE_CONVERTER_BIN` / `IMAGE_CONVERTER_BIN`
/// first, then searches the directories listed in `$PATH`.
pub fn find_app_binary() -> Option<PathBuf> {
    if let Ok(path_str) = std::env::var("NAUTILUS_IMAGE_CONVERTER_BIN")
        .or_else(|_| std::env::var("IMAGE_CONVERTER_BIN"))
    {
        let path = PathBuf::from(&path_str);
        if path.is_file() {
            return Some(path);
        }
    }

    let binary_name = "nautilus-image-converter";
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(binary_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Checks whether the companion `nautilus-image-converter` application is installed.
pub fn is_app_installed() -> bool {
    find_app_binary().is_some()
}

/// Dispatches the image action to the standalone `nautilus-image-converter` application.
///
/// If the application is not installed, it displays a native GTK alert dialog
/// attached to Nautilus informing the user to install `nautilus-image-converter-reborn-app`.
pub fn execute_action(action: ImageAction, paths: &[PathBuf]) {
    if paths.is_empty() {
        eprintln!("[nautilus-image-converter-reborn] No image files selected for {action:?}");
        return;
    }

    let parent_window = unsafe { parent_window::get_active_nautilus_window() };

    // Check if the companion application is installed
    let Some(binary_path) = find_app_binary() else {
        eprintln!(
            "[nautilus-image-converter-reborn] Companion app 'nautilus-image-converter' is not installed."
        );
        unsafe {
            parent_window::show_missing_app_dialog(parent_window);
        }
        return;
    };

    let paths_clone = paths.to_vec();

    // Attempt to detect host Nautilus window surface for transient-for binding
    if let Some(surface) = unsafe { parent_window::get_active_nautilus_surface() } {
        // 1. Try X11 / XWayland parent XID
        if let Some(xid) = unsafe { parent_window::get_x11_parent_xid(surface) } {
            spawn_converter(&binary_path, action, &paths_clone, Some(xid), None, parent_window);
            return;
        }

        // 2. Try Wayland XDG Foreign exported handle
        let paths_wayland = paths_clone.clone();
        let binary_path_clone = binary_path.clone();
        let exported = unsafe {
            parent_window::export_wayland_parent_handle(surface, move |handle_opt| {
                spawn_converter(
                    &binary_path_clone,
                    action,
                    &paths_wayland,
                    None,
                    handle_opt.as_deref(),
                    parent_window,
                );
            })
        };

        if exported {
            return;
        }
    }

    // 3. Fallback: spawn standalone without parent window binding
    spawn_converter(&binary_path, action, &paths_clone, None, None, parent_window);
}

/// Spawns `nautilus-image-converter` with the configured arguments.
fn spawn_converter(
    binary_path: &PathBuf,
    action: ImageAction,
    paths: &[PathBuf],
    parent_xid: Option<u64>,
    parent_handle: Option<&str>,
    parent_window: Option<*mut c_void>,
) {
    let mut cmd = Command::new(binary_path);

    if let Some(xid) = parent_xid {
        cmd.arg("--parent-xid").arg(xid.to_string());
    }

    if let Some(handle) = parent_handle {
        cmd.arg("--parent-handle").arg(handle);
    }

    cmd.arg("--");
    cmd.arg(action.mode_str());
    for path in paths {
        cmd.arg(path);
    }

    cmd.stdin(Stdio::null());

    match cmd.spawn() {
        Ok(child) => {
            eprintln!(
                "[nautilus-image-converter-reborn] Launched '{}' (PID: {}) with mode '{}' (xid: {:?}, handle: {:?})",
                binary_path.display(),
                child.id(),
                action.mode_str(),
                parent_xid,
                parent_handle
            );
        }
        Err(err) => {
            eprintln!(
                "[nautilus-image-converter-reborn] Could not launch '{}': {err}",
                binary_path.display()
            );
            if err.kind() == std::io::ErrorKind::NotFound {
                unsafe {
                    parent_window::show_missing_app_dialog(parent_window);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_action_modes_and_ids() {
        assert_eq!(ImageAction::Resize.mode_str(), "resize");
        assert_eq!(ImageAction::Rotate.mode_str(), "rotate");
        assert_eq!(ImageAction::Convert.mode_str(), "convert");

        assert_eq!(ImageAction::Resize.id_str(), "ImageConverter::resize");
        assert_eq!(ImageAction::Rotate.id_str(), "ImageConverter::rotate");
        assert_eq!(ImageAction::Convert.id_str(), "ImageConverter::convert");
    }

    #[test]
    fn test_binary_lookup() {
        // /usr/bin/nautilus-image-converter is installed on the system
        let bin = find_app_binary();
        assert!(bin.is_some());
        assert!(is_app_installed());
    }
}
