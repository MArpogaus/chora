#!/bin/bash
# The core's last words: a faulting signal leaves a backtrace in the log.
set -u
repo=$(cd "$(dirname "$0")/.." && pwd)
bin=${BIN:-$repo/target/debug}
export CHORA_LOG=$(mktemp -t last-words.XXXXXX)
"$repo/tests/headless-session.sh" 800x600 > /tmp/last-words.env 2>&1 &
for _ in $(seq 60); do grep -q CHORA_SOCKET /tmp/last-words.env && break; sleep 0.5; done
core=$(pgrep -x chora | tail -1)
[ -n "$core" ] || { echo "FAIL: no core"; exit 1; }
kill -SEGV "$core"
sleep 2
grep -q "chora: SIGSEGV at address" "$CHORA_LOG" && grep -q "install_last_words" "$CHORA_LOG" &&
  { echo "ok: the core logged its backtrace"; exit 0; }
echo "FAIL: no last words in $CHORA_LOG"; tail -3 "$CHORA_LOG"; exit 1
