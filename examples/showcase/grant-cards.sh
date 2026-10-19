#!/bin/sh
# Grant the capability `screen.capture` for the demo cards.py.
exec "${CHORACTL:-choractl}" grant cards screen.capture
