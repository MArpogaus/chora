#!/bin/bash
# Gate check 9 of 0.8: a text node shows "Grüße 你好 🙂" with fallback
# fonts, and a pixel test against a reference capture passes. The text
# node is task T9, which stays open for two blockers.
#
# Usage: tests/gate-0.8-9.sh
set -u
. "$(dirname "$0")/gate-lib.sh"
test -d "$HOME/.cargo/registry" &&
    find "$HOME/.cargo/registry" -name "cosmic-text*" -print -quit |
    grep -q . ||
    skip "cosmic-text is not in the cargo cache and crates.io is unreachable; the text node stays open as T9"
skip "no CJK or emoji font is installed here, so no reference capture of 你好 🙂 can be made; see T9"
