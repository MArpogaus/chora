#!/usr/bin/env bash
# plans/0.8b-driver.sh - one instance, owns plans/0.8b-state.md.
# Follows plans/0.8b.md and the skill local-pi-session. A copy of
# plans/0.8b-driver.sh with 0.8b paths, six tasks and the audit check fixed.
set -u
repo=${REPO:-/home/marcel/Projekte/chora}
sessions=${PI_SESSIONS:-/home/agent/.pi/agent/sessions/--home-marcel-Projekte-chora--}
cd "$repo" || exit 1
d=$repo/plans
state=$d/0.8b-state.md
log=$d/0.8b-log.txt
audit=/home/agent/.agents/skills/local-pi-session/tools/audit.py
kills=/home/agent/.agents/skills/local-pi-session/tools/kill-loop.py
export PATH=/home/agent/.local/bin:$PATH

run=$d/run-0.8b   # for heartbeat.sh of local-pi-session: alive and state.txt
mkdir -p "$run"
say() { echo "$(date -Is) | $*" >> "$log"; touch "$run/alive"; }
status() { awk -v t="$1" '$1==t":"{print $2}' "$state"; }
set_status() { # $1=Tn $2=status
  awk -v t="$1:" -v s="$2" '$1==t{$2=s}1' "$state" > "$state.tmp" && mv "$state.tmp" "$state"
  say "$1 -> $2"
}
stopped() { [ -f "$d/0.8b-stop" ] || [ -f "$d/0.8b-done.md" ]; }

RULES="Repo $repo, branch dev. Read plans/0.8b.md fully, and the Rules section of plans/0.8.md, and follow them always: never push, never rewrite history, never touch services/voice/, cargo only via 'micromamba run -n chora cargo', run fmt+clippy+test before every commit until all pass (tests that failed before your change for lack of a GPU may stay red), conventional commit, stage files by name, no AI attribution. Ignore target/. Tests that render need: export __EGL_VENDOR_LIBRARY_DIRS=\$HOME/.local/opt/mesa-egl/usr/share/glvnd/egl_vendor.d LD_LIBRARY_PATH=\$HOME/.local/opt/mesa-egl/usr/lib64. You may write files for your task, plans/0.8b-questions.md and plans/0.8b-review-*.md, but NEVER plans/0.8b-state.md, plans/0.8b-driver.sh or the heartbeat."

healthy() { # block until the provider answers a ping
  local out
  while :; do
    out=$(timeout 180 pi --model gufo/gufo --no-session -p "Reply with exactly PONG." 2>&1)
    case "$out" in *PONG*) return 0;; esac
    say "provider down, waiting 300s"
    sleep 300
  done
}
pi_task() { # $1=timeout $2=prompt $3=logsuffix
  healthy
  # the log is append-only: remember where this call starts
  call_start=$(stat -c %s "plans/0.8b-agent-$3.log" 2>/dev/null || echo 0)
  timeout "$1" pi --model gufo/gufo -p "$2" >> "plans/0.8b-agent-$3.log" 2>&1
  echo $? >> plans/0.8b-agent-$3.log
}
# a crash of this call only, not of an earlier call in the same log
crashed() { tail -c +"$((call_start + 1))" "plans/0.8b-agent-$1.log" 2>/dev/null | tail -c 300 | grep -q "Connection error"; }

[ -f "$state" ] || { for i in $(seq 6); do echo "T$i: todo" >> "$state"; done; }
say "driver started (pid $$)"

for n in $(seq 6); do
  stopped && break
  st=$(status "T$n")
  case "$st" in done|open*|skip) continue;; esac
  set_status "T$n" doing

  revbase=$(awk -v t="T$n:" '$1==t{print $3}' "$state")
  round=1; tries=0; nothing=0
  while [ "$round" -le 3 ]; do
    tries=$((tries+1))
    if [ "$tries" -gt 8 ]; then
      set_status "T$n" "open (8 void rounds)"
      git add "$state" && git commit -q -m "docs(plans): T$n open after 8 void rounds"
      break
    fi
    base=$(git rev-parse HEAD)
    if [ "$st" = review ] && [ "$round" = 1 ]; then
      base=$revbase
      say "T$n round 1: review only of ${base:0:7}..HEAD"
    else
      if [ "$round" = 1 ]; then
        prompt="$RULES
Do task T$n of plans/0.8b.md completely: code, a test that fails without the change, the docs line it asks for, then commit. If the task needs more than about 300 changed lines or the right behaviour is unclear, write your question into plans/0.8b-questions.md under a line 'T$n:' and commit NOTHING and change NOTHING."
        suffix="T${n}impl"
      else
        prev=$((round-1))
        prompt="$RULES
