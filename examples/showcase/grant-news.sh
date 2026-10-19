#!/bin/sh
# Grants the key injection to the name `agent`, which the news reader
# docs use for Page_Down and Page_Up.
exec "${CHORACTL:-choractl}" grant agent input.inject
