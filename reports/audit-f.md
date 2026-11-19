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
