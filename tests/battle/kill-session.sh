#!/bin/bash
# kill-session.sh: end leftover chora processes by exact process name.
pkill -f tests/battle/keep.sh; pkill -x chora; pkill -x chora-tiling; pkill -x chora-xp; pkill -x chora-probe
sleep 1
pkill -9 -x chora; pkill -9 -x chora-tiling; pkill -9 -x chora-xp; pkill -9 -x chora-probe
exit 0
