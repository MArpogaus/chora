#!/bin/bash
# usage: drv.sh start NAME | begin NAME | end NAME TASK... | stop NAME
repo=/home/user/chora; export ROUND_DIR=/tmp/persona-round
cmd=$1 name=$2; shift 2
case $cmd in
start)
  out=$(bash $repo/tests/persona/round.sh start $name construct 1280x800 2>&1)
  f=$(echo "$out" | sed -n 's/^PERSONA_ENV=//p'); echo "$f" >/tmp/claude-0/r14/$name.env
  set -a; . $f; set +a
  echo '[{"seq":1,"role":"agent","text":"Hi, I am atlas, your helper."}]' >"$PERSONA_DIR/voice/entries.json"
  echo "PERSONA_ENV=$f" ;;
begin) export PERSONA_ENV=$(cat /tmp/claude-0/r14/$name.env); bash $repo/tests/persona/round.sh begin $name ;;
end) export PERSONA_ENV=$(cat /tmp/claude-0/r14/$name.env); set -a; . $PERSONA_ENV; set +a
  mkdir -p /tmp/claude-0/r14/ends; python3 -I $repo/tests/persona/check.py end "$@" >/tmp/claude-0/r14/ends/$name-${1//:/_}.txt 2>&1; cat /tmp/claude-0/r14/ends/$name-${1//:/_}.txt | grep -v "^breach I6" | head -8 ;;
stop) bash $repo/tests/persona/round.sh stop $name ;;
esac
