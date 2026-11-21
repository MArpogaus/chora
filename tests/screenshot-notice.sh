#!/bin/bash
# The notice of flavors/shared/screenshot.sh in a session without a
# session bus (rw-7 of kit5): notify-send reaches no server there, so the
# script shows the notice through `choractl call notify notify.show`, the
# live API of chora-notify, and names the file. grim and choractl are
# fakes in a folder of the test; no session runs.
#
# Usage: tests/screenshot-notice.sh
set -u
repo=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d /tmp/screenshot-notice.XXXXXX)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/home" "$work/run"
cat >"$work/bin/grim" <<'SH'
#!/bin/sh
for last; do :; done
printf png >"$last"
SH
cat >"$work/bin/choractl" <<SH
#!/bin/sh
printf '%s\n' "\$*" >>"$work/calls"
SH
chmod +x "$work/bin/grim" "$work/bin/choractl"
env -u DBUS_SESSION_BUS_ADDRESS -u CHORA_NOTIFIER HOME="$work/home" \
    XDG_RUNTIME_DIR="$work/run" XDG_PICTURES_DIR="$work/home/Pictures" \
    PATH="$work/bin:$PATH" LC_ALL=C sh "$repo/flavors/shared/screenshot.sh" 2>"$work/err" ||
    { echo "FAIL: screenshot.sh failed: $(cat "$work/err")"; exit 1; }
file=$(find "$work/home" -name '*.png' | head -n 1)
[ -n "$file" ] || { echo "FAIL: no screenshot saved"; exit 1; }
grep -q "^call notify notify.show .*Screenshot saved.*$(basename "$file")" "$work/calls" 2>/dev/null ||
    { echo "FAIL: no notify.show through choractl: calls $(cat "$work/calls" 2>/dev/null), stderr $(cat "$work/err")"; exit 1; }
if grep -q CHORA_NOTIFIER "$repo/flavors/shared/screenshot.sh"; then
    echo "FAIL: screenshot.sh still reads CHORA_NOTIFIER, which no flavor sets"
    exit 1
fi
rm -f "$work/calls" "$work/sent"
cat >"$work/bin/notify-send" <<SH
#!/bin/sh
printf '%s\\n' "\$*" >>"$work/sent"
SH
chmod +x "$work/bin/notify-send"
env -u CHORA_NOTIFIER HOME="$work/home" XDG_RUNTIME_DIR="$work/run" \
    XDG_PICTURES_DIR="$work/home/Pictures" PATH="$work/bin:$PATH" LC_ALL=C \
    sh "$repo/flavors/shared/screenshot.sh" 2>"$work/err" ||
    { echo "FAIL: screenshot.sh failed: $(cat "$work/err")"; exit 1; }
grep -q '"expire_ms": 6000' "$work/calls" 2>/dev/null ||
    { echo "FAIL: a notify-send that answers is no balloon of chora-notify; calls $(cat "$work/calls" 2>/dev/null)"; exit 1; }
echo "ok: the screenshot notice goes through choractl to chora-notify, a short balloon"
