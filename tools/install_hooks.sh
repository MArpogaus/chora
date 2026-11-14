#!/usr/bin/env bash
# install_hooks.sh: installs the fail-open PM hooks and the log-worktree guard. Idempotent.
# The hooks run a stable copy in /home/user/.claude/pm, outside every git worktree (friction 44).
# Usage: bash tools/install_hooks.sh   (PM_DIR and PM_SETTINGS override the targets, for tests)
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
pm_dir=${PM_DIR:-/home/user/.claude/pm}
settings=${PM_SETTINGS:-/home/user/.claude/settings.local.json}
mkdir -p "$pm_dir"
install -m 644 "$root/tools/pm_context.py" "$pm_dir/pm_context.py"
install -m 755 "$root/tools/hooks/pm-hook.sh" "$pm_dir/pm-hook.sh"
install -m 755 "$root/tools/takeover.sh" "$pm_dir/takeover.sh"
python3 "$root/tools/hooks/write_settings.py" "$settings" "$pm_dir"
hooks_dir=$(git -C "$root" rev-parse --path-format=absolute --git-common-dir)/hooks
target="$hooks_dir/post-checkout"
if [ -e "$target" ] && ! cmp -s "$target" "$root/tools/hooks/post-checkout"; then
    grep -q "keeps the PM's log worktree" "$target" || { echo "install_hooks: foreign $target; merge by hand" >&2; exit 1; }
fi
install -m 755 "$root/tools/hooks/post-checkout" "$target"
echo "install_hooks: hooks in $settings, scripts in $pm_dir, guard in $target"
