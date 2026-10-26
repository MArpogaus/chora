#!/bin/sh
# keep.sh PIDFILE PROGRAM [ARGS...] runs a plugin and starts it again when
# it ends, as Restart= of its systemd unit does on a login. A headless
# session has no user manager, so restart_plugin of lib.sh ends the pid in
# PIDFILE instead of `choractl restart`.
pidfile=$1
shift
while :; do
  "$@" &
  echo $! >"$pidfile"
  wait $!
  sleep 0.2
done
