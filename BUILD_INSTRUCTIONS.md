# PepTrack - Build Instructions

## Prerequisites

### macOS
- Xcode Command Line Tools: `xcode-select --install`
- Rust 1.91+: Install from https://rustup.rs/
- Node.js 18+: Install from https://nodejs.org/

### All Platforms
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installations
rust --version
cargo --version
node --version
npm --version
```

## Development Build

### 1. Install Dependencies

```bash
# Install frontend dependencies
cd frontend
npm install
cd ..

# Rust dependencies will be installed during build
```

### 2. Run in Development Mode

```bash
# From project root
cargo tauri dev
```

This will:
- Start the Vite dev server (frontend hot reload)
- Compile and run the Tauri app
- Open the application window

## Production Build

### macOS - DMG Installer

```bash
# From project root
cargo tauri build
```

**Outputs:**
- **DMG:** `src-tauri/target/release/bundle/dmg/PepTrack_0.1.0_x64.dmg`
- **App Bundle:** `src-tauri/target/release/bundle/macos/PepTrack.app`

**Install:**
1. Double-click the DMG file
2. Drag PepTrack.app to Applications folder
3. Right-click and "Open" on first launch (Gatekeeper bypass for unsigned apps)

### macOS - Code Signing (Optional)

To distribute without Gatekeeper warnings, you need an Apple Developer account ($99/year).

```bash
# Set signing identity in src-tauri/tauri.conf.json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)"
    }
  }
}

# Then build
cargo tauri build
```

### Windows - MSI Installer

```bash
cargo tauri build
```

**Outputs:**
- **MSI:** `src-tauri/target/release/bundle/msi/PepTrack_0.1.0_x64_en-US.msi`
- **EXE:** `src-tauri/target/release/PepTrack.exe`

### Linux - AppImage & DEB

```bash
cargo tauri build
```

**Outputs:**
- **AppImage:** `src-tauri/target/release/bundle/appimage/peptrack_0.1.0_amd64.AppImage`
- **DEB:** `src-tauri/target/release/bundle/deb/peptrack_0.1.0_amd64.deb`

## Build Configuration

The build is configured in `src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "category": "Healthcare",
    "macOS": {
      "minimumSystemVersion": "10.15"
    }
  }
}
```

### Customization

**Change App Name:**
- Update `productName` in `src-tauri/tauri.conf.json`
- Update `package.json` in frontend

**Change Version:**
- Update `version` in `src-tauri/tauri.conf.json`
- Update `version` in `src-tauri/Cargo.toml`
- Update `version` in frontend `package.json`

**Change App Icon:**
- Replace icons in `src-tauri/icons/` directory
- Generate all sizes with:
  ```bash
  cargo tauri icon path/to/your/1024x1024.png
  ```

## Troubleshooting

### "xcrun: error" on macOS
```bash
xcode-select --install
sudo xcodebuild -license accept
```

### Missing dependencies on Linux
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### Build fails with "frontend dist not found"
```bash
cd frontend
npm run build
cd ..
cargo tauri build
```

### Slow build times
- First build compiles all dependencies (~10-15 minutes)
- Subsequent builds are much faster (~2-3 minutes)
- Use `cargo tauri build --debug` for faster debug builds

## CI/CD

### GitHub Actions Example

```yaml
name: Build

on:
  push:
    tags:
      - 'v*'

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
      - name: Install dependencies
        run: |
          cd frontend
          npm install
      - name: Build
        run: cargo tauri build
      - name: Upload DMG
        uses: actions/upload-artifact@v4
        with:
          name: PepTrack-macOS
          path: src-tauri/target/release/bundle/dmg/*.dmg
```

## Size Optimization

### Release Build (Optimized)
```bash
# Already configured in Cargo.toml
cargo tauri build --release
```

**Typical sizes:**
- macOS DMG: ~15-20 MB
- Windows MSI: ~10-15 MB
- Linux AppImage: ~15-20 MB

### Strip Debug Symbols (Further reduction)
```toml
# Add to src-tauri/Cargo.toml
[profile.release]
strip = true
```

## Development Tips

### Fast Rebuilds
```bash
# Only rebuild frontend (instant)
cd frontend && npm run build && cd ..

# Only rebuild Rust (if Rust code changed)
cargo build --manifest-path src-tauri/Cargo.toml
```

### Debug Logging
```bash
# Enable all logs
RUST_LOG=debug cargo tauri dev

# Filter specific modules
RUST_LOG=peptrack=debug,tauri=info cargo tauri dev
```

### Hot Reload
- Frontend changes: Auto-reload (Vite HMR)
- Rust changes: Requires restart of `cargo tauri dev`

## Testing Before Release

1. **Run all tests:**
   ```bash
   cargo test --workspace
   cd frontend && npm run test -- --run
   ```

2. **Build and test the release binary:**
   ```bash
   cargo tauri build
   # Test the built app manually
   ```

3. **Check for warnings:**
   ```bash
   cargo clippy --workspace --all-targets
   ```

4. **Verify signing (macOS):**
   ```bash
   codesign -vv ./src-tauri/target/release/bundle/macos/PepTrack.app
   ```

## Distribution

### macOS
- **Unsigned:** Users must right-click > Open first time
- **Signed:** Requires Apple Developer account ($99/year)
- **Notarized:** Additional step for distribution outside App Store

### Windows
- **Unsigned:** Windows Defender SmartScreen warning on first run
- **Signed:** Requires code signing certificate (~$100-400/year)

### Linux
- **AppImage:** Universal, works everywhere
- **DEB:** Debian/Ubuntu package managers
- **Snap/Flatpak:** Additional packaging required

## Support

For build issues:
- Check Tauri docs: https://tauri.app/v2/guides/building/
- GitHub Issues: https://github.com/cboyd0319/PepTrack/issues

## License

See LICENSE file in repository root.
