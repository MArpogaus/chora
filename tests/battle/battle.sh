#!/bin/bash
# Overnight battle driver. Waits for PHASE=battle, runs W1..W7 per pass,
# tries small fixes for new findings, stops per THE END of the TODO.
repo=/home/marcel/Projekte/chora
cd "$repo" || exit 1
bt=$repo/tests/battle
state=$repo/target/review-loop/state.txt
log=$bt/log.txt
export PATH=/home/agent/.local/share/mamba/envs/chora/bin:$PATH
sdir=/home/agent/.pi/agent/sessions/--home-marcel-Projekte-chora--
stopped() { grep -q PHASE=done "$state"; }
say() { echo "$(date -Is) | $*" >> "$log"; }
nfind() { grep -c '^## ' "$bt/findings.md" 2>/dev/null; true; }

titles() { python3 - "$bt/findings.md" <<'PY'
import re, sys
print("\n".join(sorted(set(
    b.split("\n", 1)[0]
    for b in re.split(r"(?m)^## ", open(sys.argv[1]).read())[1:]))))
PY
}
pass=0 clean_passes=0
declare -A timeouts
say "driver started (pid $$)"
while ! stopped; do
  if ! grep -q PHASE=battle "$state"; then sleep 60; continue; fi
  cargo build --workspace >> "$bt/build.log" 2>&1 || say "pass build failed, using old binaries"
  pass=$((pass+1)); before=$(titles)
  say "pass $pass start (findings before: $before)"
  for w in 1 2 3 4 5 6 7 8; do
    stopped && break
    run=$bt/run-$pass-w$w.md
    echo "# pass $pass W$w $(date -Is)" > "$run"
    RUN="$run" timeout 1200 bash "$bt/w$w.sh" >> "$run" 2>&1; rc=$?
    echo "exit=$rc" >> "$run"
    if [ $rc = 124 ]; then
      timeouts[$w]=$(( ${timeouts[$w]:-0} + 1 ))
      say "W$w timed out (${timeouts[$w]} in a row)"
      [ ${timeouts[$w]} -ge 3 ] && { say "THE END: W$w timed out 3 passes in a row"; break 2; }
    else
      timeouts[$w]=0
    fi
  done
  after=$(titles); new=$(comm -13 <(echo "$before") <(echo "$after") | grep -c .; true)
  say "pass $pass done: $new new findings (before=$before after=$after)"
  if [ "$new" = 0 ]; then
    clean_passes=$((clean_passes+1))
    [ "$clean_passes" -ge 2 ] && { say "THE END: two clean passes"; break; }
  else
    clean_passes=0
    # one sub-agent per open finding, small fixes only (spec's fix rule)
    while read -r -d '' title; do
      stopped && break
      before_s=$(ls "$sdir" | sort)
      timeout 1200 pi --model gufo/gufo -p "In /home/marcel/Projekte/chora: the battle finding '$title' in tests/battle/findings.md. Fix it ONLY if the fix is small: at most 1 file and 20 lines, and you can write a test that fails without the fix. Before committing run: cargo fmt, cargo clippy --workspace -- -D warnings, cargo test --workspace. Conventional commit, stage files by name, never push, never touch services/voice/. Then set the finding's status line to 'fixed in <commit>'. If it is not small or you cannot write the test, change nothing and set the status line to 'open (not small)'." >> "$bt/fix.log" 2>&1 < /dev/null
      say "fix attempt for: $title (pi exit $?)"
    done < <(python3 - "$bt/findings.md" <<'PY'
import re, sys
blocks = re.split(r"(?m)^## ", open(sys.argv[1]).read())[1:]
open_titles = list(dict.fromkeys(b.split("\n", 1)[0] for b in blocks if re.search(r"^status: open", b, re.M)))
sys.stdout.write("\0".join(open_titles[:4]) + "\0")
PY
)
  fi
done
say "driver ending (pass $pass, clean passes $clean_passes)"
# THE END: stop the heartbeat, write done.md, set PHASE=done
pkill -f chora-review-heartbeat 2>/dev/null
pkill -f 'sleep 1200' 2>/dev/null
{
  echo "# Review loop and battle test: summary"
  echo
  echo "- Review rounds: $(ls $repo/target/review-loop/round-*.md 2>/dev/null | grep -vc bad) (reports: target/review-loop/)"
  echo "- Battle passes: $pass, clean passes at the end: $clean_passes"
  echo "- Findings: $(nfind); open:"
  grep -B6 'status: open' "$bt/findings.md" 2>/dev/null | grep '^## ' | sed 's/^## /  - /'
  echo "- Log: tests/battle/log.txt, run reports: tests/battle/run-*.md"
  echo "- Friction and API points: tests/battle/FRICTION.md"
} > "$bt/done.md"
echo "PHASE=done ROUND=$pass CLEAN=$clean_passes" > "$state"
say "done.md written, PHASE=done"
