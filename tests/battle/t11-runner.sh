#!/bin/bash
# T11 forensics driver: run W5 until a "core dead" finding appears or
# 10 iterations pass. Evidence of the biting run is copied next to it.
repo=/home/marcel/Projekte/chora
bt=$repo/tests/battle
base=$(grep -c '^## core dead' "$bt/findings.md")
echo "$(date -Is) runner start (core dead findings before: $base)" >> "$bt/t11-runner.log"
for i in $(seq 10); do
  run=$bt/run-t11-w5-$i.md
  echo "# T11 W5 iteration $i $(date -Is)" > "$run"
  # shellcheck disable=SC2094 # the run file is only appended to
  RUN="$run" timeout 1800 bash "$bt/w5.sh" >> "$run" 2>&1
  echo "exit=$?" >> "$run"
  now=$(grep -c '^## core dead' "$bt/findings.md")
  echo "$(date -Is) iteration $i done (core dead findings: $now)" >> "$bt/t11-runner.log"
  if [ "$now" -gt "$base" ]; then
    cp "$bt/chora-w5.log" "$bt/t11-core-dead.log"
    echo "CORE DEAD in iteration $i, log copied" >> "$bt/t11-runner.log"
    break
  fi
done
echo "$(date -Is) runner end" >> "$bt/t11-runner.log"
