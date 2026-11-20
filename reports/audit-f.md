# audit-f r1: foundation criteria F1-F8 measured mid-week, gaps ranked into slices

Lane: cloud, opus, read-only on code, 11-19 14:00-14:40 UTC. Brief briefs/audit-f-r1.md.
Trees measured:
- **H** = session head dd2a0aa3 (train 62). Train 63 (05878d2a, flakes-f) merged during the audit; it
  touches no kit, size or copies line, so the H numbers hold for it.
- **P** = projection: claude/integ 3619ccb3 (wm3, wm7b, wm5, kit5b, kit7 on H) plus cubesplit 10cf04cf,
  flakes-f 847f0964 and flakes-g c227d433, all merged clean. kitcov (helper.rs) and kit3 (11 files)
  conflict with integ and are not in P.
- **RC** = fe8ba6d0, the tree of reports/debt-rc.md.

Rule used: met = measured value, command and a check by someone who did not build it (this audit
counts as that check for what it re-measured). "Partly" is open.

## 0. Found on the way: integ is red

`cargo test -p chora-xp --bin chora-xp wm::state` fails on claude/integ 3619ccb3 and on claude/wm3
e15155da itself (every integ merge from 582620d8 on): `a_give_shuts_the_start_menu` and
`a_give_with_the_drag_shuts_the_start_menu` panic "xp has no window s1" (plugins/chora-xp/src/wm/state.rs:254,
:270 on wm3). The tests push the window into `wm.windows` by hand; `give` goes through the book since
WM1d, and the book does not know s1. Deterministic, not a flake, not XDG-related (fails with and without
`XDG_RUNTIME_DIR`). Fix: create the window through the book (`FakeCore` event) in both tests. Sent to the
PM at 14:2x. Train 65 cannot go green without it.

## 1. F1-F8

| # | measured value | command | state | gap (one line) |
|---|---|---|---|---|
| F1 | plan reports/arch-plan-fable.md: 27 slices, each with a gate column and cost; rank 4.1; two opus reviews (review-arch-plan-risk.md, -fit.md) and plan-critique-1119.md | read | **met** | none |
| F2 | kit modules used by all four looks on P: book, actions, settings::Common, notice, launch, helper, wm::open. Missing: keysheet (KIT3 not merged; 2 I5 rows red). Copy rows not at 0 on P: Command::Focus 21 (10 in looks), Event::FocusChanged 9 (max 8: **grew**), confirm.answer 8, UnixListener::bind in plugin tests 6, open entry by hand 4, surface filter 1 (mc acts.rs:33). ARC copies with no row: `fn on_path` 4 (kit launcher.rs:222, desktop_entry.rs:188, xp session.rs:401, voice config.rs:1338) | `tools/cut-check` (REPORT lines); per-row counts with the cut-check rule (script in section 5) | open | KIT3 unmerged; focus copies in all four looks; six rows above 0; on_path has no row |
| F3 | designed: plan 1.1, 1.2 (cube role), 1.3 (desks), 2.2 (API). Merged on H: WM1 (train 52), WM1b-d, WM2, WM7a; on P also WM3 (open), WM5 (stack), WM7b. `crates/chora-kit/src/wm/` = book, focus, give, kept, open, stack, state | `ls crates/chora-kit/src/wm`; `git merge-base --is-ancestor` | **met** | WM4 (desks) and WM6 are cut; next week |
| F4 | production lines over 1,500: H 1 file (construct cube.rs 1,641, listed); P 0 (cubesplit). Debt counts: reports/debt-2026-11-19.md counts sizes only, not the five debt-rc categories (workarounds 9, refactors 7, design questions 8, records 9, hygiene 14), so "lower" is not measured. Lint lines grew RC to P: `#[allow]` 378 to 388, without `reason` 238 to 246, `eprintln!` in core, plugins, services 135 to 147 | `tools/size-ratchet`; size script (section 5); `grep -rE '#\[(allow\|expect)\(' crates plugins services --include=*.rs \| grep -vc reason`; `grep -rE 'eprintln!' crates/chora/src plugins services --include=*.rs \| wc -l` | open | cubesplit unmerged; no recount in debt-rc's categories; lint counts higher than RC |
| F5 | chora-kit lines: H **90.75 %** (8,820 lines, 816 missed), P 91.20 %; under 80 % on H: conn.rs 63.9, runtime.rs 68.7, icons.rs 76.0, launcher.rs 76.2, helper.rs 77.4 (kitcov raises them, not merged). Kit modules without a FakeCore test: cube.rs (talks to the core through Conn), wm/focus, wm/open, wm/give, wm/kept. Modules added since RC in the four looks and the kit (52): **20 under 80 % lines** by unit tests, e.g. construct cube/construct.rs 3.4 %, xp wm/panels.rs 10.3 %, wm/press.rs 17.0 %, wm/pointer.rs 24.8 %, wm/keys.rs 24.9 %, tiling methods.rs 0 %, overlay.rs 30.9 %, mc acts.rs 15.6 %. Per look on P: mc 88.1 %, tiling 84.9 %, xp 76.1 %, construct 60.3 %. Flakes: F17 fix merged (train 63); open F16, F18, F19, F26, F28, pull 169 px (flakes-g), construct-hud claim; trains on 11-19 red or rerun on a flake: 54 (F29), 60 (F21); 62 was rerun after the gate shell died, not a flake | `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features chora-kit/testing --summary-only`; looks: `cargo llvm-cov --no-fail-fast -p chora-mc -p chora-xp -p chora-tiling -p chora-construct -p chora-kit --features chora-kit/testing` then `cargo llvm-cov report --summary-only` | open | kit total met; moved modules lack tests; FakeCore cells missing for cube and four wm parts; flaky gate not shown absent |
| F6 | all 29 first-parent commits on the session branch since 11-17 10:56 are train merges, which only `tests/merge-gate.sh --push` pushes (procedures.md Train 3); the deps check blocks a plugin-to-plugin dependency (normal, dev, build, nested) | `git log --first-parent --since="2026-11-17 10:56" --format=%s <head>`; `tests/cut-check-selftest.sh` (ok) | **met** | none for the measure; copies stay report-only above 0 (F2) |
| F7 | tour v9 not possible this week (owner 11-17 14:20: no tour work). State-level part on P: `tests/acts/known-red.toml` 6 rows: KIT3 2 (I5 tiling, mc), WM6 1 (I13), WM2 3 (kit7-a1 mc I3; kit7-a2 construct Tab act and I4). kit7-a1/a2 were found on 11-19 after the refactor began; whether tour v8 had them is not shown | `grep -c '^\[\[red' tests/acts/known-red.toml`; slice rows | open | v9 next week; kit7-a1/a2 may be refactor regressions |
| F8 | records on P: D96-D106 (FakeCore, acts, actions, book, ask, paths, words, ratchet/cut, launch, D92 kept, hosted cube). No record for kit::wm::focus (WM2), kit::wm::open (WM3), kit::wm::stack (WM5; D89 is the core order only), notice (KIT5: chora-notify the only server, zbus out of kit), helper presenter (KIT7), scene.boxes and texts (NET3), kit::cube (WM7a), keysheet (KIT3). D77 lacks the D95 status line (owner 11-19 05:59). CONTRIBUTING.md:17 lists kit parts without focus, open, stack, notice, launch, cube, keysheet. dev-rules.md (this branch) names the cut, not where shared code lives | `grep -n` over docs/decisions.md, CONTRIBUTING.md, dev-rules.md | open | 8 records, D77 line, two lists |

Met: 3 of 8 (F1, F3, F6). With the in-flight branches merged as they are, still 3: P closes the size half
of F4 and the coverage total of F5 only.

