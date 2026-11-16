#!/usr/bin/env bash
# Prove the pre-commit guards red and green in a temporary git repository.
# Usage: tests/guards.sh   (run by hand or by the guards hook; exits 1 if a guard misbehaves)
set -u
while IFS= read -r variable; do unset "$variable"; done < <(git rev-parse --local-env-vars)
if ! command -v pre-commit >/dev/null; then
  echo "tests/guards.sh needs pre-commit on PATH" >&2
  exit 1
fi
tests_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

new_repo() {
  rm -rf "${work:?}/repo"
  mkdir "$work/repo"
  git -C "$work/repo" init -q
  git -C "$work/repo" config user.email guards@example.com
  git -C "$work/repo" config user.name guards
  git -C "$work/repo" config commit.gpgsign false
}

verdict() {
  local label=$1 want=$2 status=$3
  local got=pass
  [ "$status" -ne 0 ] && got=fail
  if [ "$got" = "$want" ]; then
    echo "ok   $label ($want)"
  else
    echo "FAIL $label (wanted $want, exit $status)"
    failures=$((failures + 1))
  fi
}

hook_environment_case() {
  [ -n "${GUARDS_NESTED:-}" ] && return
  local outer="$work/outer"
  git init -q "$outer"
  git -C "$outer" config user.email guards@example.com
  git -C "$outer" config user.name guards
  git -C "$outer" config commit.gpgsign false
  git -C "$outer" commit -q --allow-empty -m outer
  local head
  head=$(git -C "$outer" rev-parse HEAD)
  GUARDS_NESTED=1 GIT_DIR="$outer/.git" GIT_WORK_TREE="$outer" \
    GIT_INDEX_FILE="$outer/.git/index" bash "${BASH_SOURCE[0]}" >/dev/null 2>&1
  local nested=$? bare status
  bare=$(git -C "$outer" config --get core.bare)
  status=1
  if [ "$nested" -eq 0 ] && [ "$bare" = false ] && [ "$(git -C "$outer" rev-parse HEAD)" = "$head" ] &&
    [ -z "$(git -C "$outer" status --porcelain)" ]; then
    status=0
  fi
  verdict "guards.sh leaves the repository of a hook environment alone" pass $status
}

hook_environment_case

missing_pre_commit_case() {
  [ -n "${GUARDS_NESTED:-}" ] && return
  mkdir "$work/bin"
  ln -s "$(command -v git)" "$work/bin/git"
  local output status=1
  output=$(env PATH="$work/bin" "$(command -v bash)" "${BASH_SOURCE[0]}" 2>&1)
  case "$?:$output" in 1:*"needs pre-commit on PATH"*) status=0 ;; esac
  verdict "guards.sh names a missing pre-commit" pass $status
}

missing_pre_commit_case

comment_case() {
  local label=$1 want=$2 path=$3 base=$4 added=$5
  new_repo
  cd "$work/repo" || exit 1
  mkdir -p "$(dirname "$path")"
  if [ -n "$base" ]; then
    printf '%b' "$base" >"$path"
    git add "$path"
    git commit -q -m base
  fi
  printf '%b' "$added" >>"$path"
  git add "$path"
  python3 -I "$tests_dir/no-inline-comments.py" "$path" >/dev/null
  verdict "no-inline-comments: $label" "$want" $?
}

rename_case() {
  local label=$1 want=$2 old=$3 new=$4 base=$5 added=$6
  new_repo
  cd "$work/repo" || exit 1
  printf '%b' "$base" >"$old"
  git add "$old"
  git commit -q -m base
  git mv "$old" "$new"
  printf '%b' "$added" >>"$new"
  git add "$new"
  python3 -I "$tests_dir/no-inline-comments.py" "$new" >/dev/null
  verdict "no-inline-comments: $label" "$want" $?
}

