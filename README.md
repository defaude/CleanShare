# clean-share

Cross-Platform MVP zum Bereinigen von Share-Texten: Nur Tracking-Parameter aus URLs werden entfernt, restlicher Text bleibt unverändert.

## Architektur

- `crates/link_cleaner_core`: Pure Rust Rule-Engine (keine OS-/UI-Abhängigkeiten)
- `crates/link_cleaner_wasm`: `wasm-bindgen` Wrapper, exportiert `clean_text(input: string): string`
- `crates/link_cleaner_uniffi`: UniFFI-Scaffold für spätere Swift/Kotlin Bindings
- `apps/desktop_tauri`: Tauri v2 Desktop MVP (Live-Cleaning + Copy)
- `apps/web_demo`: Minimale Web-Demo (Vite + TypeScript) mit WASM-Integration

## Prerequisites

- Git
- Node.js LTS + npm
- Rust via `rustup` (inkl. `rustfmt` und `clippy`)
- `wasm-pack`
- Rust target `wasm32-unknown-unknown`
- Tauri prerequisites je OS:
1. macOS: Xcode Command Line Tools
2. Windows: Visual Studio Build Tools (Desktop C++ workload)
3. Linux: WebView/GTK-Systempakete je Distribution

Offizielle Tauri Prerequisites: [https://v2.tauri.app/start/prerequisites/](https://v2.tauri.app/start/prerequisites/)

## Setup

```bash
rustup component add rustfmt clippy
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked
```

## Rust Checks und Tests

Im Repo-Root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## WASM Build und Web Demo

WASM-Paket direkt aus `link_cleaner_wasm` bauen:

```bash
wasm-pack build crates/link_cleaner_wasm --target web --out-dir pkg --out-name link_cleaner_wasm
```

Web-Demo starten:

```bash
cd apps/web_demo
npm install
npm run dev
```

Hinweis: `npm run dev` baut zuerst das WASM-Paket aus `../../crates/link_cleaner_wasm` nach `apps/web_demo/pkg`.

## Desktop App (Tauri v2)

```bash
cd apps/desktop_tauri
npm install
npm run tauri:dev
```

Das Tauri-Backend ruft `link_cleaner_core::clean_text` über den Command `clean_text` auf.

Alternative (bei installierter Rust Tauri CLI):

```bash
cd apps/desktop_tauri/src-tauri
cargo tauri dev
```

## UniFFI Scaffold

Bindings später generieren (nach installiertem `uniffi-bindgen`):

```bash
cargo install uniffi_bindgen
cd crates/link_cleaner_uniffi
./scripts/generate-bindings.sh swift
./scripts/generate-bindings.sh kotlin
```

## Cleaning Rules v0

Tracking-Parameter werden konservativ entfernt:

- Alle Query-Parameter mit Prefix `utm_`
- `gclid`, `fbclid`, `mc_cid`, `mc_eid`, `ref`, `ref_src`, `ref_url`, `igshid`, `igsh`, `si`
- Amazon-spezifisch (v0): `tag`, `linkCode` entfernen, Pfadsegment ab `/ref=` abschneiden

Bewusst beibehalten:

- Semantische Parameter wie `t` (Twitter/X Timestamp)
- Fragmente `#...`

## Golden Tests (Core)

Die 12 angeforderten Golden-Cases liegen in `crates/link_cleaner_core/src/lib.rs` und validieren exakt Input -> Output für YouTube, UTM, gclid/fbclid, Mailchimp, Amazon, Instagram, TikTok, X, Google Maps, Multiple URLs/Punctuation, No URL und Fragment-Erhalt.

## Known Limitations

- URL-Erkennung basiert auf `http://` / `https://`; ohne Scheme werden Links in v0 nicht erkannt.
- Trailing-Punctuation-Heuristik ist konservativ und deckt Standardfälle ab.
- Mobile Apps sind noch nicht enthalten; UniFFI ist als Scaffold vorbereitet.

## Next Steps

- Domänenspezifische Regeln versionieren (Feature Flags/Rule Sets)
- `clean_text_with_report` in Web/Desktop UI anzeigen
- Mobile Host-Apps (iOS/Android) mit UniFFI-Bindings anbinden