A read-only review of your work on task T$n found the items listed in plans/0.8b-review-T${n}-r${prev}.md (repo $repo). Fix every finding with new commit(s), keep the test and docs requirements of the task, or - if a finding is wrong - argue why in your final message."
        suffix="T${n}fix${round}"
      fi
      say "T$n round $round: implement starting at ${base:0:7}"
      pi_task 3600 "$prompt" "$suffix"
      if crashed "$suffix"; then
        say "T$n round $round: provider crashed, round void"
        continue
      fi

      if grep -q "^T$n:" plans/0.8b-questions.md 2>/dev/null; then
        set_status "T$n" "open (ask Marcel)"
        break
      fi
      head_now=$(git rev-parse HEAD)
      if [ "$head_now" = "$base" ] && [ "$round" = 1 ]; then
        # nothing to review and no question: retry once, then give up
        if [ "${nothing:-0}" = 1 ]; then
          set_status "T$n" "open (nothing committed)"
          git add "$state" && git commit -q -m "docs(plans): T$n open, nothing committed"
          break
        fi
        nothing=1
        say "T$n round $round: implementer committed nothing, one retry"
        continue
      fi
    fi

    # review, read-only, one report file may be written
    rv=$d/0.8b-review-T${n}-r${round}.md
    touch "$d/.revmarker-0.8b"
    prompt="$RULES
Read-only review: compare HEAD against commit ${base} with git log --oneline ${base}..HEAD and git diff ${base}..HEAD (git show per commit and reading any file is fine). Judge the work against task T$n in plans/0.8b.md and its Rules: correctness, missing test, missing docs, style. You may NOT commit, edit or create anything in the repo except the file plans/0.8b-review-T${n}-r${round}.md. In it, list every finding as a '- ' line with file:line, or write nothing. End the file with a line that is exactly CLEAN if and only if you found nothing."
    pi_task 1800 "$prompt" "T${n}rev${round}"
    if crashed "T${n}rev${round}"; then
      say "T$n round $round: provider crashed during review, round void"
      continue
    fi

    # audit: reviewer must have read the diff and changed nothing but its report
    sj=$(find "$sessions" -name "*.jsonl" -newer "$d/.revmarker-0.8b" -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2)
    audit_ok=0
    if [ -n "${sj:-}" ]; then
      python3 "$audit" "$sj" > "$d/0.8b-audit-T${n}-r${round}.txt" 2>&1
      # allowed mutations: the round's report file, /tmp, the agent home, target/
      # audit.py lists each mutation indented under the line 'mutations: N'.
      # It counts a read-only 'git stash list' as one; such a line passes.
      # ponytail: the whole line passes, so a real mutation before it in the
      # same line would pass too.
      bad=$(sed -n '/^mutations:/,$p' "$d/0.8b-audit-T${n}-r${round}.txt" | grep -E '^    ' | grep -vE "0\.8b-review-T${n}-r${round}\.md|/tmp/|/home/agent|/dev/null|target/" | grep -vE 'git stash list( \| head -[0-9]+)?$' | wc -l)
      if [ "$bad" = 0 ] && grep -q 'git \(show\|diff\|log\)' "$sj"; then audit_ok=1; fi
    fi
    [ "$audit_ok" = 1 ] || { tail -3 "$d/0.8b-audit-T${n}-r${round}.txt" >> "$log" 2>/dev/null; say "T$n round $round: audit FAILED or no diff read - verdict void"; }

    if [ "$audit_ok" = 1 ] && grep -qx CLEAN "$rv" 2>/dev/null; then
      set_status "T$n" "done $(git rev-parse --short HEAD)"
      git add "$state" && git commit -q -m "docs(plans): T$n done ($(git rev-parse --short HEAD))"
      break
    fi
    [ "$round" = 3 ] && { set_status "T$n" "open (3 rounds, no CLEAN)"; git add "$state" plans/0.8b-review-T${n}-r3.md 2>/dev/null && git commit -q -m "docs(plans): T$n open after 3 review rounds"; }
    round=$((round+1))
  done
done

# all done or open?
if ! grep -qE ": (todo|doing)" "$state"; then
  {
    echo "# 0.8 work plan - result"
    echo
    cat "$state"
    echo
    echo "Reviews: plans/0.8b-review-*.md, audits: plans/0.8b-audit-*.txt"
  } > "$d/0.8b-done.md"
  say "ALL TASKS settled, done.md written"
  # the gate of 0.8, one line per check, on fresh binaries
  micromamba run -n chora cargo build --workspace --locked >> "$log" 2>&1 || say "build for the gate failed"
  {
    echo "# Gate 0.8 after plan 0.8b ($(git rev-parse --short HEAD))"
    echo
    for g in $(seq 11); do
      out=$(CHORA_FONT_DIRS=/home/agent/.local/share/fonts/chora-test timeout 900 bash "tests/gate-0.8-$g.sh" 2>&1 | tail -1)
      echo "- check $g: $out"
    done
  } > "$d/0.8b-gate.md"
  say "gate written to plans/0.8b-gate.md"
  echo "PHASE=done" > "$run/state.txt"
fi
say "driver ending"