rename_case "rust rename keeps old comments" pass a.rs b.rs '// old\nfn a() {}\n' 'fn b() {}\n'
rename_case "rust rename checks new comments" fail a.rs b.rs '// old\nfn a() {}\n' '// new\n'
rename_case "shell rename keeps old comments" pass a.sh b.sh '#!/bin/sh\nls\n# old\nls\n' 'ls\n'

comment_case "rust line comment" fail a.rs "" '// explain\nfn a() {}\n'
comment_case "rust trailing comment" fail a.rs "" 'fn a() {} // explain\n'
comment_case "rust doc comment" pass a.rs "" '/// docs\n//! crate docs\nfn a() {}\n'
comment_case "rust SAFETY comment" pass a.rs "" '// SAFETY: checked\nfn a() {}\n'
comment_case "rust url in string" pass a.rs "" 'const U: &str = "http://x.org";\n'
comment_case "rust slashes in string" pass a.rs "" 'const U: &str = "a // b";\n'
comment_case "rust slashes after a colon outside a string" fail a.rs "" 'fn a() { open(http://x.org); }\n'
comment_case "rust old comment untouched" pass a.rs '// old\nfn a() {}\n' 'fn b() {}\n'
comment_case "shell comment line" fail a.sh '#!/bin/sh\n# header\n\nls\n' '# explain\nls\n'
comment_case "shell trailing comment" fail a.sh "" 'ls # explain\n'
comment_case "shell header and shebang" pass a.sh "" '#!/bin/sh\n# usage: a.sh\n# more usage\nls\n'
comment_case "shell directive" pass a.sh '#!/bin/sh\nls\n' '# shellcheck disable=SC2086 reason\nls \x24x\n'
comment_case "shell hash in quotes" pass a.sh '#!/bin/sh\nls\n' 'echo "a # b" '"'"'c # d'"'"'\n'
comment_case "shell length expansion" pass a.sh '#!/bin/sh\nls\n' 'echo "\x24{#x}" \x24#\n'
comment_case "python comment line" fail a.py "" 'x = 1\n# explain\n'
comment_case "python trailing comment" fail a.py "" 'x = 1  # explain\n'
comment_case "python header" pass a.py "" '#!/usr/bin/env python3\n# usage: a.py\nx = 1\n'
comment_case "python noqa and type" pass a.py 'x = 1\n' 'import os  # noqa: F401\ny = []  # type: list\n'
comment_case "python hash in string" pass a.py 'x = 1\n' 's = "#a"\n'
comment_case "other files unchecked" pass notes.txt "" '# explain\n'
comment_case "rust raw string" pass a.rs "" 'const S: &str = r#"\n// inside\n"#;\n'
comment_case "rust multi-line string" pass a.rs "" 'const S: &str = "a\n// inside\nb";\n'
comment_case "rust string opened in old lines" pass a.rs 'const S: &str = r#"\nold\n' '// inside\n"#;\n'
comment_case "rust comment after raw string" fail a.rs "" 'const S: &str = r#"x"#; // explain\n'
comment_case "python triple-quoted string" pass a.py "" 's = """\n# inside\n"""\n'
comment_case "python comment after triple quotes" fail a.py "" 's = """\nx\n"""\n# explain\n'
comment_case "shell heredoc" pass a.sh "" 'cat <<EOF\n# inside\nEOF\n'
comment_case "shell quoted heredoc" pass a.sh "" "cat <<'END'\\n# inside\\nEND\\n"
comment_case "shell comment after heredoc" fail a.sh "" 'cat <<EOF\nx\nEOF\n# explain\n'
comment_case "shell arithmetic shift is no heredoc" fail a.sh "" 'x=\x24((1<<n))\n# explain\n'
comment_case "shell heredoc word in quotes" fail a.sh "" 'echo "<<EOF"\n# explain\n'
comment_case "form feed does not shift the line numbers" fail a.rs "" 'let s = "a\fb";\n// explain\n'
comment_case "python pragma" pass a.py "" 'x = 1  # pragma: no cover\n'
comment_case "rust rustfmt skip comment" fail a.rs "" '// rustfmt::skip\nfn a() {}\n'
comment_case "rust four slashes" fail a.rs "" '//// banner\nfn a() {}\n'
comment_case "rust injected reason" pass a.rs "" 'Event::PointerPressed { node, .. } => 1, // injected: draws a hover\n'
comment_case "rust SAFETY without space" pass a.rs "" '//SAFETY: checked\nfn a() {}\n'
comment_case "rust double quote char" fail a.rs "" "fn a() { let c = '\"'; } // explain\\n"
comment_case "rust slashes after quote char" pass a.rs "" "fn a() { let c = '\"'; let s = \"//\"; }\\n"
comment_case "rust escaped quote char" pass a.rs "" "fn a() { let c = '\\\\''; let s = \"//\"; }\\n"
comment_case "extensionless python" fail bin/tool "" '#!/usr/bin/env python3\nx = 1  # explain\n'
comment_case "extensionless shell" fail bin/tool "" '#!/bin/sh\nls # explain\n'
comment_case "extensionless header" pass bin/tool "" '#!/bin/sh\n# usage: tool\nls\n'
comment_case "extensionless without shebang" pass bin/tool "" '# explain\n'
comment_case "path with spaces" fail "my dir/a file.rs" "" '// explain\n'

