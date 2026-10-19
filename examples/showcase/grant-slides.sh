#!/bin/sh
# Grant the capability `screen.capture` for the demo slides.py.
exec "${CHORACTL:-choractl}" grant slides screen.capture
