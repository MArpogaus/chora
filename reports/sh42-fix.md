# sh42-fix

## r1

Lane: cloud, branch claude/sh42-fix-r1 from c3c37c2e (session head),
now b92de049. Step 0 green (unshallow, `cargo check --workspace`).
Brief: briefs/sh42-fix-r1.md; repro reports/sh42-repro.md "## r2".

### Root cause

Nobody but the joining look clears the core focus: chora-tiling itself
sends `focus: null` in its first transaction.

1. `plugins/chora-tiling/src/main.rs:641` `start()` calls `relayout()`.
   With no window of its own, `wm.focused()` is `None`, and
   `plugins/chora-tiling/src/windows.rs:109` asks `focus.sends(None)`.
2. `crates/chora-kit/src/wm/focus.rs:167` `Focus::sends` was
   `!lost && (!sent || surface != focus)`. A fresh `Focus` has
   `sent == false`, so it said yes to an empty focus that the look never
   held, and `windows.rs:110` pushed `Command::Focus { surface: None }`.
   The core applied it (the plugin is on the main socket, no confinement)
   and told `focus.changed(null)`: the foot of xp or mc lost the keys.
   This is the join rule of D119 broken in the kit, not in the core seat:
   mc joining beside xp (control) starts with `take_free_keyboard`, which
   first checks for a focused node, so it never sends an empty focus.
