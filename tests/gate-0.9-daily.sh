#!/bin/bash
# Gate check 1 of 0.9, daily helper: run it once a day at the end of the
# working day, inside the chora session. It writes one line per day to
# the log (a second run on the same day replaces the line) and prints the
# count of clean days in a row. A day is clean when chora runs, no crash
# entry of chora is from today, the journal of chora since boot or since
# midnight shows no emergency reset, and the user answers y.
#
# Usage: tests/gate-0.9-daily.sh [LOGFILE]   default ~/gate-0.9-daily.log
set -u
log=${1:-$HOME/gate-0.9-daily.log}
today=$(date +%F)
boot=$(uptime -s)
start=$today
[ "$(date -d "$boot" +%s)" -gt "$(date -d "$today" +%s)" ] && start=$boot
running=no
systemctl --user list-units --state=active --plain --no-legend 'chora@*' 'wayland-wm@*' |
    grep -q . && running=yes
dumps=$(coredumpctl list chora --since "$today" --no-pager --no-legend 2>/dev/null | grep -c .)
resets=$(journalctl --user -u 'chora@*' -u 'wayland-wm@*' --since "$start" --no-pager 2>/dev/null |
    grep -c 'emergency reset')
read -r -p "Did you work only in chora today, and was every reset your own? [y/n] " a
verdict=BAD
[ "$running" = yes ] && [ "$dumps" = 0 ] && [ "$resets" = 0 ] && [ "$a" = y ] && verdict=CLEAN
touch "$log"
grep -v "^$today " "$log" >"$log.new"
echo "$today $verdict running=$running coredumps=$dumps resets=$resets user=$a" >>"$log.new"
mv "$log.new" "$log"
# clean days in a row, counted to the newest line
n=0
while read -r _ v _; do
    if [ "$v" = CLEAN ]; then n=$((n + 1)); else n=0; fi
done < <(sort "$log")
tail -n 1 "$log"
echo "Clean days in a row: $n of 10"
