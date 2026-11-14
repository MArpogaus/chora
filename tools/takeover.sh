#!/usr/bin/env bash
# takeover.sh: readies this container for a new PM session after the old one died. Idempotent.
# Usage: bash takeover.sh   (from the log branch, or the stable copy /home/user/.claude/pm/takeover.sh)
# Runbook: README.md "PM down". It changes no code branch with local changes and stops on a dirty log.
set -u
repo=/home/user/chora
log=/home/user/wt/log
code_branch=claude/autonomous-dev-session-3w-i33pg7
die() { echo "takeover: $*" >&2; exit 1; }
[ -d "$repo/.git" ] || die "$repo is missing; clone MArpogaus/chora there first (add_repo)"
git -C "$repo" config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'
git -C "$repo" fetch -q origin || die "fetch failed"
if [ -d "$log" ]; then
    [ -z "$(git -C "$log" status --porcelain 2>/dev/null)" ] || die "$log has local changes; commit or save them first"
    [ "$(git -C "$log" symbolic-ref --short -q HEAD)" = claude/session-log ] || git -C "$log" checkout -q claude/session-log
    git -C "$log" pull -q --ff-only origin claude/session-log || die "log pull failed"
else
    git -C "$repo" worktree prune
    mkdir -p "$(dirname "$log")"
    git -C "$repo" worktree add -q -B claude/session-log "$log" origin/claude/session-log || die "worktree add failed"
fi
if [ -z "$(git -C "$repo" status --porcelain)" ] && [ "$(git -C "$repo" symbolic-ref --short -q HEAD)" != "$code_branch" ]; then
    git -C "$repo" checkout -q -B "$code_branch" "origin/$code_branch"
fi
bash "$log/setup.sh" || echo "takeover: setup.sh reported MISSING; fix it before building"
bash "$log/tools/install_hooks.sh" || die "install_hooks failed"
bash "$log/tests/test_hooks.sh" >/dev/null || die "hook test failed"
cat <<EOF
takeover: container ready. Now, in the new session (tools/triggers.md has the prompts):
1. get_session without an id gives your id; put it in the pm: line of state.md; commit and push.
2. list_triggers: disable PM watchdog and PM self-check bound to the old id; create both bound to
   this session; update the id in the PM heartbeat prompt.
3. send_message to every running worker in the Lanes table: the new PM id.
4. Archive the old PM session (owner ok for dead PM sessions, 11-14 16:47 UTC).
5. Read rules.md, state.md, decisions-now.md, the tails of answers.md and friction.md.
6. Recover what only the old transcript holds: list_events of the old id since its last log commit.
EOF
