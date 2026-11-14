#!/bin/sh
# pm-hook.sh MODE: runs pm_context.py next to it for a Claude Code hook.
# It never blocks: every path ends with exit 0, even when the log worktree is gone or on the
# wrong branch (friction 44). The card mode warns loudly instead.
here=$(cd "$(dirname "$0")" && pwd)
log=${PM_LOG:-/home/user/wt/log}
branch=$(git -C "$log" symbolic-ref --short -q HEAD 2>/dev/null)
if [ "${1:-}" = card ] && [ "$branch" != claude/session-log ]; then
    echo "PM WARNING: $log is not on claude/session-log (HEAD: ${branch:-missing or detached})."
    echo "Repair first: bash $here/takeover.sh (it recreates the worktree and the hooks)."
fi
timeout 15 python3 "$here/pm_context.py" "$@" 2>/dev/null
exit 0
