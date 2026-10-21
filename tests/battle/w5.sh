#!/bin/bash
# W5 load: snake stress, ipc-bench, 16 connections, node budget.
WF=w5; . "$(dirname "$0")/lib.sh"
session_start "1280x800" || exit 1

step "snake SNAKE_FOOD=60"
admin grant snake screen.capture >> "$RUN" 2>&1
if PYTHONPATH=$repo/examples/showcase SNAKE_FOOD=60 SNAKE_RUN=30 timeout 120 python3 examples/showcase/snake.py >> "$RUN" 2>&1; then
  step "snake ok"
else
  finding "W5: snake stress run failed" wrong "PYTHONPATH=examples/showcase SNAKE_FOOD=60 python3 snake.py"
fi
health "after snake"

bash tests/ipc-bench.sh "$bin" "$repo/tests/battle/bench" >> "$RUN" 2>&1 || finding "W5: ipc-bench failed" slow "tests/ipc-bench.sh"

step "connection limit and node budget in one python pass"
boxpath=$(admin clients create budget | sed -n 's/.*"path": "\([^"]*\).*/\1/p')
[ -n "$boxpath" ] || finding "W5: clients create budget failed" wrong "choractl clients create budget"
timeout 300 python3 tests/battle/budget.py "$CHORA_SOCKET" "$boxpath" >> "$RUN" 2>&1 \
  || finding "W5: connection or node budget probe failed" wrong "budget.py $CHORA_SOCKET $boxpath"
health "after the budgets probe"
ctl capture "$repo/tests/battle/w5-final.png" --output HEADLESS-1 >> "$RUN" 2>&1
session_stop
