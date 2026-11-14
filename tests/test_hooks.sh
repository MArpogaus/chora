#!/usr/bin/env bash
# test_hooks.sh: the PM hooks never block and the log worktree guard switches back (friction 44).
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail=0
check() { if eval "$2"; then echo "ok   $1"; else echo "FAIL $1"; fail=1; fi; }

printf '{"hooks": {"UserPromptSubmit": []}, "env": {"KEEP": "1"}}' >"$tmp/settings.json"
python3 "$root/tools/hooks/write_settings.py" "$tmp/settings.json" "$tmp/pm"
mkdir -p "$tmp/pm"
cp "$root/tools/pm_context.py" "$root/tools/hooks/pm-hook.sh" "$tmp/pm/"
commands=$(python3 -c 'import json,sys
d=json.load(open(sys.argv[1]))["hooks"]
for ev in d.values():
    for m in ev:
        for h in m["hooks"]: print(h["command"])' "$tmp/settings.json")
check "other settings keys survive" "grep -q KEEP '$tmp/settings.json'"
check "no UserPromptSubmit hook" "! grep -q UserPromptSubmit '$tmp/settings.json'"
check "every command ends in || true" "! printf '%s\n' \"\$commands\" | grep -v '|| true\$' | grep -q ."
while IFS= read -r cmd; do
    PM_LOG="$tmp/no-log" sh -c "$cmd" </dev/null >/dev/null 2>&1
    check "exit 0 without log: $cmd" "[ $? -eq 0 ]"
    gone=${cmd//$tmp\/pm/$tmp\/gone}
    sh -c "$gone" </dev/null >/dev/null 2>&1
    check "exit 0 without scripts: $gone" "[ $? -eq 0 ]"
done <<<"$commands"
PM_LOG="$tmp/no-log" sh "$tmp/pm/pm-hook.sh" card </dev/null >"$tmp/card.out" 2>&1
check "card warns when the log worktree is missing" "grep -q 'PM WARNING' '$tmp/card.out'"

repo="$tmp/repo"
git init -q -b claude/session-log "$repo"
git -C "$repo" -c user.name=t -c user.email=t@t commit -q --allow-empty -m log
git -C "$repo" branch other
mkdir -p "$tmp/githooks"
cp "$root/tools/hooks/post-checkout" "$tmp/githooks/"
git -C "$repo" config core.hooksPath "$tmp/githooks"
PM_LOG="$repo" git -C "$repo" checkout -q other 2>/dev/null
check "guard switches the log worktree back" "[ \"\$(git -C '$repo' symbolic-ref --short HEAD)\" = claude/session-log ]"
PM_LOG="$repo" git -C "$repo" checkout -q --detach other 2>/dev/null
check "guard switches back from a detached HEAD" "[ \"\$(git -C '$repo' symbolic-ref --short -q HEAD)\" = claude/session-log ]"
exit $fail
