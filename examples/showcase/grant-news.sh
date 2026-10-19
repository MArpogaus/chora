#!/bin/sh
# Grants the key injection to the name `demo-keys`, which the news reader
# docs use for Page_Down and Page_Up.
exec "${CHORACTL:-choractl}" grant demo-keys input.inject
