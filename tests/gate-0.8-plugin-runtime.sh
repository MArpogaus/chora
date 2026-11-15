#!/bin/bash
# Gate check 1 of 0.8, its grep: no plugin crate has a reconnect loop,
# connects by itself or calls methods.export, output.list or
# events.subscribe. The runtime of chora-kit does that for them.
# plugins/clippy.toml makes clippy refuse the connect calls at once; this
# grep stays as the backstop. A hit inside the item or statement that an
# allow of `clippy::disallowed_methods` stands on (a test on a fake core)
# passes; the item ends at the first line of the indent of its attribute
# that ends with `}` or `;` (rustfmt layout).
#
# Usage: tests/gate-0.8-plugin-runtime.sh [DIR]   DIR: plugins/ by default
set -u
. "$(dirname "$0")/gate-lib.sh"
dir=${1:-$repo/plugins}
export PLUGIN_RUNTIME_PATTERN='"(methods\.export|output\.list|events\.subscribe)"|client::(connect|from_env|from_stream)|conn::connect|runtime::with[^[:alnum:]_]|reconnect'
hits=$(find "$dir" -name '*.rs' -exec awk '
    FNR == 1 { allowed = 0 }
    /^[[:space:]]*#\[/ { attribute = $0; sub(/[^[:space:]].*/, "", attribute) }
    /clippy::disallowed_methods/ { allowed = 1; indent = attribute; next }
    tolower($0) ~ ENVIRON["PLUGIN_RUNTIME_PATTERN"] && !allowed { print FILENAME ":" FNR ": " $0 }
    allowed { rest = index($0, indent) == 1 ? substr($0, length(indent) + 1) : " " }
    allowed && rest ~ /[};][[:space:]]*$/ && rest !~ /^([[:space:]]|#|\)\])/ { allowed = 0 }
' {} +)
if [ -n "$hits" ]; then
    fail "the plugin runtime of chora-kit does this:
$hits"
fi
ok "no plugin connects, exports or subscribes by itself"