message_names_the_rules() {
  new_repo
  cd "$work/repo" || exit 1
  printf '// explain\n' >a.rs
  git add a.rs
  local output
  output=$(python3 -I "$tests_dir/no-inline-comments.py" a.rs)
  local status=1
  case "$output" in
    *"a.rs:1: // explain"*"SAFETY"*"directives"*"usage header"*"doc comment"*) status=0 ;;
  esac
  verdict "no-inline-comments: message names the rules" pass $status
}
message_names_the_rules

session_case() {
  local label=$1 want=$2 path=$3
  new_repo
  cd "$work/repo" || exit 1
  mkdir -p "$(dirname "$path")"
  printf 'x\n' >"$path"
  git add "$path"
  pre-commit run no-session-files --files "$path" \
    -c "$tests_dir/../.pre-commit-config.yaml" >/dev/null 2>&1
  verdict "no-session-files: $label" "$want" $?
}

session_message_case() {
  new_repo
  cd "$work/repo" || exit 1
  printf 'x\n' >state.md
  git add state.md
  local output
  output=$(pre-commit run no-session-files --files state.md \
    -c "$tests_dir/../.pre-commit-config.yaml" 2>&1)
  local status=1
  case "$output" in *"belong outside the repository"*) status=0 ;; esac
  verdict "no-session-files: message says where they belong" pass $status
}

session_message_case
session_case "plans/autonomy" fail plans/autonomy/plan.md
session_case "reports" fail reports/day1.md
session_case "reflections" fail reflections/a.md
session_case "media" fail media/shot.png
session_case "drafts" fail drafts/a.md
for name in charter state ledger friction answers questions backlog HANDOFF; do
  session_case "top-level $name.md" fail "$name.md"
done
session_case "docs/reports is code" pass docs/reports/a.md
session_case "plans/other" pass plans/other/a.md
session_case "nested state.md" pass docs/state.md
session_case "source file" pass src/main.rs

message_case() {
  local label=$1 want=$2 text=$3
  printf '%b' "$text" >"$work/message"
  python3 -I "$tests_dir/commit-message-length.py" "$work/message" >/dev/null 2>&1
  verdict "commit-message-length: $label" "$want" $?
}

subject_68=$(printf 's%.0s' $(seq 68))
subject_69=$(printf 's%.0s' $(seq 69))
body_72=$(printf 'b%.0s' $(seq 72))
body_73=$(printf 'b%.0s' $(seq 73))
long_url="https://example.org/$(printf 'u%.0s' $(seq 100))"

