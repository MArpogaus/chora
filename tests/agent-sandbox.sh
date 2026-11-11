#!/bin/bash
# Test of dist/sandbox/agent-sandbox.sh, the sandbox of the coding agents:
# the arguments reach the command, stdin and stdout pass through, the
# project directory is writable, and the home directory, the runtime
# directory, the socket of chora and the network are not there.
#
# Usage: tests/agent-sandbox.sh
# Exit status: 0 all checks passed, 1 a check failed.
# shellcheck disable=SC2016,SC2329
set -u
repo=$(cd "$(dirname "$0")/.." && pwd)
box=$repo/dist/sandbox/agent-sandbox.sh
work=$(mktemp -d /tmp/agent-sandbox.XXXXXX)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/home/project" "$work/run" "$work/sockets"
touch "$work/home/secret" "$work/run/wayland-0" "$work/sockets/chora.sock"
export HOME=$work/home XDG_RUNTIME_DIR=$work/run CHORA_SOCKET=$work/sockets/chora.sock
unset AGENT_SANDBOX_HOME
cd "$work/home/project" || exit 1

failed=0
check() {
    local name=$1 want=$2 got
    shift 2
    got=$("$@" 2>&1)
    if [ "$got" = "$want" ]; then
        echo "ok: $name"
    else
        echo "FAIL: $name: want '$want', got '$got'"
        failed=1
    fi
}
seen() { "$box" sh -c 'if test -e "$1"; then echo seen; else echo hidden; fi' sh "$1"; }
links() { "$box" "$@" sh -c 'sed -n "3,\$s/:.*//p" /proc/net/dev | sort | xargs'; }
refused() {
    (cd "$1" && "$box" true >/dev/null 2>&1)
    echo $?
}

check "the arguments reach the command" "a b c" "$box" echo a "b c"
check "stdin and stdout pass through" "ping" sh -c "echo ping | '$box' cat"
check "the exit status is that of the command" 7 sh -c "'$box' sh -c 'exit 7'; echo \$?"
check "the project directory is writable" "" "$box" touch written
check "a file written there is in the project" "$work/home/project/written" ls "$work/home/project/written"
check "the home directory is another one" hidden seen "$HOME/secret"
check "the runtime directory is empty" hidden seen "$XDG_RUNTIME_DIR/wayland-0"
check "the socket of chora is not there" hidden seen "$CHORA_SOCKET"
check "CHORA_SOCKET is unset" "" "$box" sh -c 'echo "${CHORA_SOCKET:-}"'
check "there is no network" "lo" links
check "--net keeps the network" "$(sed -n '3,$s/:.*//p' /proc/net/dev | sort | xargs)" links --net
check "the home directory is no project" 2 refused "$HOME"
check "a directory with the socket of chora is no project" 2 refused "$work/sockets"
check "no command is a usage error" 2 sh -c "'$box' >/dev/null 2>&1; echo \$?"
exit "$failed"
