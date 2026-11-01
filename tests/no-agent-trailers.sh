#!/usr/bin/env bash
# commit-msg hook: reject agent attribution lines. $1 is the message file.
if grep -iqE 'Co-Authored-By:|Claude-Session:|Generated with' "$1"; then
  echo "commit message has an agent attribution line (Co-Authored-By, Claude-Session, Generated with)" >&2
  exit 1
fi
