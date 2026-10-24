# Results of the 0.6 review loop and battle test (2026-10-19/20)

Review: `git diff v0.5.4..HEAD`, seven round reports in
`target/review-loop/round-*.md`, each audited (session JSONL: diff calls,
no repo mutations). Two consecutive CLEAN rounds closed the phase.

Fixes that came out of it (all on `dev`, nothing pushed):

- `2c2125e` fix(transaction): re-check the remove target at the end of a wait
- `683145f` fix(headless): cap the live virtual outputs at 16
- `6d492ab` fix(protocols): drop early surface requests beyond 64
- `77db138` fix(choractl): let clients up go on past a name in its grace time
- `1dbc003` test(probe): ask for server-side decorations with --server-deco

Battle: four passes of W1-W8; pass 3 and 4 were clean (THE END per TODO).
One finding: "tag moves landed on 3 windows" — proven a harness race by a
unit test over the tiling core (fix.log), the workflow tolerance is >= 3.
Everything else green, including 40 windows through 108 layout changes,
50 hotplug rounds and 20 plugin restarts, snake at 16.4 tick/s, the
17th-connection refusal, the watchdog as the de-facto node limit at
~14.6k nodes, and every sandbox escape attempt refused.

Design friction with API/doc consequences: see [FRICTION.md](FRICTION.md).

# 0.7 battle re-run (2026-10-21 .. 2026-10-23)

Same W1-W8 suite, rebuilt on the 0.7 primitives (`choractl wait`,
`input.inject` dry run and `hold`, compact `spawn`). Driver
`tests/battle/battle.sh` under the `local-pi-session` heartbeat.

Two consecutive passes with no new distinct finding closed the phase.
W5 reproduced its stress finding in the first of those passes; the
finding title was already known and judged, so the pass still counted
clean under the distinct-title judge (see FRICTION backlog).

- harness: `b6f00af` (wait the WM handoff after drags, first-block
  parse), `79f74b6` (fix agents got starved by stdin hijack)
- product: `a55a764` fix(ipc) - a refused client reads its error line
- open, by judgement not by time: silent core death after the budgets
  probe under load, W7 tiling silent after SIGUSR1, watchdog
  tolerance against stress clients -> FRICTION.md

# prep-0.8 validation (2026-10-24)

Branch `prep-0.8` = `dev` + `b3a6d84` (WM plugins reconnect after the
emergency reset). Full battle suite: two clean passes, THE END at
02:56 UTC. W7 (SIGUSR1) passes without a finding for the first time;
the W5 stress finding did not appear at all on this branch.
