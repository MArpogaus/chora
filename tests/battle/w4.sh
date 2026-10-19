#!/bin/bash
# W4 hotplug: add and remove outputs 20 times while windows are open.
WF=w4; . "$(dirname "$0")/lib.sh"
session_start "1280x800,1280x800+1280+0" "$bin/chora-tiling" || exit 1
for i in $(seq 6); do open $((i*35)) $((i*35)) 200; done
health "6 windows up"

for i in $(seq 20); do
  spec=$([ $((i % 3)) = 0 ] && echo "640x480+2560+0" || echo "1024x768@1.25+2560+0")
  name=$(admin add-output "$spec" | python3 -c 'import json,sys
try: print(json.load(sys.stdin)["name"])
except Exception: print("")' 2>/dev/null)
  if [ -z "$name" ]; then finding "W4 add-output $spec failed (round $i)" wrong "choractl add-output $spec"; continue; fi
  sleep 0.4
  admin remove-output "$name" >/dev/null 2>&1 || finding "W4 remove-output $name failed (round $i)" wrong "remove $name"
  health "hotplug round $i ($name)"
done
[ "$(count_windows)" -ge 5 ] || finding "W4: windows vanished during hotplug (only $(count_windows))" wrong "6 windows, 20 hotplug cycles"
ctl capture "$repo/tests/battle/w4-final.png" --output HEADLESS-1 >> "$RUN" 2>&1
session_stop
