#!/usr/bin/env bash
# Build whisper.cpp for the development machine (used by the voice e2e test).
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
src="$root/.devtools/whisper.cpp"
[ -d "$src" ] || git clone -q --depth 1 --branch "${WHISPER_VERSION:-v1.7.6}" https://github.com/ggml-org/whisper.cpp "$src"
cmake -S "$src" -B "$root/build/whisper-host" -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF \
  -DGGML_OPENMP=OFF -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_SERVER=OFF >/dev/null
cmake --build "$root/build/whisper-host" -j"$(nproc)" --target whisper-cli >/dev/null
echo "$root/build/whisper-host/bin/whisper-cli"
