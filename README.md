# Nautilus Image Converter Reborn - Extension

Extension project of Nautilus Image Converter Reborn.

## Installation
### Ubuntu (or other Debian-based distro)
Download package from [release page](https://github.com/Kayxue/nautilus-image-converter-reborn-extension/releases), and install it by double clicking on it or through command below:
```bash
sudo apt install <path-to-deb-package>
```

### Fedora (or other RedHat-based distro)
Download package from [release page](https://github.com/Kayxue/nautilus-image-converter-reborn-extension/releases), and install it through command below:
```bash
sudo dnf install <path-to-rpm-package>
```

### Arch Linux (or other ArchLinux-based distro)
First, clone the repository through Git CLI.
```bash
git clone https://github.com/Kayxue/nautilus-image-converter-reborn-extension-arch.git
```
Second, open terminal, `cd` into the folder and run command below:
```bash
makepkg -si
```

## Repository

* Extensions repository: This repository
* App repository: [Here](https://github.com/Kayxue/nautilus-image-converter-reborn-app)

## Technology Stack (Extension part)

- Rust
- nautilus-extension (thanks for [this PR](https://github.com/talklittle/nautilus-extension-rs/pull/12))
- glib
- gio
- libc

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

## AI Contribution Policy
Similar to [gtk-rs](https://github.com/gtk-rs/gtk4-rs#ai-contribution-policy)

## Contribution
Welcome to contribute to this project. Feel free to open an issue or submit a pull request. But:
1. Please follow [Code of Conduct](https://rust-lang.org/policies/code-of-conduct/)
2. Please format files and confirm your code before pushing to GitHub
