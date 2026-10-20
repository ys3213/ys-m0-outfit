#!/bin/sh
set -eu
cd "$(dirname "$0")"
# 工具可安裝在使用者目錄，不必改整台電腦的 PATH。
if command -v cargo >/dev/null 2>&1; then
  zoo_cargo=$(command -v cargo)
else
  export CARGO_HOME="$HOME/.local/share/ys-zoo-rust/cargo"
  export RUSTUP_HOME="$HOME/.local/share/ys-zoo-rust/rustup"
  zoo_cargo="$CARGO_HOME/bin/cargo"
fi
if command -v wasm-bindgen >/dev/null 2>&1; then
  zoo_bindgen=$(command -v wasm-bindgen)
else
  zoo_bindgen="${CARGO_HOME:-$HOME/.cargo}/bin/wasm-bindgen"
fi
"$zoo_cargo" build --manifest-path rust/Cargo.toml --locked --target wasm32-unknown-unknown --release
"$zoo_bindgen" rust/target/wasm32-unknown-unknown/release/mist_garden.wasm --target web --out-dir pkg --no-typescript
printf 'Build complete: m3-creature/pkg\n'
