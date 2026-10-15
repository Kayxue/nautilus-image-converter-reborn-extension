# Nautilus Image Converter Reborn - Extension

A modern context-menu extension for GNOME Files (**Nautilus 43+ / GTK4 / API 4**) written in Rust.

It integrates seamlessly with the Nautilus right-click context menu, detecting selected image files and providing **"Resize Image(s)..."** and **"Rotate Image(s)..."** options.

---

## Architecture Overview

```
src/
├── lib.rs           # Extension entrypoint, GType registration & Nautilus C ABI exports
├── provider.rs      # Nautilus MenuProvider implementation & context-menu item construction
├── file_utils.rs    # Image MIME type inspection, path resolution & selection filtering
├── actions.rs       # Non-blocking invocation of nautilus-image-converter CLI
└── parent_window.rs # Host GTK4 window detection (X11 XID / Wayland XDG Foreign)
tests/
└── provider_tests.rs # Integration and configuration tests
```

### Key Components

- **`lib.rs`**: Exports the three required entrypoints (`nautilus_module_initialize`, `nautilus_module_list_types`, and `nautilus_module_shutdown`) using the `nautilus_module!` macro.
- **`provider.rs`**: Implements `nautilus_extension::MenuProvider`. Adheres to GNOME HIG by only displaying items when image files are exclusively selected, and handles singular/plural text ("Resize Image..." vs "Resize Images...").
- **`file_utils.rs`**: Inspects Nautilus MIME types and file extensions, strictly filtering local files to the formats supported by the companion app (excluding unsupported formats like SVG or HEIC).
- **`actions.rs`**: Asynchronously invokes the companion app using `nautilus-image-converter [--parent-xid <xid>] [--parent-handle <handle>] [--] <mode> [<paths...>]`. If the app is not installed, it displays a native GTK alert dialog.
- **`parent_window.rs`**: Dynamically queries the host Nautilus GTK4 window to attach the dialog as a modal/transient child (via XID on X11 or Wayland XDG Foreign handle), and displays GTK alert dialogs.


---

## Supported Image Formats

The extension strictly matches the formats supported by the companion application:

| Format | Extensions | MIME Types |
|--------|------------|------------|
| **PNG** | `.png` | `image/png`, `image/x-png` |
| **JPEG** | `.jpg`, `.jpeg`, `.jpe`, `.jfif` | `image/jpeg`, `image/pjpeg` |
| **GIF** | `.gif` | `image/gif` |
| **WebP** | `.webp` | `image/webp` |
| **PNM** | `.pnm`, `.pbm`, `.pgm`, `.ppm`, `.pam` | `image/x-portable-*` |
| **TIFF** | `.tiff`, `.tif` | `image/tiff`, `image/x-tiff` |
| **TGA** | `.tga`, `.icb`, `.vda`, `.vst` | `image/x-tga`, `image/tga` |
| **DDS** | `.dds` | `image/x-dds`, `image/dds` |
| **BMP** | `.bmp`, `.dib` | `image/bmp`, `image/x-bmp` |
| **ICO** | `.ico` | `image/x-icon`, `image/vnd.microsoft.icon` |
| **HDR** | `.hdr`, `.pic` | `image/vnd.radiance`, `image/x-hdr` |
| **OpenEXR** | `.exr` | `image/x-exr` |
| **Farbfeld** | `.ff` | `image/x-farbfeld`, `image/farbfeld` |
| **AVIF** | `.avif` | `image/avif` |
| **QOI** | `.qoi` | `image/qoi`, `image/x-qoi` |

*Note: Vector formats (SVG) and unsupported container formats (such as HEIC/HEIF) are deliberately filtered out.*




---

## Building

Ensure you have Rust (edition 2024 / 1.85+) and `libnautilus-extension-4` installed on your system.

```bash
# Build the release shared object (.so)
cargo build --release

# Run unit and integration tests
cargo test
```

The resulting shared object will be located at:
```
target/release/libnautilus_image_converter_reborn_extension.so
```

---

## Installation

Compiled native (`.so`) Nautilus extensions are loaded from the system extension directory (`/usr/lib/nautilus/extensions-4/`):

```bash
# Copy the compiled shared library to the Nautilus extensions directory
sudo cp target/release/libnautilus_image_converter_reborn_extension.so /usr/lib/nautilus/extensions-4/

# Restart Nautilus to load the extension
nautilus -q
nautilus &
```

---

## Testing & Debugging

1. **Restart Nautilus** to reload extensions:
   ```bash
   nautilus -q
   nautilus &
   ```

2. **Run Nautilus with Extension Debug Logging**:
   ```bash
   nautilus -q
   NAUTILUS_DEBUG=Extensions nautilus
   ```

3. **Configuring the Companion Application Binary**:
   By default, the extension executes `nautilus-image-converter` found in `$PATH` with:
   ```bash
   nautilus-image-converter [--parent-xid <xid>] [--parent-handle <handle>] [--] <mode> [<paths...>]
   ```
   To specify a custom binary or path during development, set `NAUTILUS_IMAGE_CONVERTER_BIN`:
   ```bash
   # Example: specify a custom binary
   NAUTILUS_IMAGE_CONVERTER_BIN=/path/to/nautilus-image-converter nautilus
   ```

