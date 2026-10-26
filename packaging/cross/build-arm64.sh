#!/usr/bin/env bash
# Build release binaries for aarch64 (FuriPhone) in a rootless podman container.
#   packaging/cross/build-arm64.sh            -> target/aarch64-unknown-linux-gnu/release/
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
image=glissando-cross:forky
if ! podman image exists "$image"; then
    podman build -t "$image" -f "$root/packaging/cross/Containerfile" "$root/packaging/cross"
fi
mkdir -p "$HOME/.cargo/registry"
podman run --rm \
    -v "$root:/src:Z" \
    -v "$HOME/.cargo/registry:/root/.cargo/registry:Z" \
    "$image" \
    cargo build --locked --release --target aarch64-unknown-linux-gnu \
        -p glissando -p glissando-lang --bins
file "$root"/target/aarch64-unknown-linux-gnu/release/glissando
