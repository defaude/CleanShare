# CleanShare

![CleanShare logo](./CleanShare_logo.png)

CleanShare removes tracking parameters from URLs. The project currently targets two delivery formats:

- a macOS Tauri app for local use on a computer
- a static web app

Both apps use the same Rust cleaning logic. WASM is used only to expose that Rust logic to the static web app.

## Project Structure

- `crates/link_cleaner_core`: shared Rust cleaning engine without UI or platform dependencies
- `crates/link_cleaner_wasm`: `wasm-bindgen` web bindings for `link_cleaner_core`
- `apps/desktop_tauri`: macOS Tauri v2 app with clipboard monitoring and manual cleaning
- `apps/web_demo`: static Vite web app with WASM integration

## Requirements

- Node.js with npm
- Rust with `rustup` and `cargo`
- Rust `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- `wasm-pack`:
  ```bash
  cargo install wasm-pack
  ```

## Desktop App

The desktop app is the macOS local app. It can monitor the clipboard for texts that contain URLs with tracking
parameters, clean the text, and write the cleaned result back to the clipboard. Clipboard monitoring can be toggled in
the app UI, and text can also be cleaned manually.

Run the desktop app during development:

```bash
cd apps/desktop_tauri
npm ci
npm run tauri:dev
```

Build the desktop app:

```bash
cd apps/desktop_tauri
npm ci
npm run tauri:build
```

The Tauri configuration builds macOS `.app` and `.dmg` bundles.

## Static Web App

The web app lives in `apps/web_demo`, but it is a supported static web target. Its build script compiles the WASM
bindings first, then runs the TypeScript and Vite build.

Build the static web app:

```bash
cd apps/web_demo
npm ci
npm run build
```

The output is written to:

```text
apps/web_demo/dist/
```

The generated `dist/` directory is statically hostable. Assets use relative paths, so the same output can be served from
a domain root or from a subpath.

## Checks

Run Rust tests from the repository root:

```bash
cargo test --workspace
```

Check the desktop frontend:

```bash
cd apps/desktop_tauri
npm ci
npm run build
```

Check the Tauri backend:

```bash
cd apps/desktop_tauri/src-tauri
cargo check --locked
```

Check the static web app:

```bash
cd apps/web_demo
npm ci
npm run build
```

## Icon Assets

Icon assets are committed for reproducible desktop packaging. When branding changes, regenerate icons from the source logo:

```bash
cd apps/desktop_tauri
npm run tauri -- icon ../../CleanShare_logo.png
```

Keep this desktop icon set in `apps/desktop_tauri/src-tauri/icons`:

- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.png`
- `icon.icns`
- `icon.ico`
