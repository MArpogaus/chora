#!/bin/bash
# The act contracts (NET2, NET3, design 3.2 of the arch plan): the selftest of
# the harness without a session (tests/test_acts.py), then
# tests/acts/run.py: the live selftest, which plants a breach of each
# invariant in a session of mc and must see it fail, and the acts of every
# look in its own headless session, judged against
# tests/acts/known-red.toml. A listed failure does not block; a listed
# pass and an unlisted failure print `FAIL:` and fail the gate.
#
# tests/persona-selftest.sh follows: the stand-ins and grants of a persona
# session. The live selftest also plants a breach in a persona session and
# must see it in breaches.jsonl.
#
# Usage: tests/gate-acts.sh [--no-selftest] [LOOK...]   BIN: the binaries
#   LOOK  xp, tiling, mc, construct (default all)
set -u
. "$(dirname "$0")/gate-lib.sh"
need python3
need_bins chora choractl chora-xp chora-mc chora-tiling chora-construct chora-probe
python3 -I "$repo/tests/test_acts.py" || fail "the selftest of the harness"
python3 -I "$repo/tests/test_tour.py" || fail "the selftest of the tour rows (demo/tour)"
"$repo/tests/persona-selftest.sh" || fail "the selftest of the persona session"
python3 -I "$repo/tests/acts/run.py" "$@" || fail "acts: a cell blocks (see the FAIL lines)"
ok "every act and invariant passes or is listed as known red"
