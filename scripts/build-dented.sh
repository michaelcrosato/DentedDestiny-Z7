#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Prefer installed system headers; a local deb-extraction fallback also works on unprivileged WSL.
if [[ -d .cache/native/usr/lib/x86_64-linux-gnu/pkgconfig ]]; then
  export PKG_CONFIG_PATH="$PWD/.cache/native/usr/lib/x86_64-linux-gnu/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
fi
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
cargo build --locked -p pav_app -p pav_tools "$@"
