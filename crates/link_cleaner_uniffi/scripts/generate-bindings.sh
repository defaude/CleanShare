#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <swift|kotlin|python> [out-dir]"
  exit 1
fi

LANGUAGE="$1"
OUT_DIR="${2:-./bindings/$LANGUAGE}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKSPACE_DIR="$(cd "$CRATE_DIR/../.." && pwd)"

cargo build -p link_cleaner_uniffi --release --manifest-path "$WORKSPACE_DIR/Cargo.toml"

case "$(uname -s)" in
  Darwin)
    LIB_EXT="dylib"
    ;;
  Linux)
    LIB_EXT="so"
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT)
    LIB_EXT="dll"
    ;;
  *)
    echo "Unsupported OS"
    exit 1
    ;;
esac

LIB_PATH="$WORKSPACE_DIR/target/release/liblink_cleaner_uniffi.$LIB_EXT"

mkdir -p "$OUT_DIR"
uniffi-bindgen generate --library "$LIB_PATH" --language "$LANGUAGE" --out-dir "$OUT_DIR"

echo "Generated $LANGUAGE bindings in $OUT_DIR"
