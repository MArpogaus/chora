#!/bin/bash
# Gate check 7 of 0.8: a capture shows the injection mark while a client
# injects input, and a capture later shows it gone. This is the script
# of T8, kept as the numbered gate entry.
#
# Usage: tests/gate-0.8-7.sh   BIN: directory with the binaries
. "$(dirname "$0")/gate-lib.sh" # for the software EGL variables
exec "$repo/tests/injection-indicator.sh"
