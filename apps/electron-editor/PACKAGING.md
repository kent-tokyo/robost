# Robost Editor - Packaging & Distribution Guide

## Overview

Robost Editor is an Electron application with a bundled Rust RPA CLI backend. This guide covers building and packaging the application for macOS and Windows distribution.

## Project Structure

```
apps/electron-editor/
├── src/
│   ├── main/           # Electron main process
│   ├── renderer/       # React UI application
│   └── index.html      # Entry point
├── assets/
│   ├── rpa/            # RPA binary resources (platform-specific)
│   │   ├── darwin-arm64/rpa
│   │   ├── darwin-x64/rpa
│   │   └── win32-x64/rpa.exe
│   └── entitlements.plist   # macOS security settings
├── forge.config.js     # Electron Forge configuration
├── package.json        # Dependencies & scripts
└── webpack.*.config.js # Build configuration
```

## Prerequisites

### macOS
- Xcode Command Line Tools: `xcode-select --install`
- Rust toolchain for cross-compilation:
  ```bash
  rustup target add aarch64-apple-darwin x86_64-apple-darwin
  ```
- Code signing certificate (for distribution):
  - Developer ID Certificate from Apple Developer Program

### Windows
- Windows SDK
- Rust toolchain:
  ```bash
  rustup target add x86_64-pc-windows-gnu
  ```
- Optionally: MinGW for Windows binary compilation

### Both Platforms
- Node.js 18+ and npm
- Cargo (Rust package manager)

## Building Process

### Step 1: Prepare RPA Binaries

Build the Rust CLI backend for each target platform:

```bash
cd apps/electron-editor
npm run build:rpa
```

This script:
- Builds `robost-cli` with `--release` flag for each target
- Copies binaries to `assets/rpa/{platform}/`
- Skips targets unavailable on the current platform

**Platform-specific binaries:**
- `assets/rpa/darwin-arm64/rpa` — macOS ARM64 (Apple Silicon)
- `assets/rpa/darwin-x64/rpa` — macOS x64 (Intel)
- `assets/rpa/win32-x64/rpa.exe` — Windows x64

### Step 2: Install Dependencies

```bash
cd apps/electron-editor
npm install
```

### Step 3: Development Build & Test

```bash
npm run start
```

This launches the app in development mode with hot reload.

### Step 4: Package the Application

#### macOS

Build DMG and ZIP packages:

```bash
npm run make:mac
```

Output:
- `out/make/zip/darwin/x64/robost-editor-1.0.0-darwin.zip`
- `out/make/dmg/robost-editor-1.0.0.dmg`

#### Windows

Build NSIS installer and Squirrel packages:

```bash
npm run make:win
```

Output:
- `out/make/squirrel.windows/x64/robost-editor-1.0.0 Setup.exe`
- `out/make/zip/win32/x64/robost-editor-1.0.0-win.zip`

#### All Platforms

```bash
npm run make
```

## Code Signing (Optional but Recommended)

### macOS Code Signing

Electron Forge uses `osxSign` configuration in `forge.config.js`:

```javascript
osxSign: {
  identity: 'Developer ID Application',
  hardendedRuntime: true,
  entitlements: 'assets/entitlements.plist',
  entitlementsInherit: 'assets/entitlements.plist',
}
```

To sign:
1. Obtain Developer ID certificate from Apple Developer Program
2. Import into Keychain
3. Set environment variable:
   ```bash
   export APPLE_ID="your-id@apple.com"
   export APPLE_TEAM_ID="YOUR_TEAM_ID"
   export APPLE_ID_PASSWORD="@keychain:PROFILE_NAME"
   ```
4. Run: `npm run make:mac`

### Windows Code Signing

Configure in `forge.config.js`:

```javascript
{
  name: '@electron-forge/maker-squirrel',
  config: {
    certificateFile: process.env.WINDOWS_CERT_FILE,
    certificatePassword: process.env.WINDOWS_CERT_PASSWORD,
  }
}
```

Set environment variables:
```bash
export WINDOWS_CERT_FILE="/path/to/cert.pfx"
export WINDOWS_CERT_PASSWORD="password"
npm run make:win
```

## Distribution

### Direct Distribution
1. Upload `.dmg` (macOS) or `.exe` (Windows) to your hosting
2. Users download and install manually

### Notarization (macOS)

For Apple Gatekeeper compatibility:

```bash
npm run make:mac
# Notarization is attempted automatically if APPLE_ID is set
```

### Auto-Updates

Configure electron-updater in `electron-updater.yml`:

```yaml
owner: username
repo: robost-editor
provider: github
releaseType: release
```

## Development Notes

### Modifying RPA Binary References

If you change the RPA binary location or naming:

1. Update `forge.config.js` `extraResources` section
2. Update `src/main/rpaManager.ts` `locateRpaBinary()` method
3. Rebuild: `npm run build:rpa` and `npm run make`

### Testing Packaged App

After building:

```bash
# macOS
open out/make/dmg/robost-editor-1.0.0.dmg

# Windows
.\out\make\squirrel.windows\x64\robost-editor-1.0.0\ Setup.exe
```

## Troubleshooting

### RPA Binary Not Found

If the app starts but RPA commands fail:

1. Check binary exists: `ls assets/rpa/*/rpa*`
2. Ensure executable: `chmod +x assets/rpa/*/rpa*`
3. Check `rpaManager.ts` logs in DevTools (F12)

### Code Signing Errors (macOS)

- Ensure certificate is in Keychain: `security find-identity -v -p codesigning`
- Verify `entitlements.plist` syntax
- Check Xcode version: `xcode-select --print-path`

### Windows Build Fails

- Install Visual Studio Build Tools
- Ensure Rust Windows target: `rustup target add x86_64-pc-windows-gnu`

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build & Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          targets: aarch64-apple-darwin,x86_64-apple-darwin,x86_64-pc-windows-gnu
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: npm run build:rpa
      - run: npm run make
      - uses: softprops/action-gh-release@v1
        with:
          files: out/make/**/*
```
