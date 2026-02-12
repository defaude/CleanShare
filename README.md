# CleanShare

<img src="./CleanShare_logo.png" alt="CleanShare logo" width="320" />

Cross-platform MVP for cleaning shared text: only tracking parameters are removed from URLs, and the rest of the text stays unchanged.

## Architecture

- `crates/link_cleaner_core`: pure Rust rule engine (no OS/UI dependencies)
- `crates/link_cleaner_wasm`: `wasm-bindgen` wrapper exporting `clean_text(input: string): string`
- `crates/link_cleaner_uniffi`: UniFFI scaffold for future Swift/Kotlin bindings
- `apps/desktop_tauri`: Tauri v2 desktop MVP (live cleaning + copy + clipboard monitoring with OS notification)
- `apps/web_demo`: minimal web demo (Vite + TypeScript) with WASM integration

## Prerequisites

- Git
- Node.js LTS + npm
- Rust via `rustup` (including `rustfmt` and `clippy`)
- `wasm-pack`
- Rust target `wasm32-unknown-unknown`
- Tauri prerequisites per OS:
1. macOS: Xcode Command Line Tools
2. Windows: Visual Studio Build Tools (Desktop C++ workload)
3. Linux: distro-specific WebView/GTK system packages

Official Tauri prerequisites: [https://v2.tauri.app/start/prerequisites/](https://v2.tauri.app/start/prerequisites/)

## Setup

```bash
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked
```

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

## Cleaning Rules v0

Tracking parameters are removed conservatively:

- All query parameters with the `utm_` prefix
- `gclid`, `fbclid`, `mc_cid`, `mc_eid`, `ref`, `ref_src`, `ref_url`, `igshid`, `igsh`, `si`
- Amazon-specific (v0): remove `tag`, `linkCode`, and strip path segments starting at `/ref=`

Intentionally kept:

- Semantic parameters such as `t` (Twitter/X timestamp)
- Fragments `#...`

## Golden Tests (Core)

The 12 requested golden cases are located in `crates/link_cleaner_core/src/lib.rs` and validate exact input -> output behavior for YouTube, UTM, gclid/fbclid, Mailchimp, Amazon, Instagram, TikTok, X, Google Maps, multiple URLs/punctuation, no URL, and fragment preservation.

## Known Limitations

- URL detection is based on `http://` / `https://`; links without a scheme are not detected in v0.
- Trailing punctuation handling is conservative and covers standard cases.
- Mobile apps are not included yet; UniFFI is prepared as a scaffold.

## Next Steps

- Version domain-specific rules (feature flags/rule sets)
- Show `clean_text_with_report` in the web/desktop UI
- Integrate UniFFI bindings into mobile host apps (iOS/Android)
