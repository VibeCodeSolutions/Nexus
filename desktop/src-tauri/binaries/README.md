# Sidecar-Binaries

Dieses Verzeichnis enthält die plattformspezifisch umbenannten `nexus-core`-Binaries,
die Tauri als Sidecar-Prozess startet.

## Namenskonvention

Tauri erwartet den Sidecar unter exakt diesem Namen, inklusive Rust-Target-Triple:

- Linux x86_64:   `nexus-core-x86_64-unknown-linux-gnu`
- macOS Apple:    `nexus-core-aarch64-apple-darwin`
- macOS Intel:    `nexus-core-x86_64-apple-darwin`
- Windows x86_64: `nexus-core-x86_64-pc-windows-msvc.exe`

## Lokaler Dev-Setup (einmalig nach Core-Änderungen)

```bash
cd core
cargo build --release
cp target/release/nexus-core \
   ../desktop/src-tauri/binaries/nexus-core-x86_64-unknown-linux-gnu
chmod +x ../desktop/src-tauri/binaries/nexus-core-x86_64-unknown-linux-gnu
```

Dann aus `desktop/`:

```bash
pnpm tauri dev   # oder npm run tauri dev
```

## CI

Die Release-Builds füllen dieses Verzeichnis im Build-Job (Phase 7).
Die tatsächlichen Binaries sind per `.gitignore` ausgenommen; nur diese README wird versioniert.
