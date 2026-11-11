#!/bin/bash
# A minimal sandbox for the coding agents of chora-voice, with bubblewrap.
# It runs a command in the current directory, the project, and hides the
# rest of the session from it:
#
# - the root file system is read-only, and /tmp, /run and the runtime
#   directory are empty, so no socket of chora, Wayland or D-Bus is there;
# - the directory of CHORA_SOCKET is empty too, and the variable is unset;
# - the home directory is another one: AGENT_SANDBOX_HOME, by default
#   ~/.config/sandbox-agent-home, so no key or file of the user is there;
# - the current directory is the only other writable place;
# - there is no network, unless --net asks for it;
# - stdin and stdout pass through, without a tty, as ACP needs them.
#
# It refuses a project directory that is the home directory, the root, or
# one that holds the socket of chora.
#
# Usage: agent-sandbox.sh [--net] [--] COMMAND [ARG...]
#   --net       keep the network of the host
#   -h, --help  this text
#   AGENT_SANDBOX_HOME  the home directory inside the sandbox
# Exit status: that of COMMAND; 2 a usage error or a refused directory.
set -u

usage() { sed -n '2,/^set -u/p' "$0" | sed '$d; s/^# \{0,1\}//'; }
die() {
    echo "agent-sandbox.sh: $*" >&2
    exit 2
}

net=(--unshare-net)
while [ $# -gt 0 ]; do
    case $1 in
    --net) net=() ;;
    -h | --help)
        usage
        exit 0
        ;;
    --)
        shift
        break
        ;;
    -*) die "unknown option $1" ;;
    *) break ;;
    esac
    shift
done
[ $# -gt 0 ] || die "no command; see --help"
command -v bwrap >/dev/null || die "bwrap (bubblewrap) is not installed"

project=$(pwd -P) || die "no current directory"
home=$(cd "${HOME:-/}" && pwd -P) || die "no home directory"
[ "$project" != / ] && [ "$project" != "$home" ] || die "$project is no project directory"
box_home=${AGENT_SANDBOX_HOME:-$home/.config/sandbox-agent-home}
mkdir -p "$box_home" || die "cannot create $box_home"

hidden=(/tmp /run)
[ -z "${XDG_RUNTIME_DIR:-}" ] || hidden+=("$XDG_RUNTIME_DIR")
if [ -n "${CHORA_SOCKET:-}" ]; then
    sockets=$(cd "$(dirname "$CHORA_SOCKET")" 2>/dev/null && pwd -P) || sockets=
    case $project/ in
    "${sockets:-/nowhere}"/*) die "$project holds the socket of chora" ;;
    esac
    [ -z "$sockets" ] || hidden+=("$sockets")
fi
empty=()
for dir in "${hidden[@]}"; do
    [ -d "$dir" ] && empty+=(--tmpfs "$dir")
done

exec bwrap \
    --ro-bind / / \
    --dev /dev \
    --proc /proc \
    "${empty[@]}" \
    --bind "$box_home" "$home" \
    --bind "$project" "$project" \
    --chdir "$project" \
    --unsetenv CHORA_SOCKET \
    --unsetenv WAYLAND_DISPLAY \
    --unsetenv DBUS_SESSION_BUS_ADDRESS \
    --unshare-user --unshare-ipc --unshare-pid --unshare-uts \
    "${net[@]}" \
    --new-session \
    --die-with-parent \
    -- "$@"
