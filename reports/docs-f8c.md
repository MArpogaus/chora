# docs-f8c (F8)

## r1

Branch claude/docs-f8c, commit 65ef41a5 on ee53e09c. No code change (2 files: docs/decisions.md, CONTRIBUTING.md).
Step 0: cargo, python3, vale, lychee present; `git fetch --unshallow` ok; `cargo check --workspace` green.

- docs/decisions.md: D107 focus (merge 05b50d26 WM2, looks moved in 10865d0b focus-port), D108 open (582620d8 WM3),
  D109 stack (3781cb7e WM5; D89 stays the root order of the core), D110 notice (926bb214 KIT5: chora-notify the only
  server, zbus not in kit), D111 helper (3619ccb3 KIT7), D112 scene.boxes and texts (4548feb3 NET3), D113 kit::cube
  (dd2a0aa3 WM7a), D114 keysheet (136ca663 KIT3), D115 kit module map (points to CONTRIBUTING). Each describes the code
  as read (files, API, who uses it, the copies.toml rows that guard it). D77 starts with the status line "D95 holds and
  amends D77 (owner, 2026-11-19 05:59)".
- CONTRIBUTING.md "Where shared code lives": 28 lines for crates/chora-kit/src, 7 for crates/chora-kit/src/wm.
- dev-rules.md (this branch): one line pointing to that section.

Done-when table (`grep -c "\bNAME\b"` in docs/decisions.md; the CONTRIBUTING column counts the list line `- \`NAME\``; `wm` is the
folder and is named in prose, plain grep hits it 1 time or more):

| name | decisions.md | CONTRIBUTING.md |
|---|---|---|
| actions | 3 | 1 |
| ask | 3 | 1 |
| book | 2 | 1 |
| chat | 3 | 1 |
| clock | 3 | 1 |
| conn | 1 | 1 |
| cube | 10 | 1 |
| desktop_entry | 1 | 1 |
| effects | 14 | 1 |
| float | 1 | 1 |
| focus | 11 | 1 |
| geometry | 6 | 1 |
| give | 10 | 2 |
| glass | 1 | 1 |
| helper | 2 | 1 |
| hud | 2 | 1 |
| icons | 2 | 1 |
| kept | 9 | 1 |
| keysheet | 1 | 1 |
| lamp | 2 | 1 |
| launch | 2 | 1 |
| launcher | 4 | 1 |
| modes | 4 | 1 |
| notice | 3 | 1 |
| open | 6 | 1 |
| runtime | 4 | 1 |
| settings | 8 | 1 |
| shared | 9 | 1 |
| stack | 3 | 1 |
| state | 13 | 1 |
| testing | 2 | 1 |
| volume | 2 | 1 |
| wm | 8 | 0 |
| wobbly | 3 | 1 |
| words | 7 | 1 |
| xdg | 2 | 1 |

Checks: pre-commit (vale, lychee, typos, commitizen) passed on the commit. `tests/merge-gate.sh --jobs 3` once on 65ef41a5:
all steps passed (build 125 s, test 146 s, e2e 285 s; log dir /tmp/merge-gate.duFi0P); nothing pushed by the gate.

Left open on purpose:
- D95 still reads "proposed (owner, 2026-11-17)" in the Process table; the brief asked only for the D77 line.
- D109 says only tiling holds a `stack::Rules` today (grep over plugins); the other looks keep their own order.
- D115 names the kit modules without a record (chat, clock, conn, desktop_entry, effects, float, geometry, glass, hud,
  icons, lamp, launcher, modes, runtime, settings, shared, volume, wobbly, xdg) as plain building blocks.

## Refactor proposals
- crates/chora-kit/src/wm/mod.rs:9-12 (S): the module doc says open shell surfaces and desks "follow in later slices"; `open`
  is merged. Update the doc in the next code slice (this round changed no code).
- crates/chora-kit/src/wm/stack.rs, plugins/chora-{mc,xp,construct} (M): only tiling uses `Rules`; give the three looks the same
  rules and add a copies.toml row for a hand-made held-on-top order.
- docs/decisions.md D95 (S): set the status to "holds" with the owner date, as D77 now says.

## Review r1 (PM, 17:50; builder: sonnet lane; docs only, 2 files +52 -1)
Checks: each of D107-D114 names a merge that exists and matches it (05b50d26 wm2, 582620d8 wm3, 3781cb7e wm5,
926bb214 kit5b, 3619ccb3 kit7, 4548feb3 net3, dd2a0aa3 wm7a, 136ca663 kit3); D77 carries the D95 status line with the
owner's time; D115 maps the modules without a record; no code change; the lane's gate and pre-commit green.
- MINOR rf-1: D95 row still says "proposed" although the owner accepted it 11-19 05:59 (wrong doc; next docs round).
- MINOR rf-2: crates/chora-kit/src/wm/mod.rs doc says open "follows in later slices" (stale; next docs round).
Verdict (computed): no open MAJOR -> MERGE-OK 65ef41a5. Joins train 68.
