#!/bin/bash
# Gate check 1 of 0.8, its grep: no plugin crate has a reconnect loop or
# calls methods.export, output.list or events.subscribe. The runtime of
# chora-kit does that for them.
#
# Usage: tests/gate-0.8-plugin-runtime.sh [DIR]   DIR: plugins/ by default
set -u
. "$(dirname "$0")/gate-lib.sh"
dir=${1:-$repo/plugins}
pattern='"(methods\.export|output\.list|events\.subscribe)"|Client::(connect|from_env)|reconnect'
if hits=$(grep -rniE --include='*.rs' "$pattern" "$dir"); then
    fail "the plugin runtime of chora-kit does this:
$hits"
fi
ok "no plugin connects, exports or subscribes by itself"
