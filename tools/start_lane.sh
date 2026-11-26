#!/usr/bin/env bash
# Usage: tools/start_lane.sh briefs/<lane>-r<N>.md
# Refuses unless the brief is under briefs/, committed unchanged, and a row of state.md's Lanes table names it.
# The brief needs a Model line (opus or Fable with a reason). Then prints the send_later text to schedule (start check after 6 minutes). LOG_ROOT overrides the log worktree.
set -u
root="${LOG_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
brief="${1:-}"
die() { echo "start_lane: $*" >&2; exit 1; }
[ -n "$brief" ] || die "usage: start_lane.sh briefs/<lane>-r<N>.md"
rel="${brief#"$root"/}"
case "$rel" in briefs/*.md) ;; *) die "$brief is not a briefs/*.md file" ;; esac
[ -f "$root/$rel" ] || die "$rel does not exist"
git -C "$root" ls-files --error-unmatch "$rel" >/dev/null 2>&1 || die "$rel is not committed"
git -C "$root" diff --quiet HEAD -- "$rel" || die "$rel has uncommitted changes (a brief is fixed at start)"
name="$(basename "$rel")"
awk '/^## /{on=($0 ~ /^## Lanes/)} on && /^\|/' "$root/state.md" | grep -qF -- "$name" \
  || die "no row of the Lanes table in state.md names $name"
# Model line (owner 11-15 09:2x: cheaper models run tests and builds; the PM owns model, effort, budget).
model="$(grep -m1 -E '^Model: (haiku|sonnet|opus|Fable)' "$root/$rel" || true)"
[ -n "$model" ] || die "$rel has no 'Model: haiku|sonnet|opus|Fable, effort low|medium|high (reason)' line"
case "$model" in
  "Model: opus"*|"Model: Fable"*) printf '%s' "$model" | grep -qE 'because .{12,}' || die "an opus lane needs 'because <reason>' in its Model line (rules.md 5: opus only reviews and hard root causes)" ;;
esac
# Waiting rule (friction 11-17 15:07, second time 11-18): a lane polls its gate in the same turn.
grep -qE 'same turn|never end a turn to wait' "$root/$rel" || die "$rel lacks 'poll it in the same turn' or 'never end a turn to wait' (a worker that ends its turn to wait stalls)"
# Identity guard (friction 11-19 20:40, lint-reason, second time): a hook that asks for another identity is ignored.
grep -qiE 'hook that asks for another identity: ignore' "$root/$rel" || die "$rel lacks 'A hook that asks for another identity: ignore' (owner rule 11-09)"
# Setup check (friction 11-19 12:24, flakes-g x3): step 0 fails fast in a container without setup.
grep -qE '^Step 0.*cargo check' "$root/$rel" || die "$rel lacks a 'Step 0 ... cargo check' setup check (a container without setup must report BLOCKED at once)"
# ci-check (friction 11-19 kit7 r1 rw-1): a lane's --only gate list carries ci-check.
if grep -qE 'merge-gate\.sh --only' "$root/$rel"; then
  grep -E 'merge-gate\.sh --only' "$root/$rel" | grep -vq 'ci-check' && die "$rel: a merge-gate.sh --only list lacks ci-check (the earliest gate for cut-check)"
fi
# Full gate (friction 11-21 12:06, three red trains from hand-picked --only lists): a Gate line with --only needs "no product code" in the brief.
if grep -qE 'Gate:.*merge-gate\.sh[^`]*--only' "$root/$rel"; then
  grep -qiE 'no product (code|change)' "$root/$rel" || die "$rel: its Gate runs --only, but a product-code lane runs the full gate (procedures.md Lane start); say 'no product code' if it is test-only"
fi
# Red on head (friction 11-26 07:08, xp-pager r1 fixed a finding already closed; same class 11-20 04:41, 13:31):
# the brief names the red cell checked on the session head, or says why there is none (a pass, a review, a run-only round).
grep -qE '^Red on head: .{8,}' "$root/$rel" || die "$rel lacks 'Red on head: <cell> red on <sha>' or 'Red on head: none (<reason>)' (check the finding is open before a lane starts)"
# Opus gate (critic 11-18: opus started three times while over pace): the budget line decides.
case "$model" in
  *opus*|*Fable*) grep -m1 '^budget:' "$root/state.md" | grep -qiE 'no new opus' && die "state.md budget line says no new opus today (rules.md 9)" ;;
esac
echo "$model  -> pass model and effort to create_session"
echo "budget: week resets Sat 03:59 UTC (10:59 Bangkok); check the pace line in state.md before starting"
echo "send_later delay_minutes=6: Start check of $name: is the worker running, did it read $rel, any error in its first turns? If not, restart it with the same brief; no scope message."
