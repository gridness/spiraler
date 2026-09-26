#!/usr/bin/env bash
set -euo pipefail

dmg=${1:?Usage: verify-macos-dmg.sh DMG VERSION}
version=${2:?Usage: verify-macos-dmg.sh DMG VERSION}
mountpoint=$(mktemp -d "${TMPDIR:-/tmp}/spiraler-dmg.XXXXXX")

attached=false
cleanup() {
  if "$attached"; then
    hdiutil detach "$mountpoint" -quiet
  fi
  rmdir "$mountpoint"
}
trap cleanup EXIT

hdiutil verify "$dmg"
hdiutil attach -readonly -nobrowse -mountpoint "$mountpoint" "$dmg"
attached=true
app="$mountpoint/Spiraler.app"
# Verify the app inside the distributed image, including its sealed resources.
codesign --verify --deep --strict --verbose=2 "$app"
lipo "$app/Contents/MacOS/spiraler" -verify_arch arm64
test "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  "$app/Contents/Info.plist")" = "$version"
