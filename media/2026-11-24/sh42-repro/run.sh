#!/bin/bash
# usage: run.sh LOOK(xp|mc) MODE(join|nojoin|nowin_join|switch) N CAPDIR
cd /home/user/chora
look=$1 mode=$2 n=$3 cap=$4
export CHORA_TEST_FLAGS=--test-injected-press-is-real
size=1280x800; [ $look = mc ] && size=1280x720
out=/tmp/sh42/s.$look.$mode.$n.out
tests/persona/session.sh $look $size >$out 2>&1 &
for i in $(seq 50); do grep -q PERSONA_ENV $out && break; sleep 0.3; done
envf=$(sed -n 's/^PERSONA_ENV=//p' $out)
set -a; . $envf; set +a; export PERSONA_ENV=$envf
export WAYLAND_DISPLAY=$(ls $XDG_RUNTIME_DIR | grep -m1 wayland-)
act="python3 -I tests/persona/act.py"
F=/tmp/sh42/in.$look.$mode.$n.txt; : >$F
holders(){ echo "core=$(choractl query kind:surface | jq -c '[.[]|select(.focused==true)|.id]') xp=$(choractl state xp 2>/dev/null|jq -c .focus.holder) mc=$(choractl state mc 2>/dev/null|jq -c .focus.holder) tiling=$(choractl state tiling 2>/dev/null|jq -c .focus.holder)"; }
sleep 5
if [ $mode != nowin_join ] && [ $mode != switch_nowin ]; then
  if [ $look = xp ]; then
    choractl run foot -a r2 sh -c "stty raw -echo; cat > $F" >/dev/null
  else
    choractl run foot --app-id=mcstray sh -c "stty raw -echo; cat > $F" >/dev/null
  fi
  sleep 3
  [ $look = mc ] && { $act key Return >/dev/null; sleep 1; }
fi
sleep 1; pre=$(holders)
$act type "echo a$n" >/dev/null; sleep 1
r_pre=$(cat -v $F)
case $mode in
 join|nowin_join)
  choractl run env CHORA_FLAVOR_DIR=$PWD/flavors/tiling setsid dbus-run-session -- chora-tiling --no-companions >/dev/null ;;
 switch|switch_nowin)
  choractl run setsid dbus-run-session -- chora-mc --manage-windows >/dev/null ;;
esac
sleep 5
post=$(holders)
$act type "echo r$n" >/dev/null; sleep 1
r_post=$(cat -v $F)
lost=NO; [[ "$r_post" == *"r$n"* ]] && lost=NO || lost=YES
[ -n "$cap" ] && { $act look --max-side 640 >/dev/null; cp $(ls -t $PERSONA_SHOTS/*.png|head -1) $cap/$look-$mode-$n.png; }
rec=NA
if [ $lost = YES ]; then
  fsurf=$(choractl query kind:surface | jq -r '.[0].rect // empty')
  if [ $look = mc ]; then
    $act key Return >/dev/null; sleep 1; $act type "echo k$n" >/dev/null; sleep 1
    [[ "$(cat -v $F)" == *"k$n"* ]] && rec=Return || {
      $act click 640 360 >/dev/null; sleep 1; $act type "echo c$n" >/dev/null; sleep 1
      [[ "$(cat -v $F)" == *"c$n"* ]] && rec=click || rec=none; }
  else
    $act click 400 280 >/dev/null; sleep 1; $act type "echo c$n" >/dev/null; sleep 1
    [[ "$(cat -v $F)" == *"c$n"* ]] && rec=click || rec=none
  fi
fi
echo "$look $mode #$n | pre: $pre | post: $post | typed-after-switch-reaches: $([ $lost = YES ] && echo no || echo yes) | recovered: $rec | file=$(cat -v $F)"
tests/persona/session.sh --stop $envf >/dev/null 2>&1; wait
sleep 1
