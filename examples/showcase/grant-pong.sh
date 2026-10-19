#!/bin/sh
# Grant the capability `screen.capture` for the demo pong.py.
exec "${CHORACTL:-choractl}" grant pong screen.capture
