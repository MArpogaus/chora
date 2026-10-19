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
