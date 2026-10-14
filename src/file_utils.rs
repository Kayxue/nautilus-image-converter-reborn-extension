use nautilus_extension::FileInfo;
use std::path::PathBuf;
use url::Url;

/// Supported image formats matching the capabilities of the companion `image` crate:
/// - Png
/// - Jpeg
/// - Gif
/// - WebP
/// - Pnm (PBM, PGM, PPM, PAM)
/// - Tiff
/// - Tga
/// - Dds
/// - Bmp
/// - Ico
/// - Hdr (Radiance HDR)
/// - OpenExr
/// - Farbfeld
/// - Avif
/// - Qoi
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    // Png
    "png",
    // Jpeg
    "jpg", "jpeg", "jpe", "jfif",
    // Gif
    "gif",
    // WebP
    "webp",
    // Pnm (portable anymap / bitmap / graymap / pixmap / arbitrarymap)
    "pnm", "pbm", "pgm", "ppm", "pam",
    // Tiff
    "tiff", "tif",
    // Tga
    "tga", "icb", "vda", "vst",
    // Dds
    "dds",
    // Bmp
    "bmp", "dib",
    // Ico
    "ico",
    // Hdr
    "hdr", "pic",
    // OpenExr
    "exr",
    // Farbfeld
    "ff",
    // Avif
    "avif",
    // Qoi
    "qoi",
];

/// Known MIME types corresponding strictly to the supported image formats.
pub const SUPPORTED_MIME_TYPES: &[&str] = &[
    // Png
    "image/png",
    "image/x-png",
    // Jpeg
    "image/jpeg",
    "image/pjpeg",
    // Gif
    "image/gif",
    // WebP
    "image/webp",
    // Pnm
    "image/x-portable-anymap",
    "image/x-portable-bitmap",
    "image/x-portable-graymap",
    "image/x-portable-pixmap",
    "image/x-portable-arbitrarymap",
    // Tiff
    "image/tiff",
    "image/x-tiff",
    // Tga
    "image/x-tga",
    "image/x-targa",
    "image/tga",
    // Dds
    "image/x-dds",
    "image/dds",
    // Bmp
    "image/bmp",
    "image/x-bmp",
    "image/x-ms-bmp",
    // Ico
    "image/x-icon",
    "image/vnd.microsoft.icon",
    "image/ico",
    // Hdr
    "image/vnd.radiance",
    "image/x-hdr",
    "image/hdr",
    // OpenExr
    "image/x-exr",
    // Farbfeld
    "image/x-farbfeld",
    "image/farbfeld",
    // Avif
    "image/avif",
    // Qoi
    "image/qoi",
    "image/x-qoi",
];

/// Checks whether a MIME type is in the allowed format list.
pub fn is_supported_mime_type(mime: &str) -> bool {
    SUPPORTED_MIME_TYPES.contains(&mime)
}

/// Checks whether a file extension (without leading dot) is in the allowed format list.
pub fn is_supported_extension(ext: &str) -> bool {
    SUPPORTED_EXTENSIONS.contains(&ext)
}

/// Checks if a Nautilus [`FileInfo`] represents a valid local image matching
/// the supported image formats.
pub fn is_image_file(file: &FileInfo) -> bool {
    // Ignore deleted / gone files and directories
    if file.is_gone() || file.is_directory() {
        return false;
    }

    // Only process local files
    if file.uri_scheme().is_some_and(|scheme| scheme != "file") {
        return false;
    }

    // 1. Primary check: check reported MIME type against allowed list
    if let Some(mime) = file.mime_type() {
        let mime_lower = mime.to_ascii_lowercase();
        if is_supported_mime_type(&mime_lower) {
            return true;
        }
    }

    // 2. Fallback check: check file extension if MIME is missing or generic
    if let Some(path) = get_file_path(file) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_ascii_lowercase();
            return is_supported_extension(&ext_lower);
        }
    } else if let Some(uri_str) = file.uri()
        && let Ok(parsed_url) = Url::parse(&uri_str)
        && let Ok(path) = parsed_url.to_file_path()
        && let Some(ext) = path.extension().and_then(|e| e.to_str())
    {
        let ext_lower = ext.to_ascii_lowercase();
        return is_supported_extension(&ext_lower);
    }

    false
}

/// Resolves the filesystem path for a [`FileInfo`].
///
/// Tries `FileInfo::location_path()` first, falling back to parsing the URI.
pub fn get_file_path(file: &FileInfo) -> Option<PathBuf> {
    if let Some(path) = file.location_path() {
        return Some(path);
    }

    let uri_str = file.uri()?;
    let url = Url::parse(&uri_str).ok()?;
    url.to_file_path().ok()
}

/// Returns `true` if the slice is non-empty and every item is a supported image file.
pub fn are_all_images(files: &[FileInfo]) -> bool {
    !files.is_empty() && files.iter().all(is_image_file)
}

/// Returns `true` if at least one file in the slice is a supported image file.
pub fn has_any_image(files: &[FileInfo]) -> bool {
    files.iter().any(is_image_file)
}

/// Extracts local filesystem [`PathBuf`]s for all valid image files in the slice.
pub fn extract_image_paths(files: &[FileInfo]) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|file| is_image_file(file))
        .filter_map(get_file_path)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_15_supported_extensions() {
        let expected_formats = [
            "png", "jpg", "jpeg", "gif", "webp", "pnm", "tiff", "tga", "dds", "bmp", "ico",
            "hdr", "exr", "ff", "avif", "qoi",
        ];
        for ext in expected_formats {
            assert!(
                is_supported_extension(ext),
                "Expected extension '{ext}' to be supported"
            );
        }
    }

    #[test]
    fn test_unsupported_formats_rejected() {
        let unsupported = [
            "svg", "heic", "heif", "pdf", "psd", "xcf", "ai", "raw", "cr2", "nef", "mp4", "txt",
        ];
        for ext in unsupported {
            assert!(
                !is_supported_extension(ext),
                "Unsupported extension '{ext}' should be rejected"
            );
        }

        assert!(!is_supported_mime_type("image/svg+xml"));
        assert!(!is_supported_mime_type("image/heic"));
        assert!(!is_supported_mime_type("image/heif"));
        assert!(!is_supported_mime_type("application/pdf"));
    }
}
