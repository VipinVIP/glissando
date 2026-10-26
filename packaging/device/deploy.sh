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
install -m755 "$bin/glissando" "$bin/glissando-settings" "$stage/bin/"
install -m755 "$root"/packaging/device/glissando-{trial,launch,session} "$stage/bin/"
cp -r "$root/data/layouts" "$root/data/emoji" "$stage/share/glissando/"
mkdir -p "$stage/share/glissando/dict"
cp "$root"/data/dict/*.gld "$stage/share/glissando/dict/"
install -Dm644 "$root/data/icons/io.github.x_vlad_x.Glissando.svg" "$stage/share/icons/hicolor/scalable/apps/io.github.x_vlad_x.Glissando.svg"
install -Dm644 "$root/packaging/desktop/io.github.x_vlad_x.Glissando.Settings.desktop" "$stage/share/applications/io.github.x_vlad_x.Glissando.Settings.desktop"
ssh "$host" 'mkdir -p ~/.local/opt/glissando'
tar -C "$stage" -czf - . | ssh "$host" 'tar -C ~/.local/opt/glissando -xzf - && ls -la ~/.local/opt/glissando/bin'
# Make the settings app visible in the launcher (user scope only).
ssh "$host" 'set -e
o=$HOME/.local/opt/glissando
mkdir -p ~/.local/share/applications ~/.local/share/icons/hicolor/scalable/apps
sed "s|^Exec=.*|Exec=$o/bin/glissando-settings|" "$o/share/applications/io.github.x_vlad_x.Glissando.Settings.desktop" > ~/.local/share/applications/io.github.x_vlad_x.Glissando.Settings.desktop
cp "$o/share/icons/hicolor/scalable/apps/io.github.x_vlad_x.Glissando.svg" ~/.local/share/icons/hicolor/scalable/apps/'
echo "installed to ~/.local/opt/glissando on $host"
