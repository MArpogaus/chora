#!/usr/bin/env bash
# Usage: tools/start_lane.sh briefs/<lane>-r<N>.md
# Refuses unless the brief is under briefs/, committed unchanged, and a row of state.md's Lanes table names it.
# The brief needs a Model line (opus with a reason). Then prints the send_later text to schedule (start check after 6 minutes). LOG_ROOT overrides the log worktree.
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
model="$(grep -m1 -E '^Model: (haiku|sonnet|opus)' "$root/$rel" || true)"
[ -n "$model" ] || die "$rel has no 'Model: haiku|sonnet|opus, effort low|medium|high (reason)' line"
case "$model" in
  *opus*) printf '%s' "$model" | grep -qE 'because .{12,}' || die "an opus lane needs 'because <reason>' in its Model line (rules.md 5: opus only reviews and hard root causes)" ;;
esac
echo "$model  -> pass model and effort to create_session"
echo "budget: week resets Sat 03:59 UTC (10:59 Bangkok); check the pace line in state.md before starting"
echo "send_later delay_minutes=6: Start check of $name: is the worker running, did it read $rel, any error in its first turns? If not, restart it with the same brief; no scope message."
