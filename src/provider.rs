use crate::actions::{execute_action, ImageAction};
use crate::file_utils::{are_all_images, extract_image_paths};
use nautilus_extension::{FileInfo, Menu, MenuItem, MenuProvider};

/// Menu provider that adds image transformation entries to Nautilus context menus.
#[derive(Default, Debug, Clone)]
pub struct ImageConverterMenuProvider {
    /// When true, wraps the actions inside a parent submenu. Defaults to false (top-level items).
    pub use_submenu: bool,
    /// When true, includes the "Convert Image(s)..." item. Defaults to false.
    pub enable_convert: bool,
}

impl ImageConverterMenuProvider {
    /// Creates a new provider instance with top-level context menu items.
    pub fn new() -> Self {
        Self {
            use_submenu: false,
            enable_convert: false,
        }
    }

    /// Sets whether items should be organized into an "Image Operations" submenu.
    pub fn with_submenu(mut self, use_submenu: bool) -> Self {
        self.use_submenu = use_submenu;
        self
    }

    /// Sets whether the Convert Image(s) menu item should be shown.
    pub fn with_convert(mut self, enable_convert: bool) -> Self {
        self.enable_convert = enable_convert;
        self
    }
}

impl MenuProvider for ImageConverterMenuProvider {
    fn get_file_items(&self, files: &[FileInfo]) -> Vec<MenuItem> {
        // Only display context menu items if all selected files are valid image files.
        // This adheres to GNOME HIG by not cluttering menus during mixed selections.
        if !are_all_images(files) {
            return Vec::new();
        }

        let count = files.len();

        let resize_label = if count == 1 {
            "Resize Image..."
        } else {
            "Resize Images..."
        };

        let rotate_label = if count == 1 {
            "Rotate Image..."
        } else {
            "Rotate Images..."
        };

        let resize_item = MenuItem::new(ImageAction::Resize.id_str(), resize_label).on_activate(
            |activation| {
                let paths = extract_image_paths(activation.files());
                execute_action(ImageAction::Resize, &paths);
            },
        );

        let rotate_item = MenuItem::new(ImageAction::Rotate.id_str(), rotate_label).on_activate(
            |activation| {
                let paths = extract_image_paths(activation.files());
                execute_action(ImageAction::Rotate, &paths);
            },
        );

        let mut items = vec![resize_item, rotate_item];

        if self.enable_convert {
            let convert_label = if count == 1 {
                "Convert Image..."
            } else {
                "Convert Images..."
            };
            let convert_item = MenuItem::new(ImageAction::Convert.id_str(), convert_label)
                .on_activate(|activation| {
                    let paths = extract_image_paths(activation.files());
                    execute_action(ImageAction::Convert, &paths);
                });
            items.push(convert_item);
        }

        if self.use_submenu {
            let submenu = Menu::new(items);
            vec![MenuItem::new("ImageConverter::menu", "Image Operations").with_submenu(submenu)]
        } else {
            items
        }
    }



    fn get_background_items(&self, _current_folder: &FileInfo) -> Vec<MenuItem> {
        // Image resize and rotate are file-specific actions, so folder background menus remain empty.
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_instantiation() {
        let provider = ImageConverterMenuProvider::new();
        assert!(!provider.use_submenu);

        let provider_submenu = ImageConverterMenuProvider::new().with_submenu(true);
        assert!(provider_submenu.use_submenu);
    }
}
