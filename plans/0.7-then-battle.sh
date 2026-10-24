#!/bin/bash
# Wait for the 0.7 plan to settle, then run battle rounds (W1..W8).
cd /home/marcel/Projekte/chora || exit 1
while [ ! -f plans/0.7-done.md ]; do sleep 60; done
sleep 120  # let cargo caches and any pi worker settle
bash tests/battle/kill-session.sh
printf "PHASE=battle\nROUND=1\nCLEAN=0\n" > target/review-loop/state.txt
echo "$(date -Is) | 0.7 settled, battle started" >> tests/battle/log.txt
exec bash tests/battle/battle.sh