message_case "subject of 68" pass "$subject_68\n"
message_case "subject of 69" fail "$subject_69\n"
message_case "body line of 72" pass "short\n\n$body_72\n"
message_case "body line of 73" fail "short\n\n$body_73\n"
message_case "long URL line" pass "short\n\n$long_url\n"
message_case "line starting with a URL" pass "short\n\n$long_url and $body_73\n"
message_case "long comment line" pass "short\n\n# $body_73\n"
message_case "long line after a URL line" fail "short\n\n$long_url\n$body_73\n"
message_case "scissors line hides the diff" pass "short\n\n# ------------------------ >8 ------------------------\n$body_73\n"
message_case "long line before the scissors" fail "short\n$body_73\n# ------------------------ >8 ------------------------\n"
message_case "long merge subject" pass "Merge branch 'x': $subject_69\n"
message_case "long revert subject" pass "Revert \"$subject_69\"\n"
message_case "long fixup subject" pass "fixup! $subject_69\n"
message_case "long squash subject" pass "squash! $subject_69\n"
message_case "long amend subject" pass "amend! $subject_69\n"
message_case "long body line of a merge" fail "Merge branch 'x'\n\n$body_73\n"
message_case "leading blank line is no subject" fail "\n$subject_69\n"
message_case "body after a leading blank line" fail "\nshort\n\n$body_73\n"
message_case "subject after a leading blank line" pass "\n$subject_68\n"

kill_case() {
  local label=$1 want=$2 text=$3
  printf '%b' "$text" >"$work/script.sh"
  bash "$tests_dir/no-foreign-kill.sh" "$work/script.sh" >/dev/null
  verdict "no-foreign-kill: $label" "$want" $?
}

