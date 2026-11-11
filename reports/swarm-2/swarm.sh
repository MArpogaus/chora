#!/bin/bash
# swarm.sh: PM-owned launcher for haiku persona sessions; at most 2 run at once.
#   swarm.sh start NAME FLAVOR   blocks until a slot is free, prints PERSONA_ENV=<file>
#   swarm.sh stop NAME           stops that session (its shots and actions are kept)
set -u
S=/tmp/claude-0/-home-user/8a28ab73-2182-54dd-8448-69a0fbb58e8a/scratchpad
SRC=/tmp/swarm-src
KEEP=/home/user/wt/log/media/2026-11-12/swarm-2
RUN=$S/swarm-run
mkdir -p "$RUN" "$KEEP"
cmd=${1:-}; name=${2:-}
[ -n "$name" ] || { echo "usage: swarm.sh start|stop NAME [FLAVOR]"; exit 2; }
case $cmd in
start)
  flavor=${3:?flavor}
  out=$RUN/$name.out
  [ -e "$KEEP/$name" ] && { echo "name $name already used"; exit 2; }
  while :; do
    for s in 1 2; do
      setsid bash -c "exec 9>$RUN/slot$s.lock; flock -n 9 || exit 75
        exec env CHORA_BIN=$S/swarm-bin PERSONA_KEEP=$KEEP/$name \
          CHORA_TEST_FLAGS=--test-injected-press-is-real \
          $SRC/tests/persona/session.sh $flavor 1280x800" >"$out" 2>&1 < /dev/null &
      pid=$!
      sleep 2
      if ! kill -0 $pid 2>/dev/null; then wait $pid; [ $? -eq 75 ] && continue; echo "session failed:"; cat "$out"; exit 1; fi
      for _ in $(seq 60); do
        env=$(sed -n 's/^PERSONA_ENV=//p' "$out" | head -1)
        [ -n "$env" ] && { echo "$env" > "$RUN/$name.env"; sleep 3; echo "PERSONA_ENV=$env"; exit 0; }
        kill -0 $pid 2>/dev/null || { echo "session failed:"; cat "$out"; exit 1; }
        sleep 1
      done
      echo "session did not come up:"; cat "$out"; exit 1
    done
    sleep 20
  done ;;
stop)
  env=$(cat "$RUN/$name.env" 2>/dev/null) || { echo "no session $name"; exit 1; }
  "$SRC/tests/persona/session.sh" --stop "$env"; echo "stopped $name" ;;
*) echo "usage"; exit 2 ;;
esac
