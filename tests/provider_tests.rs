use nautilus_image_converter_reborn_extension::actions::ImageAction;
use nautilus_image_converter_reborn_extension::provider::ImageConverterMenuProvider;

#[test]
fn test_menu_provider_configuration() {
    let default_provider = ImageConverterMenuProvider::new();
    assert!(!default_provider.use_submenu);
    assert!(!default_provider.enable_convert);

    let configured_provider = ImageConverterMenuProvider::new()
        .with_submenu(true)
        .with_convert(true);
    assert!(configured_provider.use_submenu);
    assert!(configured_provider.enable_convert);
}


#[test]
fn test_image_actions() {
    assert_eq!(ImageAction::Resize.mode_str(), "resize");
    assert_eq!(ImageAction::Rotate.mode_str(), "rotate");
    assert_eq!(ImageAction::Convert.mode_str(), "convert");

    assert_eq!(ImageAction::Resize.id_str(), "ImageConverter::resize");
    assert_eq!(ImageAction::Rotate.id_str(), "ImageConverter::rotate");
    assert_eq!(ImageAction::Convert.id_str(), "ImageConverter::convert");
}


