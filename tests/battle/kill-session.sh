#!/bin/bash
# kill-session.sh WF...: end the leftovers of the sessions that these
# workflows started, and nothing else: other agents run chora sessions on
# the same host. session_start of lib.sh writes the pid of the core to
# /tmp/bt-WF.core; the core leads its process group, which holds the
# plugins, their keep.sh loops and the programs that they started. The
# probe windows of `open` are in /tmp/bt-WF.pids.
[ $# -gt 0 ] || { echo "usage: $0 WF..." >&2; exit 2; }
for wf in "$@"; do
  core=$(cat "/tmp/bt-$wf.core" 2>/dev/null)
  # a recycled pid of another program must not die
  if [ -n "$core" ] && [ "$(cat "/proc/$core/comm" 2>/dev/null)" = chora ]; then
    kill -TERM -- "-$core" 2>/dev/null
    sleep 1
    kill -KILL -- "-$core" 2>/dev/null
  fi
  # shellcheck disable=SC2013 # pids contain no blanks
  for pid in $(cat "/tmp/bt-$wf.pids" 2>/dev/null); do kill "$pid" 2>/dev/null; done
  rm -f "/tmp/bt-$wf.core" "/tmp/bt-$wf.pids"
done
exit 0
