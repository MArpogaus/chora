#!/usr/bin/env bash
# Settings docs drift check: every top-level key of the default settings of a
# component must appear, in backticks, in the section of that component in
# docs/configuration.md (docs/voice.md for the voice agent).
# Usage: tests/settings-docs.sh [BIN_DIR]   (default: target/debug)
set -euo pipefail

bin=${1:-target/debug}
cd "$(dirname "$0")/.."
status=0

# check NAME DEFAULTS_FILE DOC SECTION_START_REGEX
check() {
  local name=$1 defaults=$2 doc=$3 start=$4 section key
  # from the heading to the next heading of the same level
  section=$(awk -v re="$start" '/^## / { on = ($0 ~ re) } on' "$doc")
  [ -n "$section" ] || { echo "FAIL: $name: no section $start in $doc"; status=1; return; }
  for key in $(grep -oE '^[A-Za-z_][A-Za-z0-9_]*:' "$defaults" | tr -d :); do
    if ! grep -qF "\`$key\`" <<<"$section"; then
      echo "FAIL: $name: key '$key' is not in $doc"
      status=1
    fi
  done
}

tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT

for c in tiling mc construct; do
  "$bin/chora-$c" --print-config >"$tmp"
  case $c in
    tiling) start='^## chora-tiling' ;;
    mc) start='^## chora-mc' ;;
    construct) start='^## chora-construct' ;;
  esac
  check "chora-$c" "$tmp" docs/configuration.md "$start"
done

# chora-xp has no --print-config: its default is the file in the source tree.
check chora-xp plugins/chora-xp/xp.yaml docs/configuration.md '^## Flavors'

"$bin/chora-voice" --print-config >"$tmp"
check chora-voice "$tmp" docs/voice.md '^## 9[.] Settings'

[ "$status" -eq 0 ] && echo "ok: settings docs match the defaults"
exit "$status"
