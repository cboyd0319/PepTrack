# Building a Self-Signed `.pkg` Installer on macOS

This guide walks you through generating a self-signed macOS installer so you can grant PepTrack the filesystem/automation permissions it needs during testing.

## 1. Create Self-Signed Certificates (one-time)

You need two certificates stored in your login keychain:

| Certificate | Purpose | Suggested Name |
|-------------|---------|----------------|
| Code Signing | Codesign `PepTrack.app` | `PepTrack Dev` |
| Installer Signing | Sign the `.pkg` produced by `productbuild` | `PepTrack Dev Installer` |

### Using Keychain Access (GUI)
1. Open **Keychain Access → Certificate Assistant → Create a Certificate…**
2. Name it `PepTrack Dev`, set **Identity Type** to *Self Signed Root* and **Certificate Type** to *Code Signing*, then click *Create*.
3. Repeat the process for `PepTrack Dev Installer` but choose **Certificate Type → Developer ID Installer** (or *Code Signing* if that option is unavailable—macOS accepts it for local installs).
4. Ensure both certificates are in the **login** keychain and trusted for code signing.

### CLI Alternative
```bash
security create-keychain -p peptrack peptrack-build.keychain
security add-certificates -k login.keychain-db /path/to/generated/certs.cer
```
*(Use the GUI flow above to export/import if you prefer scripting the trust setup.)*

## 2. Export Certificate Names for the Build

```bash
export PEPTRACK_APP_CERT="PepTrack Dev"
export PEPTRACK_INSTALLER_CERT="PepTrack Dev Installer"
```

If you prefer ad-hoc signing, set either variable to `-` (codesign) or omit the installer certificate and remove `--sign` from the script, but Gatekeeper will flag the installer as “unidentified developer”.

## 3. Build the PKG

```bash
# From repo root
./scripts/build_pkg.sh
```

What the script does:
1. Runs `cargo tauri build --bundles app` (produces `src-tauri/target/release/bundle/macos/PepTrack.app`).
2. Codesigns the `.app` with `codesign --force --deep --options runtime --timestamp --sign "$PEPTRACK_APP_CERT"`.
3. Runs `productbuild --component ... --sign "$PEPTRACK_INSTALLER_CERT"` to create `dist/PepTrack-selfsigned.pkg`.

## 4. Verify the Signatures

```bash
codesign --verify --deep --strict src-tauri/target/release/bundle/macos/PepTrack.app
spctl --assess --type exec src-tauri/target/release/bundle/macos/PepTrack.app
productsign --verify dist/PepTrack-selfsigned.pkg
spctl --assess --type install dist/PepTrack-selfsigned.pkg
```

Because the certificates are self-signed, macOS will still warn, but you can approve them once in **System Settings → Privacy & Security** after attempting an install.

## 5. Install and Grant Permissions

1. Double-click `dist/PepTrack-selfsigned.pkg`.
2. When prompted by Gatekeeper, choose **Open Anyway** in **Privacy & Security**.
3. Complete the installer; PepTrack is copied to `/Applications`.
4. Launch the app, then grant File System / Downloads / Documents permissions via the system prompts. Now that the app is installed from a PKG, permissions can be managed centrally under **System Settings → Privacy & Security → Files and Folders / Automation**.

## Tips
- To rebuild quickly after code changes, remove the old bundle: `rm -rf src-tauri/target/release/bundle/macos dist`.
- If you change certificate names, update `PEPTRACK_APP_CERT` / `PEPTRACK_INSTALLER_CERT`.
- For CI experimentation, you can store the certificates as password-protected `.p12` files and import them with `security import`.