3. Why the holders differ: xp keeps its own holder and still says "xp";
   mc uses the kit `Focus`, where `moved(None)` (`focus.rs:120-123`)
   stores no core surface and sets no loss ("an empty focus is no
   loss"), so `held` answers `Nobody` and `state.get` shows `null`.
4. Why a click in mc gave nothing back: `plugins/chora-mc/src/manage.rs:218`
   `click` called `focus.take()`, which only resets `sent` when `lost` is
   set; `lost` stays false after an empty focus, `sent` is still true for
   s1, so `sends(Some("s1"))` (`manage.rs:219`) is false and mc sends
   nothing. tiling had the same gap in `pointer_focus` and closed it in
   handover-ti r2 with `lost_to_other` + `reclaim`; mc's click did not.
5. Why Return gives nothing back: with no focused surface and no focused
   node, the key reaches no client. mc takes no free keyboard beside
   another look on purpose (D119;
   `plugins/chora-mc/src/main.rs:1573`
   `a_page_beside_other_window_managers_takes_no_keyboard`). With fix 1
   the join no longer drops the keys, so the Return case does not arise
   at the join; after a drop by another client the way back is the click
   (fix 2). Making the page grab a free keyboard beside another look
   would undo D119, so it is not done.

### Commits

| Item | Red | Green | Test | Cell |
|---|---|---|---|---|
| gate cells | a04b395e, 68a4ff1b | 7a110c34, b92de049 | - | tests/gate-handover-shared.sh `join-xp`, `join-mc`, `mc-back` |
| fix 1, joining look sends no empty focus (kit) | db469100 | 7a110c34 | `wm_focus::a_joining_look_sends_no_empty_focus` | `join-xp`, `join-mc` |
| fix 2, mc click gives the keys back | db469100 | b92de049 | mc `tests::a_click_on_its_window_after_another_client_dropped_the_keyboard_sends_the_focus` | `mc-back` |

Red at a04b395e (on c3c37c2e code): `FAIL: join-xp: tiling joined and
the text does not reach s1 (core focus [], xp {"holder":"xp"}, tiling
{"holder":null})`; mc cell alone (scratch copy): `FAIL: join-mc: ...
(core focus [], mc {"holder":null}, tiling {"holder":null})`; mc-back
(68a4ff1b): `FAIL: mc-back: another client dropped the focus and a
click on s1 at 416,388 does not give it the keys back (mc
{"holder":null})`. The first version of mc-back also asked Return; it
was red for the reason in point 5 and was narrowed to the click in
68a4ff1b, a test-only commit before any fix.

Fix 1 (7a110c34): `Focus::sends` answers no for an empty focus when the
look has no sent surface (`drops_nothing`); a look that sent a window
still drops it with an empty focus (its window exited, no next one).
Fix 2 (b92de049): mc `click` on its own window calls `focus.reclaim()`
when `focus.lost_to_other(NAME)`, as tiling's `pointer_focus` does;
the plain `take()` is gone (reclaim covers `lost`).

Cells (tests/gate-handover-shared.sh, matrix row, fresh session each):
`join-xp` foot in xp, typed text reaches its file, tiling joins with
`--no-companions`, the next text reaches the file and the core still
focuses s1; `join-mc` the same with a foot on the page of mc after
Return; `mc-back` the agent drops the focus (`apply focus null`), a
click on the page window gives it the keys back. New helper
`end_session`; the cells reuse `focused`, `key`, `click`, `until_ok`.

### Repro table rerun (media/2026-11-24/sh42-repro/run.sh, unchanged, at b92de049)

| Path | runs | core focus after | look holder after | typed text lost |
|---|---|---|---|---|
| xp window, tiling joins | 5 | s1 5/5 | xp="xp", tiling=null | **0/5** |
| mc page window, tiling joins | 5 | s1 5/5 | mc="mc", tiling=null | **0/5** |
| control: xp window, mc joins | 3 | s1 3/3 | xp="xp", mc=null | 0/3 |
| control: xp, no window, tiling joins | 3 | none | xp="xp", tiling=null | n/a (no window) |
| control: mc, no window, tiling joins | 3 | none | mc="mc", tiling=null | n/a (no window) |

Raw lines: media/2026-11-24/sh42-fix-r1/results.txt. Before (r2 of the
repro): lost 5/5 xp, 5/5 mc; mc holder null after the join, now "mc".

### Gate

Full `tests/merge-gate.sh --jobs 1` at b92de049 (no --only). The run
was stopped by the 2 h limit of a background task of this lane while
`keys` ran, after 41 steps, all PASS (no step failed). The remaining 24
steps then ran in two runs with `--only` (build first, same sha), all
PASS. Together they cover every step of `--list` except known-red ones.

```
red-first  PASS     1s /tmp/merge-gate.veMYku/red-first.log
identity   PASS     0s /tmp/merge-gate.veMYku/identity.log
pre-commit PASS   103s /tmp/merge-gate.veMYku/pre-commit.log
gate-lib   PASS   178s /tmp/merge-gate.veMYku/gate-lib.log
ci-check   PASS   112s /tmp/merge-gate.veMYku/ci-check.log
build      PASS    30s /tmp/merge-gate.veMYku/build.log
test       PASS   168s /tmp/merge-gate.veMYku/test.log
clippy     PASS    24s /tmp/merge-gate.veMYku/clippy.log
2d-guard   PASS   143s /tmp/merge-gate.veMYku/2d-guard.log
package-deps PASS     1s /tmp/merge-gate.veMYku/package-deps.log
package-lists PASS     1s /tmp/merge-gate.veMYku/package-lists.log
xp         PASS    98s /tmp/merge-gate.veMYku/xp.log
matrix     PASS   334s /tmp/merge-gate.veMYku/matrix.log
virtual    PASS    48s /tmp/merge-gate.veMYku/virtual.log
xp-faces   PASS    14s /tmp/merge-gate.veMYku/xp-faces.log
xp-paperclip PASS     6s /tmp/merge-gate.veMYku/xp-paperclip.log
xp-fields  PASS    65s /tmp/merge-gate.veMYku/xp-fields.log
xp-readable PASS    41s /tmp/merge-gate.veMYku/xp-readable.log
xp-question PASS    10s /tmp/merge-gate.veMYku/xp-question.log
xp-chat-idle PASS     4s /tmp/merge-gate.veMYku/xp-chat-idle.log
xp-helper  PASS    31s /tmp/merge-gate.veMYku/xp-helper.log
xp-notepad PASS    51s /tmp/merge-gate.veMYku/xp-notepad.log
hud        PASS    79s /tmp/merge-gate.veMYku/hud.log
layout     PASS  1371s /tmp/merge-gate.veMYku/layout.log
tiling-audit PASS   267s /tmp/merge-gate.veMYku/tiling-audit.log
tiling-look PASS   733s /tmp/merge-gate.veMYku/tiling-look.log
tiling-empty PASS    16s /tmp/merge-gate.veMYku/tiling-empty.log
chat-keys  PASS     7s /tmp/merge-gate.veMYku/chat-keys.log
doors      PASS   384s /tmp/merge-gate.veMYku/doors.log
acts       PASS  1588s /tmp/merge-gate.veMYku/acts.log
last-words PASS    13s /tmp/merge-gate.veMYku/last-words.log
cube       PASS   152s /tmp/merge-gate.veMYku/cube.log
pull       PASS   302s /tmp/merge-gate.veMYku/pull.log
placed     PASS    11s /tmp/merge-gate.veMYku/placed.log
construct-hud PASS   574s /tmp/merge-gate.veMYku/construct-hud.log
construct-extra PASS   162s /tmp/merge-gate.veMYku/construct-extra.log
out        PASS    12s /tmp/merge-gate.veMYku/out.log
winit      PASS     8s /tmp/merge-gate.veMYku/winit.log
wm-pane    PASS    20s /tmp/merge-gate.veMYku/wm-pane.log
pane-keys  PASS     7s /tmp/merge-gate.veMYku/pane-keys.log
walk       PASS     3s /tmp/merge-gate.veMYku/walk.log
(run 1, /tmp/merge-gate.veMYku: stopped by the task limit during keys)
build      PASS     5s /tmp/merge-gate.chaC2Z/build.log
keys       PASS    37s /tmp/merge-gate.chaC2Z/keys.log
pixels     PASS    59s /tmp/merge-gate.chaC2Z/pixels.log
mc-paths   PASS   121s /tmp/merge-gate.chaC2Z/mc-paths.log
mc-sofa    PASS    47s /tmp/merge-gate.chaC2Z/mc-sofa.log
mc-column  PASS    49s /tmp/merge-gate.chaC2Z/mc-column.log
mc-handover PASS    10s /tmp/merge-gate.chaC2Z/mc-handover.log
launcher-click PASS    18s /tmp/merge-gate.chaC2Z/launcher-click.log
voice-acp  PASS    12s /tmp/merge-gate.chaC2Z/voice-acp.log
voice-events PASS     2s /tmp/merge-gate.chaC2Z/voice-events.log
agent      PASS    38s /tmp/merge-gate.chaC2Z/agent.log
tiling-keys-more PASS    41s /tmp/merge-gate.chaC2Z/tiling-keys-more.log
tiling-windows PASS   170s /tmp/merge-gate.chaC2Z/tiling-windows.log
(run 2, /tmp/merge-gate.chaC2Z, --only build,keys..tiling-windows)
build      PASS     0s /tmp/merge-gate.KjdzWF/build.log
tiling-system PASS    15s /tmp/merge-gate.KjdzWF/tiling-system.log
handover   PASS    26s /tmp/merge-gate.KjdzWF/handover.log
mc-scale   PASS    14s /tmp/merge-gate.KjdzWF/mc-scale.log
watchdog-starved PASS    49s /tmp/merge-gate.KjdzWF/watchdog-starved.log
leak       PASS   469s /tmp/merge-gate.KjdzWF/leak.log
xp-run     PASS    13s /tmp/merge-gate.KjdzWF/xp-run.log
demo       PASS   243s /tmp/merge-gate.KjdzWF/demo.log
tiling-bar PASS   188s /tmp/merge-gate.KjdzWF/tiling-bar.log
tiling-theme PASS    18s /tmp/merge-gate.KjdzWF/tiling-theme.log
tiling-disc PASS    74s /tmp/merge-gate.KjdzWF/tiling-disc.log
mc-host    PASS    19s /tmp/merge-gate.KjdzWF/mc-host.log
e2e        PASS   283s /tmp/merge-gate.KjdzWF/e2e.log
(run 3, /tmp/merge-gate.KjdzWF, --only build,tiling-system..e2e)
```

Not done / left as seen: xp says holder "xp" while the core focuses no
surface (seen in the no-window control, before and after); not touched,
out of scope (no keys lost there). `pgrep -fa 'chora --headless'` empty
at the end.

## Review r1

Lane: cloud, read-only, claude/sh42-fix-r1 head b92de049, base c3c37c2e
(5 files, +116 -3). Step 0 green (unshallow, `cargo check --workspace`).
`cargo test -p chora-kit`, `-p chora-mc`, `-p chora-tiling`: all ok, 0 failed.

Checks:
1. Red first: a04b395e, 68a4ff1b (gate cells) and db469100 (kit and mc
   unit tests, tests only) come before 7a110c34 (kit) and b92de049 (mc).
   Both unit tests fail on the base logic: a fresh `Focus` answered
   `sends(None)` yes; the old `take()` reset nothing after an empty focus.
2. Kit `Focus::sends` (focus.rs:167): callers are tiling windows.rs:109
   (relayout) and mc manage.rs:221 (always `Some`); xp and construct do
   not call it. The new rule only differs when `surface` is None, `lost`
   is false and `sent` is false (a look that never sent, or that reclaimed
   after `gave`/`sent(None)`). A look that sent a window keeps `surface`,
   so a closed window, an empty tag or an emptied layout still sends
   `focus: null`. A look switch goes through `gave` (lost). A window that
   the core focused without the look sets `lost` by `moved`, unchanged.
   No stale-focus path found. Side gain: `turn()` after `gave` no longer
   sends a null that took the keys from the window just handed over.
3. mc click (manage.rs:218): after an empty focus, `lost_to_other` is true
   (`sent`, `surface` s1, `core` None), `reclaim` and one send. The normal
   case (s1 sent and focused) sends nothing; s2 sends once. D119 stays:
   no free-keyboard path changed, `a_page_beside_other_window_managers_takes_no_keyboard`
   green. New: a click on an mc window also reclaims from a foreign node
   (`taken_by_other`), as tiling's `pointer_focus` does (rw-1); POLS-sound.
4. Return deviation: sound. With fix 1 the join drops no keys (cells
   join-xp, join-mc; table 0/5), so a user in mc after a tiling join does
   not type into nothing. The no-holder state is now reached only when
   another client drops the focus (or a look drops its own last window,
   as before this branch); a click recovers it. Making Return grab a free
   keyboard beside another look would undo D119.
5. Gate: the new cells run in the `matrix` step, which passed in the full
   run 1 (334 s). No cell found whose result depends on order: `mc-back`
   reuses the `join-mc` session on purpose (written in the header); each
   join cell starts a fresh session after `end_session`.
6. gate-handover-shared.sh: join-xp and join-mc assert the lost-key case
   (text reaches the foot after the join, core still focuses it), mc-back
   the click back. Two fixed `sleep 2` added (sleep-ratchet 777 -> 779 of
   the listed 779: within the list, but they spend its slack): below.

Findings:
- rsf1-1 MINOR tests/gate-handover-shared.sh:182 `join_tiling` waits a fixed `sleep 2` for tiling's first transaction; a slower first relayout lets the `focus: null` land after the typed text, a false green, and it spends 1 of the 2 counts of ratchet slack. Fix: wait until `ctl state tiling` answers (its first transaction is applied) before typing, no fixed sleep.
- rsf1-2 MINOR tests/gate-handover-shared.sh:219 `join-mc` waits a fixed `sleep 2` before `key Return`; a slow mc eats the Return and `focused 10` fails (flake path). Fix: retry `key Return` inside `until_ok 'focused 1'`, no fixed sleep.
- rsf1-3 MINOR plugins/chora-mc/src/manage.rs:218 the new `lost_to_other` also reclaims when a node of another client holds the keys (`taken_by_other`), a behaviour change with no unit test (db469100 covers only the empty focus). Fix: add an mc unit test where a `focus.node` of another owner holds the keys and a click sends the window focus once.

Counts: BLOCK 0, MAJOR 0, MINOR 3.
Verdict: MERGE-OK.

## r2

Head 88035ec0 (claude/sh42-fix-r2; merged 84f66d43 first). The two fixed `sleep 2` of r1 in tests/gate-handover-shared.sh are gone: join_tiling waits until `ctl state tiling` lists a desk, join-mc waits until `ctl state mc` lists a window, both with until_ok. rsf1-3: unit test `a_click_reclaims_the_keyboard_from_a_foreign_node` in plugins/chora-mc/src/manage.rs. A first try that waited for tiling to list the foot window failed (tiling joins beside it and does not list it), so it waits on the desk.

Ratchets (sleep list lowered 779 to 778, others unchanged):
- ok: sleep-ratchet (sleep 778, sleep-all 929)
- ok: size-ratchet (0 files over 1500)
- ok: lint-ratchet (allow 360, no-reason 0, eprintln 46)

Handover, tests/gate-handover-shared.sh, 10 runs: 1 to 10 all green (10/10).

Full tests/merge-gate.sh (no --only): all steps passed, run dir /tmp/merge-gate.sjMJgM (an earlier run, /tmp/merge-gate.GCyfiD, died with a worker restart at tiling-audit).
