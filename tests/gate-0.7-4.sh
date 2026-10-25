#!/bin/bash
# Gate check 4 of 0.7: a Podman container that mounts only the client
# directory of agent queries the scene and draws a node. Its `grants.set`
# and its `hello` with another name fail. The CI job `podman` in
# .github/workflows/ci.yaml runs this on an ubuntu runner, where podman
# is preinstalled. The answer here is SKIP without podman.
#
# Usage: tests/gate-0.7-4.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need podman
out=$(mktemp -d /tmp/gate4.XXXXXX)
trap 'kill "${pids[@]}" 2>/dev/null; rm -rf "$out"' EXIT
session_start 1280x800
# The client socket fixes the name agent. The capture later proves that
# the node came into being.
"$bin/choractl" clients create agent |
    sed -n 's/.*"path":"\(.*\)\/ipc.sock".*/\1/p' >"$out/dir"
dir=$(cat "$out/dir")
test -S "$dir/ipc.sock" || fail "no ipc.sock in the client directory"
"$bin/choractl" grant agent screen.capture || fail "the grant for agent"
# One container, two mounts: the client directory and the binaries. The
# base image only carries glibc; choractl needs nothing else.
in_agent() {
    podman run --rm -i --network=none -e CHORA_SOCKET=/run/chora/ipc.sock \
        -v "$dir:/run/chora" -v "$bin:/work:ro,z" \
        docker.io/library/ubuntu:24.04 /work/choractl "$@"
}
in_agent query kind:surface >/dev/null || fail "the container cannot query the scene"
echo '[{"create": {"id": "pod", "kind": "rect", "parent": "root",
    "props": {"size": [120, 80], "color": [0, 1, 0, 1],
              "transform": {"translate": [300, 300]}}}}]' |
    in_agent apply - >/dev/null || fail "the container cannot draw a node"
"$bin/choractl" --name agent capture "$out/shot.png" >/dev/null ||
    fail "the capture of the host"
test "$(pixel "$out/shot.png" 360 340)" = "0 255 0 255" ||
    fail "the node of the container shows $(pixel "$out/shot.png" 360 340)"
# The two refusals: grants are for the admin socket, and the socket of
# agent answers only as agent.
in_agent grant agent screen.capture >/dev/null 2>&1 &&
    fail "grants.set worked on a client socket"
in_agent --name other query kind:surface >/dev/null 2>&1 &&
    fail "hello answered as another name on the client socket"
ok "the container works through clients/agent and fails at grants.set and hello"