kill_case "the newest core of the machine" fail 'core=\x24(pgrep -x chora | tail -1)\n'
kill_case "the core of the own session" pass 'core=\x24(pgrep -P "\x24session" -x chora)\n'
kill_case "a plugin by its name machine-wide" fail 'pgrep -x chora-tiling\n'
kill_case "a plugin as a child of the own core" pass 'pgrep -P "\x24chora_pid" -x chora-xp\n'
kill_case "a command line machine-wide" fail 'pgrep -f -- "--app-id=mcside"\n'
kill_case "pidof" fail 'pidof chora-xp\n'
kill_case "the processes of the own runtime directory" pass 'session_pids -f -- "--app-id=mcside"\n'
kill_case "pkill by name" fail 'pkill -f chora-tiling\n'
kill_case "pkill of the children of a pid" pass 'pkill -P "\x24voice" -x foot\n'
kill_case "pkill in the own process group" pass 'pkill -g "\x24group" -f heartbeat\n'
kill_case "killall" fail 'killall chora\n'
kill_case "a scoped lookup does not excuse a pkill after it" fail 'pgrep -P 1 -x chora; pkill chora\n'
kill_case "a parent attached to -P" pass 'pgrep -P"\x24pid" -x foot\n'
kill_case "the runtime directory filter of session_pids" pass 'for pid in \x24(pgrep "\x24@"); do\n    grep -qzx "XDG_RUNTIME_DIR=\x24XDG_RUNTIME_DIR" "/proc/\x24pid/environ" || continue\n    echo "\x24pid"\ndone\n'
kill_case "pkill of a pid file" pass 'pkill -F "\x24out/voice.pid"\n'
kill_case "the names in a quoted echo" pass 'echo "end it by pid, never with pidof, killall or pkill -f"\n'
kill_case "the names in a plain echo" pass 'echo never pkill chora\n'
kill_case "the names in a here-document" pass 'cat <<EOF\npkill chora\npgrep -x chora\nEOF\n'
kill_case "a check for the tool" pass 'command -v pgrep >/dev/null || exit 1\n'
kill_case "pgrep by a name alone" fail 'core=\x24(pgrep chora)\n'
kill_case "pgrep --exact" fail 'pgrep --exact chora\n'
kill_case "pgrep --full" fail 'pgrep --full -- "--app-id=mcside"\n'
kill_case "a continued line" fail 'pgrep \\\n    -x chora\n'
kill_case "a signal name is no parent" fail 'pkill -HUP chora\n'
kill_case "an echo does not excuse a pkill after it" fail 'echo stop; pkill chora\n'
kill_case "the line after a here-document" fail 'cat <<EOF\ntext\nEOF\npkill chora\n'
kill_case "a quoted pgrep substitution" fail 'running "\x24(pgrep -x chora-xp)" || fail x\n'
kill_case "a quoted pidof substitution" fail 'kill "\x24(pidof chora)"\n'
kill_case "a quoted substitution in an assignment" fail 'pids="\x24(pgrep chora)"\n'
kill_case "a quoted backtick substitution" fail 'kill "\x60pgrep -x chora\x60"\n'
kill_case "the -s of kill is no session" fail 'kill -s KILL \x24(pgrep -x chora)\n'
kill_case "the -s of timeout is no session" fail 'timeout -s KILL 5 pkill chora\n'
kill_case "an echo of a substitution" fail 'echo \x24(pkill chora)\n'
kill_case "a quoted string stays blank" pass 'fail "never pkill chora" || true\n'
kill_case "a loop over tool names" pass 'for t in pgrep pkill jq; do command -v "\x24t"; done\n'
kill_case "a check for several tools" pass 'need pgrep pkill\n'
kill_case "several tools, the kill tool not last" pass 'need pgrep pkill jq\n'
kill_case "a check for several tools, the kill tool not last" pass 'command -v pgrep pkill jq >/dev/null\n'
kill_case "a need naming a kill tool and a target still fails later" fail 'need pgrep pkill jq\npkill chora\n'
kill_case "a scope in a variable" pass 'pgrep \x24scope -x foot\n'
kill_case "an awk program over several lines" pass "awk '\n    /pkill chora/ { n++ }\n' log\n"
kill_case "an arithmetic shift is no here-document" fail 'n=\x24((n << shift))\npkill chora\n'
kill_case "a comment naming environ is no filter" fail 'core=\x24(pgrep -x chora)\n# /proc/PID/environ\n'

sync_case() {
  local label=$1 want=$2 root=$3 plugin=$4
  printf '%b' "$root" >"$work/root.toml"
  printf '%b' "$plugin" >"$work/plugin.toml"
  bash "$tests_dir/clippy-config-sync.sh" "$work/root.toml" "$work/plugin.toml" >/dev/null
  verdict "clippy-config-sync: $label" "$want" $?
}

sync_case "same settings and a plugin list" pass '# c\na = 1\nb = 2\n' 'b = 2\ndisallowed-methods = [\n  { path = "x" }\n]\na = 1\n'
sync_case "a setting only in the root" fail 'a = 1\nb = 2\n' 'a = 1\n'
sync_case "a setting only in the plugins" fail 'a = 1\n' 'a = 1\nb = 2\n'
sync_case "a changed value" fail 'a = 1\n' 'a = 2\n'

optout_case() {
  local label=$1 want=$2 text=$3
  printf '%b' "$text" >"$work/optout.rs"
  python3 -I "$tests_dir/injected-optout.py" "$work/optout.rs" >/dev/null
  verdict "injected-optout: $label" "$want" $?
}

