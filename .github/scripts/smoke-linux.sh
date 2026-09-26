#!/usr/bin/env bash
set -euo pipefail

executable=${1:?Usage: smoke-linux.sh EXECUTABLE}
state=$(mktemp -d "${TMPDIR:-/tmp}/spiraler-smoke.XXXXXX")
trap 'rm -rf "$state"' EXIT
status=0
# Isolate first-launch state and exercise the actual GUI and bundled libraries.
XDG_DATA_HOME="$state/data" XDG_CONFIG_HOME="$state/config" \
  XDG_CACHE_HOME="$state/cache" XDG_RUNTIME_DIR="$state" \
  WEBKIT_DISABLE_DMABUF_RENDERER=1 \
  xvfb-run -a timeout 15s "$executable" > "$state/startup.log" 2>&1 || status=$?
if test "$status" -ne 124; then
  cat "$state/startup.log"
  echo "Spiraler exited during the GUI startup check with status $status" >&2
  if command -v gdb >/dev/null; then
    XDG_DATA_HOME="$state/data" XDG_CONFIG_HOME="$state/config" \
      XDG_CACHE_HOME="$state/cache" XDG_RUNTIME_DIR="$state" \
      WEBKIT_DISABLE_DMABUF_RENDERER=1 \
      xvfb-run -a timeout 30s gdb --batch -ex 'set detach-on-fork off' \
        -ex 'set schedule-multiple on' \
        -ex run -ex 'thread apply all bt' --args bash "$executable" || true
  fi
  exit 1
fi
echo "Spiraler stayed running through the GUI startup check."
