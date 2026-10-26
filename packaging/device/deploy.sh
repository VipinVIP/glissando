#!/usr/bin/env bash
# Copy the arm64 build and data to the phone (user prefix, no root):
#   packaging/device/deploy.sh [ssh-host]      (default host: furios)
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
host="${1:-furios}"
bin="$root/target/aarch64-unknown-linux-gnu/release"
[ -x "$bin/glissando" ] || { echo "build first: packaging/cross/build-arm64.sh" >&2; exit 1; }
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/bin" "$stage/share/glissando"
install -m755 "$bin/glissando" "$stage/bin/"
install -m755 "$root"/packaging/device/glissando-{trial,launch,session} "$stage/bin/"
cp -r "$root/data/layouts" "$root/data/emoji" "$stage/share/glissando/"
mkdir -p "$stage/share/glissando/dict"
cp "$root"/data/dict/*.gld "$stage/share/glissando/dict/"
ssh "$host" 'mkdir -p ~/.local/opt/glissando'
tar -C "$stage" -czf - . | ssh "$host" 'tar -C ~/.local/opt/glissando -xzf - && ls -la ~/.local/opt/glissando/bin'
echo "installed to ~/.local/opt/glissando on $host"