optout_case "a key pattern that drops the field with a wildcard" fail 'match e { Event::NodeKey { node, injected: _, .. } => 1 }\n'
optout_case "a key pattern that ends in the rest pattern" fail 'match e { Event::NodeKey { node, state, .. } => 1 }\n'
optout_case "a press pattern that ends in the rest pattern" fail 'if let Event::PointerPressed { node, .. } = e {}\n'
optout_case "a release pattern that ends in the rest pattern" fail 'if let Event::PointerReleased { node, .. } = e {}\n'
optout_case "a drag pattern that ends in the rest pattern" fail 'matches!(e, Event::PointerMoved { .. })\n'
optout_case "the wildcard on a line of its own" fail 'Event::NodeKey {\n    node,\n    injected: _,\n} => 1,\n'
optout_case "the rest pattern on a line of its own" fail 'Event::NodeKey {\n    node,\n    ..\n} => 1,\n'
optout_case "the wildcard with a reason" pass 'Event::NodeKey {\n    node,\n    injected: _, // injected: the look draws no key\n} => 1,\n'
optout_case "the rest pattern with a reason" pass 'Event::PointerPressed { node, .. } => 1, // injected: only draws a hover\n'
optout_case "the reason on the next line" pass 'let Event::NodeKey { node, .. } = e else {\n    // injected: a repeat\n    return;\n};\n'
optout_case "a comment without a reason does not count" fail 'Event::PointerPressed { node, .. } => 1, // injected:\n'
optout_case "every field named" pass 'Event::NodeKey { node, key, code, state, modifiers, text, injected } => 1,\n'
optout_case "the field named with a rest pattern" pass 'Event::PointerPressed { node, injected, .. } => 1,\n'
optout_case "the field compared with a rest pattern" pass 'Event::InputKey { injected: true, .. } => 1,\n'
optout_case "a constructor with a base" pass 'let e = Event::NodeKey { node, injected: false, ..base };\n'
optout_case "a slice rest pattern inside a field" pass 'Event::NodeKey { modifiers: [first, ..], injected, node } => 1,\n'
optout_case "a hover pattern that ends in the rest pattern" fail 'Event::PointerEntered { node, .. } => 1,\n'
optout_case "a scroll pattern that ends in the rest pattern" fail 'Event::PointerScroll { node, dy, .. } => 1,\n'
optout_case "a button grab pattern that ends in the rest pattern" fail 'Event::InputButton { state, .. } => 1,\n'
optout_case "a grab move pattern that ends in the rest pattern" fail 'Event::InputMoved { position, .. } => 1,\n'
optout_case "the field named in a button grab pattern" pass 'Event::InputButton { state, injected, .. } => 1,\n'
optout_case "the field bound to a name with an underscore" fail 'Event::NodeKey { node, injected: _inj, .. } => 1,\n'
optout_case "the field bound to a bare underscore name" fail 'Event::NodeKey { node, injected: _, .. } => 1,\n'
optout_case "the field bound to another name" pass 'Event::NodeKey { node, injected: heard, .. } => 1,\n'
optout_case "the flag bound into another field" fail 'Event::NodeKey { node, text: Some(injected), .. } => 1,\n'
optout_case "the flag bound into another field on its own line" fail 'Event::NodeKey {\n    node,\n    text: Some(injected),\n    ..\n} => 1,\n'
optout_case "the field and a name that holds it" pass 'Event::NodeKey { node, text: Some(injected_text), injected, .. } => 1,\n'
optout_case "the field set from an expression" pass 'let e = Event::InputMoved { position, injected: state.injecting };\n'
optout_case "an event without the field" pass 'Event::SurfaceCreated { surface, .. } => 1,\n'
optout_case "the pattern in a comment" pass '/// Event::NodeKey { node, .. }\n'

message_output_case() {
  local label=$1 text=$2 want=$3
  printf '%b' "$text" >"$work/message"
  local output
  output=$(python3 -I "$tests_dir/commit-message-length.py" "$work/message")
  local status=1
  case "$output" in *"$want"*) status=0 ;; esac
  verdict "commit-message-length: $label" pass $status
}

message_output_case "labels the subject" "$subject_69\n" "subject: 69 characters"
message_output_case "labels the body line" "short\n\n$body_73\n" "body line 3: 73 characters"

[ "$failures" -eq 0 ] && echo "all guards behave" || echo "$failures guard check(s) failed"
[ "$failures" -eq 0 ]
