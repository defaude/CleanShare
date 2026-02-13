# CleanShare

![CleanShare logo](./CleanShare_logo.png)

Just removes tracking stuff from URLs. That's it.

Right now, there's just a desktop app based on Tauri (and I hope it actually works on OS other than MacOS). It sits in
your menu bar / system tray and monitors the clipboard for texts that contain URLs with tracking parameters. If one is
found, the tracking stuff is removed from the text and the cleaned text is written back to the clipboard.

You can disable the clipboard monitor via the checkbox in the UI and manually copy your text to be cleaned into the
text field instead.

## Installation

Right now, there's no official release. 👉 Clone this repo, build the whole thing, and run the desktop app.

### Prerequisites

- Node.js LTS (with npm)
- Rust stable toolchain (rustup + cargo)

```shell
cd apps/desktop_tauri
npm install
npm run tauri:build
```

The built app will be in `desktop_tauri/src-tauri/target/release/bundle` somewhere 😅


## Architecture

- `crates/link_cleaner_core`: pure Rust rule engine (no OS/UI dependencies)
- `crates/link_cleaner_wasm`: `wasm-bindgen` wrapper exporting `clean_text(input: string): string`
- `crates/link_cleaner_uniffi`: UniFFI scaffold for future Swift/Kotlin bindings
- `apps/desktop_tauri`: Tauri v2 desktop MVP (live cleaning + copy + clipboard monitoring with OS notification)
- `apps/web_demo`: minimal web demo (Vite + TypeScript) with WASM integration

## Setup

## Rust Checks and Tests

In the repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## WASM Build and Web Demo

Build the WASM package directly from `link_cleaner_wasm`:

```bash
wasm-pack build crates/link_cleaner_wasm --target web --out-dir pkg --out-name link_cleaner_wasm
```

Start the web demo:

```bash
cd apps/web_demo
npm install
npm run dev
```

Note: `npm run dev` first builds the WASM package from `../../crates/link_cleaner_wasm` into `apps/web_demo/pkg`.

## Desktop App (Tauri v2)

```bash
cd apps/desktop_tauri
npm install
npm run tauri:dev
```

The Tauri backend calls `link_cleaner_core::clean_text` through the `clean_text` command.
It also monitors the clipboard: when tracking parameters are detected, the cleaned text is written back to the clipboard and a system notification is shown.
The monitor can be toggled via the checkbox in the top-right of the UI.

Alternative (if the Rust Tauri CLI is installed):

```bash
cd apps/desktop_tauri/src-tauri
cargo tauri dev
```

Icon assets are committed (instead of generating them on every build) for reproducible packaging.
When branding changes, regenerate icons from the source logo:

```bash
cd apps/desktop_tauri
npm run tauri -- icon ../../CleanShare_logo.png
```

Then keep only this minimal desktop icon set in `apps/desktop_tauri/src-tauri/icons`:
- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.png`
- `icon.icns`
- `icon.ico`

## UniFFI Scaffold

Generate bindings later (after installing `uniffi-bindgen`):

```bash
cargo install uniffi_bindgen
cd crates/link_cleaner_uniffi
./scripts/generate-bindings.sh swift
./scripts/generate-bindings.sh kotlin
```
