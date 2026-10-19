#!/bin/sh
# Grant the capability `screen.capture` for the demo snake.py.
exec "${CHORACTL:-choractl}" grant snake screen.capture
