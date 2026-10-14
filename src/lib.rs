//! # Nautilus Image Converter Reborn Extension
//!
//! A modern Nautilus 43+ (GTK4 / API 4) extension written in Rust that provides
//! context menu integration for resizing and rotating images.

pub mod actions;
pub mod file_utils;
pub mod parent_window;
pub mod provider;


pub use actions::{execute_action, ImageAction};
pub use provider::ImageConverterMenuProvider;

use nautilus_extension::gobject_ffi::GTypeModule;
use nautilus_extension::{nautilus_module, NautilusModule};

/// Extension registration function invoked by Nautilus's module loader.
fn register(module: *mut GTypeModule) -> nautilus_extension::glib_ffi::GType {
    // Register our GObject module and attach the MenuProvider interface
    let mut module = NautilusModule::new(module, "ImageConverterExtension");
    module.add_menu_provider(ImageConverterMenuProvider::new());
    module.register()
}


// Generates the native C entry points required by Nautilus:
// - `nautilus_module_initialize`
// - `nautilus_module_list_types`
// - `nautilus_module_shutdown`
nautilus_module!(register);