## 2. Slices to close the gaps by Sat 03:59 UTC

Ranked by F moved per % of the week. Before any of them: integ fixes section 0 (in its lane) and takes
kitcov (helper.rs) and cubesplit into train 65; kit3 r2 merges integ (11 conflicting files, expected).

| rank | name | F | files | done criterion | model | cost | conflicts with in-flight |
|---|---|---|---|---|---|---|---|
| 1 | DEBT-R: debt recount | F4 | reports/debt-2026-11-20.md (log branch) | after cubesplit is on the head: the five debt-rc categories recounted item by item with proof (file:line or merge), plus the lint lines with the commands above; F4 met if every category is at or below RC and the total is lower, and the lint lines are at or below RC (else rank 7 runs) | sonnet | 0.8 % | none (read-only) |
| 2 | DOCS-F8c | F8 | docs/decisions.md (D107+ for focus, open, stack, notice, helper, boxes/texts, kit::cube, keysheet; D77 status), CONTRIBUTING.md "Where shared code lives", dev-rules.md (log branch) | every kit::* module and every wm/* part named in CONTRIBUTING and in a record; `grep` per module name hits both; a reviewer that did not write it checks the list against `ls crates/chora-kit/src` | sonnet | 1 % | run last (after ranks 3, 4 and KIT3): records name merges |
| 3 | FOCUS-PORT (WM2b) | F2, F3, F7 | kit wm/focus.rs; mc main.rs, state.rs; construct plugin.rs, camera.rs, adopt.rs, input.rs, scene.rs; tiling main.rs, windows.rs; xp wm/windows.rs, wm/dispatch.rs; assistant main.rs; tools/copies.toml; tests/acts/known-red.toml | the looks hand focus through `kit::wm::focus`: rows "Event::FocusChanged outside kit" and "Command::Focus" at 0 in plugins (`allow` the core, scene and kit testing, which define the type); kit7-a1 and kit7-a2 rows go (I3 mc, construct Tab act, I4); each first reproduced on the v8 build 733967aa by the same choractl steps, and the report says regression or old | opus | 3.5 % | integ (start from integ after section 0); kit3 (copies.toml, known-red.toml: trivial); flakes-g none |
| 4 | COPIES-0 | F2 | kit ask.rs (answer builder), assistant voice.rs, card.rs; voice client.rs, tools.rs, config.rs; kit desktop_entry.rs, launcher.rs; xp session.rs; plugin tests of notify, assistant, construct, tiling, xp wm.rs (FakeCore instead of UnixListener); four state.rs (open entry from `kit::wm::open`); mc acts.rs:33; tools/copies.toml (new row on_path) | every copies row at max 0 or narrowed by an `allow` with a one-line reason in copies.toml; `tools/cut-check` prints no copies REPORT | sonnet | 2.5 % | rank 3 (state.rs of mc: take after rank 3, or rank 3 leaves state.rs to this slice); kitcov (launcher.rs tests: trivial) |
| 5 | MOVED-TESTS | F5 | new test files beside: xp wm/{panels,press,pointer,keys,dispatch,choices,windows}; construct cube/{construct,start,hand,role}; tiling methods.rs, overlay.rs; mc acts.rs, lifecycle.rs, region/hits.rs, windows.rs; kit tests/ cells for cube.rs, wm focus, open, give, kept through FakeCore | each module added since fe8ba6d0 at 70 % lines or more in the looks' llvm-cov run (section 1 F5 command), or listed in the report with why not (drawing code reached only by gates); kit cube and four wm parts each with a FakeCore cell; red-first does not apply (tests of existing behaviour) | haiku, two lanes (xp; construct+tiling+mc+kit) | 2.5 % | rank 3 (xp windows.rs, dispatch.rs, construct files): tests in their own files only; cubesplit must be merged (construct cube/) |
| 6 | FLAKE-SERIES | F5 | none (gate runs); backlog rows | after flakes-g merges: 5 full `tests/merge-gate.sh --jobs 3` runs on the head, no rerun, 5 of 5 green; each red gets a backlog row with step, cell, log; F5 flake part met only with 5 of 5 and no train rerun from then to Sat | haiku | 1 % | none; runs in its own container |
| 7 | LINT-DOWN (only if rank 1 shows lints above RC) | F4 | crates/chora, chora-scene, chora-draw, chora-kit (files no other lane touches) | `#[allow]` without `reason` and non-test `eprintln!` at or below RC (238, 135) by the commands of section 1 | sonnet | 1 % | kitcov (kit runtime.rs tests): rebase after it |

Sum: 12.3 % (11.3 % without rank 7). Per F: F4 0.8 % (1.8 % with rank 7), F8 1 %, F5 3.5 % (ranks 5, 6),
F2 6 % (ranks 3, 4, plus KIT3 in flight). Not sliced: F7 (next week).

Sequence (one gate at a time, about 55 min a train): Thu evening train 65 (integ, kitcov, cubesplit),
kit3 train, flakes-g train; ranks 3 and 5 start now from integ; rank 1 starts after train 65; rank 4 after
rank 3 merges (Fri midday); rank 6 after the flakes-g train (Fri); rank 2 Fri evening, its train Sat
before 03:00.

## 3. F7: what can be measured now

- Visual part (per look, v9 against the v8 stills and videos on claude/media-v8, 95 stills): needs the new
  tour; not this week.
- State part, now: the acts step of the merge gate on the head; value = known-red rows. For F7 a row counts
  as a regression unless the same steps fail on the v8 build (733967aa). Today 3 rows are unproven
  (kit7-a1, kit7-a2); rank 3 proves or fixes them. KIT3 rows go with kit3; WM6 (I13 seams) is a v8 finding,
  cut, stays listed.
- So F7 stays open this week by design; what this week can deliver is "no known-red row that the v8 build
  passes".

## 4. Measures to make exact before the Sat check

- F4 "counts lower": each debt-rc category at or below, total lower, lint lines not above RC (rank 1 uses this).
- F5 "every moved module has unit tests": 70 % lines per module added since fe8ba6d0, or listed (rank 5).
- F5 "no flaky gate": 5 of 5 full gate runs without rerun, and no train rerun on a flake to the end (rank 6).
- F2 "no copy left": every copies row at 0 or narrowed by `allow` with a reason (rank 4).

## 5. Commands used (beyond those named)

- Copies per row with paths: scratch script applying `check_copies` of tools/cut-check row by row, printing
  the count and files; same rule as the gate.
- Production lines per file: the size-ratchet rule (lines before the first `#[cfg(test)]` followed by `mod`;
  tests.rs, testing.rs, test_*.rs, *_tests.rs and tests/ count 0) over crates, plugins, services.
- New modules: `git diff --name-only --diff-filter=A fe8ba6d0 <P> -- plugins/chora-{mc,xp,tiling,construct}
  crates/chora-kit` (tests excluded), joined with the llvm-cov report.

## Refactor proposals

- tools/copies.toml "Command::Focus outside kit::wm" counts the type's definition in chora-scene and core
  ipc (crates/chora-scene/src/scene/commands.rs, crates/chora/src/ipc/clients.rs): it can never reach 0;
  `allow` those crates (S, rank 3).
- `fn on_path` four times (kit launcher.rs:222 private copy beside the pub one in desktop_entry.rs:188):
  one kit fn (S, rank 4).
- xp wm/state.rs tests build windows by hand past the book (section 0): a FakeCore helper
  `wm_with_window(id)` in xp tests so tests go through the book (S).
- chora-kit runtime.rs (1,446 production lines, 68.7 % covered on H): split the dispatch from the
  connection part before the next production change there (M).

## r2

Lane: cloud, opus, read-only on code, 11-19 19:05-20:10 UTC. Brief briefs/audit-f-r2.md.
Trees measured:
- **P** = session head f2b7112a (train 69: b5bf800c + moved-tests-mk3 c00dcf3e + mc-i5 f7a69761 are in it) merged
  locally with construct-red da68737d. Not MERGE-OK clean against f2b7112a: tests/acts/known-red.toml (mc-i5 dropped
  I5, construct-red dropped I13 and the two WM2 rows: union = no row) and docs/decisions.md (D107-D115 beside D116:
  both kept). `cargo check -p chora` green on P.
- **P+DD** = P merged with claude/debt-down c8fe032b (r2; r3 in flight for rd-7). Conflicts: docs/decisions.md
  (D-ungrab beside D116) and plugins/chora-tiling/src/main.rs `on_fake_core_with` (copies-0 changed its return to
  `Log`, debt-down added an allow with a reason on the old signature); P's side taken for the count. r3 must merge the head.
- Setup: the setup script was refused by the auto-mode classifier in this container; every tool it checks was
  already present (cargo, llvm-cov, build packages, git identity), so step 0 ran without it.

### 1. F1-F8 on P

| # | value on P | command | state | gap (one line) |
|---|---|---|---|---|
| F1 | reports/arch-plan-fable.md: 24 slice rows with gate and cost, rank 4.1; reviews review-arch-plan-risk.md, review-arch-plan-fit.md, plan-critique-1119.md on the log branch | `git show origin/claude/session-log:reports/arch-plan-fable.md \| grep -cE '^\| *(WM\|KIT\|NET\|CUT\|MV)'`; headings of the three reviews | **met** | none |
| F2 | tools/copies.toml 38 rows, all `max = 0`, 18 narrowed by `allow` with a comment; `tools/cut-check`: 18 REPORT lines (assets, names, known), **0 copies lines**. Per look (mc, xp, tiling, construct) every one uses `wm::book`, `actions`, `keysheet`, `notice`, `launch`, `settings`, `helper`, `wm::focus`, `wm::open` (grep of `chora_kit::` per part, each >= 1 file per look); the ask card has one presenter (assistant, `kit::ask`), the looks only follow `voice.confirm` to hide hints. Spot checks for copies with no row: `fn language`, `fn on_path`, `fn private_dir`, `XDG_*` outside kit: only error texts, test env and services | `tools/cut-check \| grep -ci copies` (0); `grep '^max' tools/copies.toml \| sort \| uniq -c` (38 x 0); per-part grep | **met** | at risk: debt-down adds `fn report()` three times (xp, mc, construct) with no copies row (rank 4) |
| F3 | `crates/chora-kit/src/wm/` = book, focus, give, kept, open, stack, state (WM1-WM3, WM5, WM7a/b merged; D99, D107-D109, D113); construct-red adds D116 (keys of a new window in the construct) | `ls crates/chora-kit/src/wm`; `grep -oE '^\| D1[01][0-9]' docs/decisions.md` | **met** | WM4 (desks), WM6 (seams) cut to next week, as in r1 |
| F4 | sizes: `ok: size-ratchet (0 files over 1500)`, list empty. Lints on P: `#[allow]`/`#[expect]` **393** (RC 378), without `reason` on the line **249** (RC 238), `eprintln!` **147** (RC 135). Categories (debt-2026-11-20.md minus the 4 known-red rows, which P empties): workarounds 9 (RC 9), hygiene 16 (RC 14), refactors 6 (RC 7). On **P+DD**: 375 / 190 / 127, workarounds 9, hygiene 13: every line at or below RC | `tools/size-ratchet`; `grep -rE '#\[(allow\|expect)\(' crates plugins services --include=*.rs \| wc -l`; same `\| grep -vc reason`; `grep -rE 'eprintln!' crates/chora/src plugins services --include=*.rs \| wc -l`; `grep -c '^\[\[red' tests/acts/known-red.toml` (0) | open | met when debt-down r3 merges (rd-7 MAJOR open) and an independent recount on that head confirms (rank 3) |
| F5 | chora-kit lines **91.19 %** (9,505 lines, 837 missed; under 80 %: runtime.rs 70.4, icons.rs 76.0, launcher.rs 76.2, conn.rs 79.2). 54 modules added since fe8ba6d0 in the four looks and the kit: **53 at 70 % or more**, one under and not listed: **construct cube/construct.rs 41.81 %** (421 lines; added by cubesplit; moved-tests ct r1 row 66 says "re-measure after the cube split", never re-measured). Per look: mc 91.2, xp 89.8, tiling 88.2, construct 66.1 %. Kit cube.rs `exporter`, `list`, `turn` (IO through `Conn`) have no FakeCore cell; the wm parts have model cells (tests/moved_cells.rs, wm_focus.rs, wm_book.rs) and do no IO (D99). All tests pass under llvm-cov (--no-fail-fast, rc 0). Flakes: train reruns since 11-19 14:40: **2**, both MX1 (matrix cell over its time cap: train 65 16:22, green on rerun; train 66 17:14 held), none since matrix-cap b71c5096 (trains 67, 68, 69 green on the first run). reports/flake-series.md not yet on the log branch | `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features chora-kit/testing --summary-only`; `env -u XDG_RUNTIME_DIR cargo llvm-cov --no-fail-fast -p chora-mc -p chora-xp -p chora-tiling -p chora-construct -p chora-kit --features chora-kit/testing --summary-only` joined with `git diff --name-only --diff-filter=A fe8ba6d0 HEAD -- plugins/chora-{mc,xp,tiling,construct} crates/chora-kit` (tests excluded); `git log origin/claude/session-log --since="2026-11-19 14:40"` (train lines) | open | cube/construct.rs at 41.8 %; kit cube IO without a FakeCore cell; flake-series 5/5 not shown; MX1 review m2 open |
| F7 | `tests/acts/known-red.toml`: **0 rows** on P (mc-i5 closed I5; construct-red closed I13, the Tab act and I4). Tour v9 not this week (owner 11-17 14:20) | `grep -c '^\[\[red' tests/acts/known-red.toml` | open | v9 against v8 per look is next week; the state part ("no known-red row") holds |
| F8 | records D96-D116 for the new structure (focus D107, open D108, stack D109, notice D110, helper D111, boxes/texts D112, cube D113, keysheet D114, module map D115, construct keys D116); CONTRIBUTING.md "Where shared code lives" has a line for each of the 29 modules of `crates/chora-kit/src` and the 7 of `src/wm` (script below: no module missing); dev-rules.md:19 points to it; D77 carries the D95 status | `for m in $(ls crates/chora-kit/src/*.rs ...); do grep -q "^- \`$m\`" CONTRIBUTING.md \|\| echo missing $m; done` (no output); `grep -n '^\| D77 \|^\| D95 \|^\| D116' docs/decisions.md` | **met** | backlog only: D95 row still "proposed" (rf-1), D116 still "Proposed (PM accepts at merge)" and claims a Ctrl+N child the code does not match (rc-4), `crates/chora-kit/src/wm/mod.rs:12` "surfaces and desks follow in later slices" (rf-2), CONTRIBUTING.md:17 summary line lacks focus, open, stack, notice, launch, cube, keysheet; D-ungrab number at the debt-down merge (rank 5) |

Met on P: **5 of 8** (F1, F2, F3, F6, F8). F6:

| # | value on P | command | state | gap |
|---|---|---|---|---|
| F6 | 44 first-parent commits on the session branch since 11-17 10:56, all train merges (`Merge branch ... into`); `tests/cut-check-selftest.sh`: ok (deps check blocks a plugin-to-plugin dependency) | `git log --first-parent --since="2026-11-17 10:56" --format='%h %s' origin/claude/autonomous-dev-session-3w-i33pg7 \| grep -v ' Merge '` (empty); `tests/cut-check-selftest.sh` | **met** | none |

With debt-down r3 merged and rechecked: 6 of 8. F5 needs rank 1, rank 2 and the flake series; F7 stays open by design.

### 2. Slices by Sat 03:59 UTC

Ranked by F moved per % of the week. In flight and not repeated here: debt-down r3 (F4), flake-series (F5), train 70
(construct-red).

| rank | name | F | files | done criterion | model | cost | conflicts |
|---|---|---|---|---|---|---|---|
| 1 | CUBE-TESTS | F5 | new plugins/chora-construct/src/cube/tests_construct.rs (a `mod` line in cube.rs); crates/chora-kit/tests/cube.rs (new) | cube/construct.rs at 70 % lines or more in the F5 looks command (295 of 421 covered, about 120 more: `open_cube`, `check_cube`, `retry_cube`, `step_cube`, `close_cube`, `face_nodes`, `edge_nodes`, `add_outputs`, `remove_outputs` through FakeCore), or each range left named with why (a child process the unit test cannot start); kit `cube::exporter`, `list`, `turn` each a FakeCore cell (an exporter found, none found, a listing, a refused turn). Tests only; red-first does not apply | haiku | 0.8 % | debt-down edits cube/start.rs and cube.rs (settle): new files only, one `mod` line; flake-series none |
| 2 | MX1-M2 | F5 | tests/gate-handover-matrix.sh | the cap measures the run's own wall time from its first cell (not the gate's wait in the queue), the line that says PASS prints that time and the cap, and a cap breach prints the slowest five cells; one run at `--jobs 3` beside two other steps shows the printed time under 315 s. Closes matrix-cap review m2 | sonnet | 0.3 % | flake-series: merging it changes the gate under the series; merge after the series' fifth run or let the series count runs on either head |
| 3 | AUDIT-F r3 (Sat check) | F4, F5 (and re-check of all) | reports/audit-f.md "## r3" (log branch) | on the last train head (Fri night, after train 71 and ranks 1, 2, 4, 5): the F4 greps and the debt categories by id against debt-rc, the F5 commands of r2 per module, flake-series result plus "no train rerun since", copies 0, known-red 0, records; one verdict per F with value, command, check. The lane did not build any slice it checks | sonnet | 1 % | none (read-only); runs last |
| 4 | KIT-REPORT | F2 (keeps it met), F4 margin | crates/chora-kit/src (one `report(name, err)`), plugins/chora-{xp,mc,construct}/src/main.rs, tools/copies.toml (row "own report()", max 0); optional: kit `testing::connect` for the about 24 test allows "the test connects to its fake core" | after debt-down merges: no `fn report` outside the kit (`tools/cut-check` blocks a new one), the printed texts unchanged (a unit test per look of the line), `#[allow]` total not above P+DD | sonnet | 0.8 % | debt-down (must start after train 71); copies.toml with any slice adding a row |
| 5 | DOCS-F8d | F8 (keeps it met) | docs/decisions.md (D95 and D116 status lines; D116 drop the Ctrl+N claim or say pid; D-ungrab gets its number D117), docs/api.md (rc-5: "parent at the first commit"), crates/chora-kit/src/wm/mod.rs:12, CONTRIBUTING.md:17 | `grep -n 'proposed\|Proposed' docs/decisions.md` shows no accepted record; `grep -rn D-ungrab` empty; wm/mod.rs names every wm part that exists; CONTRIBUTING:17 names every kit part or points to the list. Backlog rows rf-1, rf-2, rc-4, rc-5 closed | sonnet | 0.4 % | debt-down (decisions.md, api.md, D-ungrab): after train 71; KIT-REPORT none |
| 6 | LINT-REASON | F4 margin | the `#[allow]` lines without any `reason` (117 on DD by the review's count; 75 more have it on a wrapped line) in files no other slice touches | every `#[allow]`/`#[expect]` has a `reason` (a multi-line grep, `rg -U '#\[(allow\|expect)\([^]]*\]'` without `reason`, gives 0) or became `#[expect]` where the lint fires; no code change | haiku | 0.6 % | many files: after train 71, before AUDIT-F r3; skip if the budget is short (F4 is met without it) |

Sum 3.9 % (3.3 % without rank 6). Per F: F5 1.1 % (ranks 1, 2) plus the check; F4 1 % (rank 3, and rank 6 as margin);
F2 and F8 1.2 % to keep them met after debt-down. No slice moves F7 this week: the tour is next week, and the state
part is already at 0 rows; F7 open by design.

Sequence: ranks 1 and 2 start now (no conflict with train 70 or debt-down; rank 2 after the series' fifth run);
train 71 = debt-down r3; ranks 4, 5 (and 6) from the train 71 head, one train Fri afternoon with rank 1 and 2; rank 3
Fri late, after the last train, its report before Sat 03:00.

### 3. Notes

- da68737d and claude/debt-down each conflict with f2b7112a (the session head); train 70 and debt-down r3 must merge the head.
- The PM marked F8 met at train 68; this re-check agrees on the measure, with the five doc defects of rank 5 to backlog.
- F5 "every moved module" was believed met at mk r3; cube/construct.rs fell through because the ct round ran before the
  cube split and named it for a re-measure that no round took.

## r3

Lane: cloud, opus, read-only on code, 11-20 05:00-05:20 UTC. Brief briefs/audit-f-r3.md.
Tree: **P** = session head 79c8d2b5 (train 81; origin had nothing newer). Step 0: `git fetch --unshallow`; `cargo check -p chora`
green (the first try died in a rustup component race at toolchain install, the retry passed; not a code fault).
Pending MERGE-OK shas (trains 83-85) were measured with `git grep`/`git diff` from their merge base with P, not merged.
I built none of the slices measured here.

### 1. F1-F8 on P

| # | value on P | command | state | gap (one line) |
|---|---|---|---|---|
| F1 | reports/arch-plan-fable.md (unchanged since 11-17 14:53): 26 slice rows MV1-2, CUT1-2, NET1-4, WM1-WM7b, KIT1-KIT8, each with gate and cost; reviews review-arch-plan-risk.md, review-arch-plan-fit.md, plan-critique-1119.md present. (r2 printed 24 rows by the same grep; the plan file has not changed since, so r2 miscounted) | `grep -cE '^\| *(WM\|KIT\|NET\|CUT\|MV)' reports/arch-plan-fable.md` (26); `git log -- reports/arch-plan-fable.md` | **met** | none |
| F2 | tools/copies.toml 41 rows, all `max = 0`; `tools/cut-check`: `ok (8 crates, 18 reported)`, the 18 are assets, names, known; **0 copies lines**. Each look (mc, xp, tiling, construct) uses `book`, `actions`, `keysheet`, `notice`, `launch`, `settings`, `helper`, `wm::focus`, `wm::open` (>= 1 file each; tiling `notice` via the `use chora_kit::{..., notice}` block at main.rs:40). r2 risk gone: `fn report(` exists once outside core render helpers and voice's own method (kit runtime.rs:652); `fn on_path` only in the kit (desktop_entry.rs:188 pub, launcher.rs:222 private copy: RF2, see F4); `LC_MESSAGES` read only in kit words.rs; no `XDG_RUNTIME_DIR` read outside scene paths | `python3 tools/cut-check \| grep -ci copies` (0); `grep '^max' tools/copies.toml \| sort \| uniq -c` (41 x 0); per-part grep `chora_kit::(wm::)?<part>` or a `use` block per look | **met** | none on P. Pending: co-p2-fix ab4cbb4f adds kit `swallow` and moves tiling's swallow into it (F2 +); xp-p2r3 (running) makes xp the second user |
| F3 | `crates/chora-kit/src/wm/` = book, focus, give, kept, open, stack, state (+ mod.rs); records D99, D107-D109, D113 | `ls crates/chora-kit/src/wm` | **met** | WM4 desks, WM6 seams cut to next week (as r1, r2) |
| F4 | sizes: `ok: size-ratchet (0 files over 1500)`. Lints: `#[allow]`/`#[expect]` **360** (RC 378), without `reason` **150** (RC 238), `eprintln!` **125** (RC 135). Debt categories recounted item by item (section 2): workarounds **6** (RC 9), refactors **5** (RC 7), design questions 0 (RC 8), records 0 (RC 9), hygiene **14** (RC 14), total **25** (RC 47) | `tools/size-ratchet`; `grep -rE '#\[(allow\|expect)\(' crates plugins services --include=*.rs \| wc -l`; same `\| grep -vc reason`; `grep -rE 'eprintln!' crates/chora/src plugins services --include=*.rs \| wc -l`; per item the grep or sha of section 2 | **met** | hygiene sits at RC, not below (margin 0 until debt-f4 and voice-v12 merge: 12) |
| F5 | chora-kit lines **91.52 %** (9,674 lines, 820 missed; under 80 %: runtime.rs 72.6, launcher.rs 76.2, conn.rs 79.2). 55 modules added since fe8ba6d0 in the four looks and the kit (tests excluded): **0 under 70 %** (cube/construct.rs now 96.7 %, was 41.8 % in r2; kit cube.rs 99.2 %). Per look: mc 91.2, xp 89.9, tiling 88.2, construct **70.3 %**. llvm-cov run: 0 failed, rc 0. **Flakes: since r2 (11-19 20:10) 10 train runs went red on a flake**: 73b hud, 76 xp (rm-2), 78b doors, 79 xp, 82 cube queue, 82b tiling-look ink, 82c FLK-AG1, 82d FLK-IN1, 82e FLK-TR1, 82f FLK-HUD1. Open flake rows without a merged fix: FLK-AG1, FLK-IN1, FLK-TR1, FLK-HUD1 (flakes-k running), FLAKE-CUBE-QUEUE, tiling-look ink x2 (F23 family), co-p2-fix rw-3 tiling panel unit test, flakes-j "no helper word" (unexplained), doors Escape in tiling, mc, construct (FJ-R2, flakes-k item 5). FLAKE-DOORS xp and FLAKE-SIG are fixed on flakes-j 6192f696 (MERGE-OK, t84, not in P). flake-series 5/5 (7d9ec18e, 11-19) is void: the r1 bar is "5 of 5 and no train rerun on a flake from then to Sat" | `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features chora-kit/testing --summary-only`; `env -u XDG_RUNTIME_DIR cargo llvm-cov --no-fail-fast -p chora-mc -p chora-xp -p chora-tiling -p chora-construct -p chora-kit --features chora-kit/testing --summary-only` joined with `git diff --name-only --diff-filter=A fe8ba6d0 HEAD -- plugins/chora-{mc,xp,tiling,construct} crates/chora-kit` (tests excluded); `git log origin/claude/session-log --since="2026-11-19 20:10" --format='%ci %s' \| grep -iE 'red\|flake\|rerun'` | open | test half met; flake half: 10 flaky train reds in 9 h, at least 9 flake rows open; needs flakes-k + a second flake round + a new 5/5 series and zero flaky reruns to 03:59 |
| F6 | 73 first-parent commits on the session branch since 11-17 10:56; 72 are train merges, one is not: **9c6bf589** "test(voice): GR2 test takes the counted TempDir" (PM, 11-19 23:52, the train 78 semantic fix of backlog "HARNESS (train 78 red 23:51)"); it was inside the train 78/79 tree and reached origin with train 79's green gate (bb16f2ff, 01:20). `tests/cut-check-selftest.sh`: ok (plugin-to-plugin deps blocked) | `git log --first-parent --since="2026-11-17 10:56" --format='%h %s' origin/claude/autonomous-dev-session-3w-i33pg7 \| grep -v ' Merge '` (one line); `tests/cut-check-selftest.sh` | **met** | none for the measure (every pushed head went through a green gate); note: a PM fix commit in a train tree is a commit with no lane review; record it in the train log |
| F7 | `tests/acts/known-red.toml`: **0 rows** on P and on every pending sha. Tour v9 not this week (owner 11-17 14:20) | `grep -c '^\[\[red' tests/acts/known-red.toml`; same via `git show <sha>:` per pending sha | open | by design: v9 against v8 per look is next week; the state part holds |
| F8 | every module of `crates/chora-kit/src` (29 .rs, dirs actions, testing, wm) and the 7 wm parts has a line in CONTRIBUTING.md "Where shared code lives" (script: no output); CONTRIBUTING.md:17 now names `wm` (`book`, `focus`, `give`, `kept`, `open`, `stack`, `state`); records D96-D117; no `D-ungrab`; the only "proposed" hits are the historical D87-D94 preamble (decisions.md:126) | `for m in $(ls crates/chora-kit/src/*.rs \| xargs -n1 basename \| sed 's/\.rs$//'); do grep -q "^- \`$m\`" CONTRIBUTING.md \|\| echo missing $m; done` (only `lib`); `grep -n 'proposed\|Proposed' docs/decisions.md`; `grep -rn D-ungrab docs` (empty) | **met** on P | **at risk at train 85**: co-p2-fix ab4cbb4f adds `crates/chora-kit/src/swallow.rs` (226 lines, a kit API used by tiling and construct, xp next) with no CONTRIBUTING line and no record (`git grep swallow ab4cbb4f -- CONTRIBUTING.md docs/decisions.md`: empty); kit wm/mod.rs:12 "Desks follow in a later slice" is true (WM4 cut) |

**Met on P: 6 of 8** (F1, F2, F3, F4, F6, F8). F5 open on its flake half; F7 open by design. With trains 83-85 merged as they are: 5 of 8
(F8 drops on co-p2-fix) unless DOCS-F8e (rank 1) rides train 85.

### 2. F4 recount (categories of debt-2026-11-20.md, each item rechecked on P)

debt-f4 r1 kept workarounds at 13; that count carried four items that are closed on P. Proof per item:

| item | debt-f4 | P | proof |
|---|---|---|---|
| W7 xp ESCAPE_WAIT 15 ms x 4 | open | **closed** | 6b918eee (11-19 16:26, in P) removed it; `grep -rn ESCAPE_WAIT plugins/chora-xp` empty |
| W8 construct sleep before reading a dead child's tail | open | **closed** | 07eb024c "wait for the tail reader, not 150 ms"; the only `thread::sleep` left in plugins/chora-construct/src is a test (cube/start.rs:280) |
| N1-N4 known-red rows | open | **closed** | `tests/acts/known-red.toml` 0 rows (mc-i5, construct-red) |
| N6 FRICTION "timeout with tail of CHORA_LOG" | open | **closed** | tests/battle/FRICTION.md:55-60 "*Fixed:* ... no workaround is left" |
| B2, B3, B4, B6, W9, N5 | open | open | launch.rs:27 `GIVE_UP`; CHANGELOG.md:1 "Unreleased", no rc.4; 3 of 6 construct scripts gated only on debt-f4 (pending); F16-F19, F28 no fix; 640 shell sleep lines (RC 779); gate-construct-hud.sh:137 pointer move (co-29) |

Workarounds 6. Refactors: RF2 narrowed (two `on_path` in kit only), RF3 (voice client.rs:113 own `ConfirmParams`, 15 lines), RF7
(construct main.rs:237 `placed: bool`, xp window.rs:111 `held: bool`), N1 DB6, N2 `wm_with_window` (0 hits) open; N3
UnixListener row closed (copies row at 0) = 5. Hygiene: H2 (tokens.py still has no test; triage has tests_triage.py: narrowed),
H3, H4, H5 (8 "verify" lines in docs/known-issues.md), H6, H7, H9, H10 (narrowed, 820 ms), H11 (closed only on debt-f4 fc404a90),
H13, H14, DB7, DB8, DB9 = 14; H1, H8 (README.md:201 section), H12 (log dev-rules.md) closed.

### 3. Pending MERGE-OK shas: which F each moves

Values are the sha against its own merge base with P (`git grep` counts, `git diff --name-only`).

| sha | F moved | by how much |
|---|---|---|
| launcher-click 741d6bca | none of F1-F8 (RC shared bar sh-29) | tests only; adds the launcher-click cell (F6 coverage of the gate) |
| p2-harness 5dbd0ccf | none (RC 2 harness) | tests/persona only |
| voice-v12 ed4c7bf9 | F4 hygiene -1 (H4: voice-acp, voice-events now gate cells) | 14 -> 13 |
| xp-fix a767790d | F5 small (kit launch.rs, xp surfaces.rs with tests) | lints unchanged |
| debt-f4 fc404a90 | F4 hygiene -1 (H11 df, drift, shallow checks), B4 narrowed (3 of 6 construct scripts gated); F6 stronger (gate refuses a full disk, warns on drift) | 13 -> 12 with voice-v12 |
| harness-k 0dfdad8d | F6 (guards, grab-gap and mkdir-runtime checks in pre-commit) | gate-only |
| co-p2-fix ab4cbb4f | F2 + (one kit `swallow` for tiling and construct); **F8 - (new kit module, no CONTRIBUTING line, no record)** | F8 met -> open until DOCS-F8e |
| stray-grab 591e98f8 | F8 + (D-record amended for the press that chooses a manager; api.md) | records stay current |
| black-pane 91e6a114 | F4 eprintln +3 | 125 -> 128 |
| mc-p2-fix 17d9a798 | F4 eprintln +2 | contained in mc-remote |
| mc-remote dac835dd | F4 eprintln +3 (with mc-p2-fix); core ipc.rs change (api.md updated) | 128 -> 131 (RC 135, still met) |
| kit-tests 40b61f28 | F5 + (FakeCore cells wm give, kept, conn, state edges) | kit 92.02 -> 92.13 % (its own run, kit+looks) |
| tiling-bar 3a9a426f | none (RC look, tests) | - |
| kit-minors 7a7e00a2 | F5 nit (4 lines testing.rs doc) | - |
| flakes-j 6192f696 (t84) | F5 flake half: FLAKE-DOORS xp, FLAKE-SIG fixed | 2 of ~11 flake rows |

### 4. Split review candidates (review_due.py: six units over 5,000 lines)

1. **chora-construct (21,117 lines): highest risk.** It holds four open MAJORs (co-25 face part, co-26 verify, co-27, co-29) under
   rule 6, the lowest coverage of the four looks (70.3 % lines; 4,196 missed of 7,440 missed in all four looks, 56 %), and three
   lanes edit it at once (co-majors r3; xp-p2r3 on camera.rs and cube.rs; co-p2-fix pending on camera.rs, cube.rs, input.rs, main.rs).
   The faults sit where key routing (input.rs, camera.rs: swallow, Tab after a flight, co-29), pane ownership on an overlay start
   (co-27) and the cube's child lifecycle (cube.rs 1,817 lines by `wc -l`) meet; a split review should map those three seams and
   say which state each owns before the next MAJOR fix lands on top.
2. **chora-xp (23,730 lines, the largest): second.** Most churn since 11-17 (76 commits, +8,131/-4,947 over 45 files: the wm.rs
   split into wm/*), and it now becomes the face manager inside the construct (xp-p2r3: swallow on a face; XPF-R1 launch tokens),
   so its key and launch paths gain a second host. Coverage is good (89.9 %), so the risk is structure, not tests: startmenu.rs
   1,764 and taskbar.rs 1,341 lines (`wc -l`) and the wm/* module borders after the split have had no whole-unit review.
   (mc is third: main.rs 2,511 lines and two pending branches on main.rs, acts.rs, plus a core ipc.rs change in mc-remote.)

### 5. Slices by Sat 03:59 UTC

Ranked by F moved per % budget. No new feature. Running lanes: verify-maj-r2 (tests/ launcher-click, xp-fields), flakes-k (agent.sh,
chora inject/ipc tests, images.rs, gate-hud, tests/doors), core-minors (crates/chora ipc, state, input, grabs; api.md; decisions.md
D116), xp-p2r3 (xp, kit launch.rs, construct camera.rs ~476, cube.rs ~156, gate-construct-cube.sh, persona), co-majors-r3
(construct overlay, input; gate-construct-k2a, windows, hud, 3d; merge-gate.sh construct-extra).

| rank | name | F | files | done criterion | model | cost | conflicts |
|---|---|---|---|---|---|---|---|
| 1 | DOCS-F8e | F8 (keeps it met) | CONTRIBUTING.md (a `- \`swallow\`` line; summary line 17), docs/decisions.md (D118: one list of swallowed chords in kit, who grabs it: tiling, construct pane, xp face; the `Alt` exception), crates/chora-kit/src/swallow.rs nothing | from ab4cbb4f (or the t85 head): the F8 script of section 1 prints no missing module; `grep -n swallow docs/decisions.md CONTRIBUTING.md` hits both; a reviewer who did not write it checks D118 against swallow.rs and the xp-p2r3 use | sonnet | 0.2 % | core-minors edits decisions.md (D116 text): append D118 at the end, trivial; ride train 85 with co-p2-fix or the next one |
| 2 | FLAKES-L r2 (the flakes no lane owns) | F5 | tests/gate-tiling-look.sh (ink: wait for the first drawn focus border, no retry), plugins/chora-tiling/src/panel tests (rw-3 `the_layout_word_is_a_word_of_a_person`), tests/doors/xp.sh "no helper word" (evidence only); FLAKE-CUBE-QUEUE (construct cube turn queue, gate-construct-cube.sh) only after xp-p2r3 merges | each: cause with evidence under `--jobs 2` load, red first, fix, then 10 loaded runs of the cell or test; a row not reproduced in 20 loaded runs says so with the count; backlog rows closed or named | sonnet high | 1 % | flakes-k (none of these files); xp-p2r3 owns gate-construct-cube.sh: cube-queue item last, from the head after its merge; tiling-look also in verify-maj-r2's gate list (read only there) |
| 3 | FLAKE-SERIES r2 | F5 (flake half) | none (gate runs); backlog rows | after flakes-k and rank 2 are on the head (Fri ~20:00): 5 full `tests/merge-gate.sh --jobs 2` runs (the train setting since t83) on that head, no rerun, 5 of 5 green; each red a backlog row with step, cell, log. F5 flake half met only with 5/5 and no train rerun on a flake from the series' start to 03:59 (PM logs each train's first-run result) | haiku | 1 % | none; own container; it must run on the head that the last trains build on |
| 4 | AUDIT-F r4 (Sat check) | F2, F4, F5, F8 re-measure | reports/audit-f.md "## r4" (log branch) | on the last head (Sat ~02:00, after trains 83-86): the commands of section 1 for F2, F4 (lints: eprintln must stay <= 135 after black-pane and mc-remote; hygiene with debt-f4 and voice-v12 in), F5 (series result plus the train log), F8 (swallow line); one verdict per F with value, command, check | sonnet | 0.6 % | none (read-only); after the last train |

Sum 2.8 %. Per F: F8 0.2 %, F5 2 %, re-check 0.6 %. Nothing moves F7 this week (tour v9 next week; known-red at 0 on P and every
pending sha). F1, F3, F6 need no slice. If budget is short, drop rank 2's cube-queue item first, then rank 2 entirely; rank 3 without
rank 2 is likely red again (the ink and panel flakes have hit trains twice and once). Best use of budget beyond these for stability
and looks: the construct split review of section 4 (it carries the open MAJORs and the lowest coverage).

Honest outlook for F5: 10 flaky train reds in the 9 h since r2, at --jobs 3. Met by Sat needs flakes-k to close its four, rank 2
to close or fence the rest, and then about 8 h of trains with no flaky rerun. Likely outcome: test half met, flake half open.

### 6. Notes

- debt-f4 r1 counted W7, W8, N1-N4 and N6 open from the 11-20 debt list without rechecking; the PM's lint erratum (03:42) was right.
  Next debt count: recheck each item on the tree, as section 2 does.
- r2's F1 count (24) was a grep miscount; the plan file did not change after 11-17 14:53.
- Pending eprintln growth (+6 over trains 84-85) keeps F4 met (131 <= 135) but eats the margin; rank 4 re-measures it.

## r4

Lane: cloud, opus, read-only on code, 11-20 14:15-14:45 UTC. Brief briefs/audit-f-r4.md.
Tree: **P** = session head d81c8293 (train 87e; origin had nothing newer at 14:40). Step 0: `git fetch --unshallow`; `cargo check -p chora`
green. Pending MERGE-OK shas co-r5 c2b0310f, probe-leak f5ac3500, co-majors-r4 e0c3a266, lint-down 78ec62e1 measured with
`git diff`/`git show` from their merge base with P, not merged. flakes-l and flakes-m have pushed no commit yet (both branch tips are
their start shas) and no report; mc-fix is at a8cd0de0 (r4, under review). I built none of the slices measured here.

### 1. F4 on P (each bar against its RC value)

| bar | value on P (RC) | with pending MERGE-OK | command | state |
|---|---|---|---|---|
| files over 1,500 production lines | **0** (RC: xp wm.rs, AR7) | 0 | `tools/size-ratchet` → `ok: size-ratchet (0 files over 1500)` | met |
| `#[allow]`/`#[expect]` | **360** (378) | 359 (lint-down drops testing.rs's stale allow) | r3 grep `grep -rE '#\[(allow\|expect)\(' crates plugins services --include=*.rs \| wc -l`; `tools/lint-ratchet` of 78ec62e1 with its list on P: `FAIL: allow is 360, the list holds 359` (expected: the list already counts lint-down's -1) | met |
| without reason | **150** by lines (238 by lines); **0** by attribute (lint-ratchet) | 0 | r3 grep `\| grep -vc reason`; the tool reads each attribute to its `)]` | met |
| `eprintln!` | **131** (135) | 131 (every pending sha adds 0; lint-down r3 delta table) | `grep -rE 'eprintln!' crates/chora/src plugins services --include=*.rs \| wc -l` | met, margin 4 (was 10 in r3: black-pane +3, mc-p2-fix/mc-remote +3 landed in t85) |
| workarounds | **6** (9): B2 `GIVE_UP` launch.rs:27; B3 CHANGELOG.md:1 `## Unreleased`, 0 hits rc.4; B4 gate-construct-k2a, -3d, -windows in no gate line (merge-gate.sh:245 runs k1, mcpane, nested; k2a only in the comment); B6 F16-F19/F28 no fix commit; W9 698 `sleep` lines in tests/*.sh (r3 640 by another regex; RC 779); N5 gate-construct-hud.sh:22 pointer move | **5**: co-r5 c2b0310f adds k2a, windows and 3d to `construct-extra` (B4 closed) | per item grep as listed | met |
| refactors | **5** (7): RF2 two `on_path` in kit (desktop_entry.rs:188, launcher.rs:290); RF3 voice client.rs:113 `ConfirmParams`; RF7 construct main.rs:237 `placed: bool`, xp window.rs:111 `held: bool`; N1 DB6; N2 `wm_with_window` 0 hits | 5 | `grep -rn 'fn on_path\|struct ConfirmParams\|wm_with_window'` | met |
| design questions, records | **0, 0** (8, 9) | 0, 0 | as r3 (D1-D8 records; D96-D118) | met |
| hygiene | **12** (14): open H2 (tokens.py no test; only tests_triage.py), H3 (3 construct scripts of B4), H5 (9 "verify" lines in docs/known-issues.md), H6, H7 (CHANGELOG), H9, H10 (gate-mc-handover.sh:35 820 ms, narrowed), H13, H14, DB7, DB8, DB9. **Closed since r3**: H4 (merge-gate.sh:258 `voice-acp`, and `voice-events` cells; voice-v12 in t84/85), H11 (merge-gate.sh:83-86, :323 df 6 GB, shallow and drift checks; debt-f4 in t85) | **11** (co-r5 closes H3) | per item grep; `git ls-tree -r --name-only origin/claude/session-log \| grep -i test.*tokens` (empty) | met (r3's gap "hygiene at RC, margin 0" is gone: 12 < 14) |
| total debt | **25** (47) | 23 | sum | met |

**F4: met** on P, every count below RC. Gap: none. Risk: eprintln margin 4; the ratchet (lint-down, t90) freezes it at 131.

### 2. F6 on P

24 first-parent commits since 79c8d2b5, **all merges**; 0 direct commits (`git log --first-parent --format='%h %p %s' 79c8d2b5..P \| grep -v ' Merge '`: empty).
Each second parent is the MERGE-OK sha named in state.md for its train: t83-t85c launcher-click 741d6bca, p2-harness 5dbd0ccf,
voice-v12 ed4c7bf9, xp-fix a767790d, debt-f4 fc404a90, harness-k 0dfdad8d, flakes-j 6192f696, co-p2-fix ab4cbb4f, stray-grab 591e98f8,
black-pane 91e6a114, mc-p2-fix 17d9a798, mc-remote dac835dd, kit-tests 40b61f28, tiling-bar 3a9a426f, kit-minors 7a7e00a2,
core-minors d23b03ae, docs-f8e c529242b, xp-p2r3 f89f785c (head 9973d0eb, green t85c 08:38); t86 harness-l 75dfa4b9 (adf71956, green
09:27); t87e mc-fix cb2b80f1, xp-24 2d1e06c6, core-l b1d1491d, flakes-k e2508ec1, xp-36 3290f1d4 (d81c8293, green 12:50 at --jobs 1).
Merge content beyond the two parents (`git show --remerge-diff`): only the PM's conflict resolutions in tests/merge-gate.sh (launcher-click,
voice-v12, debt-f4, harness-k, tiling-bar), tests/persona/README.md (xp-fix) and the two tiling scripts plus merge-gate.sh (black-pane,
the t85 -> t85b re-resolution); all inside a train tree that went green. Three heads reached origin (9973d0eb, adf71956, d81c8293), each
after its train's green in the state.md log; the session branch stayed at 79c8d2b5 through the red t84/t85/t85b. The origin reflog is not
readable from a lane (the local reflog has only this fetch), so push order rests on state.md times. Cut gate: `tests/cut-check-selftest.sh`
→ `ok: cut-check-selftest`; `tools/cut-check` → `ok: cut-check (8 crates, 18 reported)`, 0 copies lines.
**F6: met.** Note: the PM's semantic fix commit of r3 (9c6bf589) has no successor; conflict resolutions are the only PM-authored lines.

### 3. F5 on P

Test half: chora-kit **91.87 %** lines (9,955 lines, 809 missed) `env -u XDG_RUNTIME_DIR cargo llvm-cov -p chora-kit --features chora-kit/testing --summary-only`, rc 0. Met (r3's per-look and moved-module figures not rerun; no module moved since).

Flake half: **open flake rows with no merged fix on P: 14**, 12 if the pending MERGE-OK land (sh-30 by probe-leak f5ac3500; K5 3D
wobble race by co-r5 c2b0310f).
- Owned by flakes-l r2 (no commit yet): tiling-look ink@2 (t84, t87d; 5 hits in all), FLK-ACT1 xp mixer read, FLK-ACT1 persona sound I5
  overlap (mc-fix a8cd0de0 also changes I5 counting: 5c422e10/6508d95d; overlap with flakes-l item 6), FLK-HUD1 (flakes-k: seen 1 of 20,
  cause not named), FLK-HUD2, FLK-LAY1, FLK-TA1 = 7.
- Owned by flakes-m r1 (no commit yet): FLAKE-CUBE-QUEUE, FJ-R2 construct Escape (tiling and mc fixed in flakes-k), xp "no helper word" = 3.
- **No owner: 2**: co-p2-fix rw-3 tiling `panel::the_layout_word_is_a_word_of_a_person` (unit test, red once in a full run); MX-CAP
  gate-handover-matrix cap 315 s (t85b red at 320 s; t88 red at the cap with the leaked probes of sh-30: after probe-leak it may hold, unmeasured).
- Legacy rows (debt B6/H14): F22, F24, F26, F28 have no fix commit and are unreproduced (not closed); F23, F29 belong to ink@2/TA1;
  F27 is launcher-click.
- Not a flake but a red-by-host: launcher-click red 10 of 10 with fuzzel 1.9.2 (every lane container); green with 1.11.1 (PM PATH).
  GATE-SILENT (a cell that does not run must show SKIP) and MC-47-CELL are open: a lane-run series is 0/5 by construction.

Train record since r3 (05:20): t84 (ink@2), t85 (merge fault, not a flake), t85b (MX-CAP), t87 (FLK-LAY1), t87b (FLK-HUD2), t87c (K5,
real test race), t87d (ink@2), t88 (sh-30 probes, matrix cap): **6 flaky reds at --jobs 2**; the only greens since t86 are t87e at
**--jobs 1**. flake-series r2: 0 of 5 at --jobs 3 (3 launcher-click/host, 1 ACT1, 1 TA1).
Greens at --jobs 1 hide flakes: every hit above is a load race (HUD timeouts at load 5-7, ink at load 5-6), and --jobs 1 removes the
load; state.md already says they do not count. **The bar needs --jobs 2** (the train setting from t83 that the flakes were found at, and
the setting of the r3 series slice): 5 of 5 full `tests/merge-gate.sh --jobs 2` on the final head with fuzzel >= 1.11 on PATH, and
no train red on a flake from the series' start to 03:59 (trains at --jobs 2 again).
**F5: open** (flake half). Gap: 14 open rows, 0 greens at --jobs 2 since t86, no series.

### 4. F1-F3, F8 on P

| # | value | command | state |
|---|---|---|---|
| F1 | 26 slice rows, plan unchanged since 11-17 14:53 | `git show origin/claude/session-log:reports/arch-plan-fable.md \| grep -cE '^\| *(WM\|KIT\|NET\|CUT\|MV)'`; `git log -1 -- reports/arch-plan-fable.md` | met |
| F2 | 0 copies lines; 42 rows all `max = 0` (41 in r3 + swallow); each look uses book, keysheet, notice (grep of `chora_kit` users per look: mc book manage.rs:160, main.rs:306); `grab_swallowed` in xp, tiling, construct (mc has no face manager) | `python3 tools/cut-check \| grep -ci copies` (0); `grep '^max' tools/copies.toml \| sort \| uniq -c` | met |
| F3 | kit wm/ = book, focus, give, kept, open, stack, state | `ls crates/chora-kit/src/wm` | met |
| F8 | every kit module has a CONTRIBUTING line (only `lib` missing, by design); CONTRIBUTING.md:44 `swallow`, decisions D118 (docs-f8e c529242b in t85): r3's risk is gone | r3 for-loop; `grep -n swallow CONTRIBUTING.md docs/decisions.md` | met |

**Met on P: 6 of 8** (F1, F2, F3, F4, F6, F8). Open: F5 (flake half), F7 (by design, next week; known-red 0 rows).

### 5. Slices by Sat 03:59 UTC (13.5 h)

F4 needs no slice. Ranked by F moved per % budget; no new feature. Running: mc-fix r4 (mc, acts I5), flakes-l r2 (tiling-look, acts,
hud, layout, tiling-audit), flakes-m r1 (cube, doors, chat-keys).

| rank | name | F | files | done criterion | model | cost | conflicts |
|---|---|---|---|---|---|---|---|
| 1 | FLAKE-SERIES r3 | F5 (the only way to met) | none (gate runs); backlog rows | on the head after t89 (probe-leak), t90 (co-majors-r4, lint-down) and the flakes-l, flakes-m trains (~Sat 00:00): 5 full `tests/merge-gate.sh --jobs 2` in a row, no rerun, with fuzzel 1.11.1 on PATH (PM host, or rank 3 first); 5 of 5 green; each red a backlog row with cell, line, log. Then the PM logs each train's first-run result to 03:59 | haiku | 1 % | none; must run on the head the last trains build on; a lane container fails launcher-click by host unless rank 3 is in |
| 2 | FLAKES-N r1 (the two unowned) | F5 | plugins/chora-tiling/src/panel tests (rw-3); tests/gate-handover-matrix.sh cap (MX-CAP) | rw-3: red count of 20 loaded runs of the chora-tiling lib before, cause, fix, 0 of 20 after (or "not reproduced in 20", named); MX-CAP: 5 timed runs on the head with probe-leak, cap = slowest + 25 % per its comment, red-first selftest of the cap rule | sonnet high | 0.5 % | flakes-l owns tests/gate-tiling-look.sh, not the panel unit tests; merge-gate.sh lines touched by probe-leak and lint-down: start from the t90 head |
| 3 | GATE-SILENT (fuzzel guard) | F5 (lane series and lane gates stop being red by host), F6 evidence | tests/merge-gate.sh summary (a cell that does not run prints SKIP with its reason); launcher-click and the fuzzel cells: skip with SKIP below fuzzel 1.11 (MC-47-CELL's guard) | red selftest first: a cell whose tool is missing prints no SKIP; after: SKIP line in the summary, gate exit 0 only when every skip is a named host gap; PM host (1.11.1) still runs the cells | sonnet | 0.3 % | mc-fix (MC-47-CELL patch, mc-sofa cell) and probe-leak/lint-down on merge-gate.sh: after t90 |

Sum 1.8 %. Order: rank 2 and 3 now (both ~2 h), rank 1 after the last flake trains. If flakes-l or flakes-m come back with rows "not
reproduced", rank 1 is the test of them. Honest outlook: F5 met needs flakes-l (7 rows) and flakes-m (3) to land by ~22:00, trains back
at --jobs 2 green, and a 5/5 series by ~03:00. Likely: test half met, flake half open at 03:59; F5 then goes to next week with the rows named.

### 6. Notes

- r3 counted hygiene 14 with H4 and H11 open; both closed on P by merged shas (voice-v12, debt-f4), so 12 is a recount, not new work.
- r3's workaround W9 count (640) and this one (698, `grep -h -E '(^|[^a-z_])sleep ' tests/*.sh`) use different regexes; neither is a bar.
- eprintln rose 125 -> 131 since r3 (t85 merges); still under RC 135; lint-down's list (131) freezes it.
