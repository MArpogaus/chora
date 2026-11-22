# shellcheck shell=sh
# Sourced by the flavor scripts: sets `runtime`, the runtime directory.
# XDG_RUNTIME_DIR when set; else one fixed directory,
# ${TMPDIR:-/tmp}/chora-UID, made with mode 700. One that is not owned by
# the user or not mode 700 ends the script with one line on stderr and
# exit 1, so that every run finds the pids of the earlier ones.
if [ -n "${XDG_RUNTIME_DIR:-}" ]; then
    runtime=$XDG_RUNTIME_DIR
else
    runtime=${TMPDIR:-/tmp}/chora-$(id -u)
    if [ ! -e "$runtime" ]; then
        mkdir -m 700 "$runtime" 2>/dev/null || [ -d "$runtime" ] || {
            echo "$0: cannot make $runtime" >&2
            exit 1
        }
    fi
    if [ ! -d "$runtime" ] || [ -L "$runtime" ] || [ "$(stat -c %u "$runtime")" != "$(id -u)" ] || [ "$(stat -c %a "$runtime")" != 700 ]; then
        echo "$0: $runtime is not a directory of mode 700 owned by you; remove it or set XDG_RUNTIME_DIR" >&2
        exit 1
    fi
fi
