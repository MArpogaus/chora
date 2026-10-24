#!/bin/bash
# Gate check 11 of 0.8: the gate checks of the earlier milestones still
# hold, and the showroom CI is green against the release candidate. The
# suites of the earlier milestones are nested (tests/nested.sh), xp
# (tests/xp.sh), agent (tests/agent.sh) and the battle gate
# (tests/battle/battle.sh, W1 to W8); plans/0.8-then-battle.sh chains
# the battle suite to this plan. CI runs on GitHub after a push, which
# stays with Marcel.
#
# Usage: tests/gate-0.8-11.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
if command -v sway >/dev/null 2>&1 && command -v foot >/dev/null 2>&1 &&
    command -v grim >/dev/null 2>&1; then
    for suite in nested xp agent; do
        "$repo/tests/$suite.sh" "$bin" || fail "$suite.sh failed"
    done
else
    skip "sway, foot and grim are not installed here: the nested, xp and agent suites of the earlier milestones cannot run, the battle gate runs through plans/0.8-then-battle.sh, and the showroom CI needs a push, which stays with Marcel"
fi
ok "the earlier gate checks still hold"
