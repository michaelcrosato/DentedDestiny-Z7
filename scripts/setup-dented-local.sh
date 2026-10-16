#!/usr/bin/env bash
# Development setup without admin access. Dependencies stay ignored inside this checkout.
set -euo pipefail
cd "$(dirname "$0")/.."
rustup target add wasm32-unknown-unknown
if ! pkg-config --exists alsa libudev; then
  mkdir -p .cache/native/debs
  (cd .cache/native/debs && apt-get download libasound2-dev libudev-dev && for f in *.deb; do dpkg-deb -x "$f" ..; done)
  python3 - <<'PY'
from pathlib import Path
root=Path('.cache/native').resolve()
for pc in (root/'usr/lib/x86_64-linux-gnu/pkgconfig').glob('*.pc'):
    pc.write_text(pc.read_text().replace('prefix=/usr','prefix='+str(root/'usr')))
for name in ['libasound.so.2','libudev.so.1']:
    src=Path('/usr/lib/x86_64-linux-gnu')/name
    dest=root/'usr/lib/x86_64-linux-gnu'/name
    if src.exists() and not dest.exists(): dest.symlink_to(src)
    linker=root/'usr/lib/x86_64-linux-gnu'/name.split('.so.')[0]
    linker=linker.with_name(linker.name+'.so')
    if linker.is_symlink(): linker.unlink()
    if not linker.exists(): linker.symlink_to(src)
PY
fi
printf '%s\n' 'Ready. Run scripts/build-dented.sh.'
