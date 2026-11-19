# debt-down

## r1

Code: claude/debt-down at 10 commits over train 65 (85764974), `git log 85764974..claude/debt-down`.
Brief: briefs/debt-down-r1.md. F4 target: every debt category and the lint lines at or below RC.

### Table before and after

| line | RC | before (11-20) | after | at or below RC |
|------|----|----------------|-------|----------------|
| workarounds | 9 | 13 | 9 | yes |
| hygiene | 14 | 16 | 13 | yes |
| `#[allow]`/`#[expect]` total | 378 | 395 | 375 | yes |
| same, without `reason` | 238 | 251 | 197 (192 counted by the review, see r2 item 4) | yes |
| `eprintln!` (crates/chora/src, plugins, services) | 135 | 147 | 126 | yes |

Commands: the three greps of audit-f row F4, run on the head:
`grep -rE '#\[(allow|expect)\(' crates plugins services --include=*.rs | wc -l` (375), `| grep -vc reason` (197),
`grep -rE 'eprintln!' crates/chora/src plugins services --include=*.rs | wc -l` (126). The other categories
(refactors 6, design 0, records 0) did not change and stay at or below RC.

### 1. Lints

- d3d70fcf: 68 `#[allow]` lines that had a trailing comment now carry it as `reason = "..."` (the comments were
  already the reason); 251 -> 197 without reason (-54, not -68: rustfmt wrapped 14 of the 68 over several lines, and
  the one-line grep of F4 still counts a wrapped `#[allow(` whose `reason` is on the next line; the 68 all have one).
- 509b5b1f: 20 `#[allow]` removed whose lint no longer fires. Found by turning every allow into an expect and
  reading the unfulfilled ones; an allow that one build needs (dead_code for a field only tests read:
  ipc/clients.rs:392; mc config.rs; construct cube.rs) stays. 395 -> 375 total.
- `eprintln!` 147 -> 126 (-21), d3d70fcf:
  - 2 removed: "frames were slow, the core dropped the effects" in tiling main.rs and xp wm/effects.rs repeat what the
    core logs (safety.rs:357, :386). mc keeps its line: it also says the panels stay plain.
  - 19 consolidated: the same `eprintln!("chora-xp: {err}")` was copied 12 times in xp; the same page line 7 times
    in mc; twice in construct. Each plugin now has one `report()` (xp main.rs:130, mc main.rs, construct
    main.rs). Every error is still reported, with the same text. **Deviation:** most of the 147 are real error
    reports, which the brief says to keep, so the count falls by consolidation, not by deletion; only the two watchdog
    lines are pure repeats of the core. A reviewer who wants fewer lines in the log has to decide which refusals
    the core's `refused` line (code only, no message, by design) may stand for.
- Not touched (copies-0 lane): kit ask.rs, desktop_entry.rs, launcher.rs; assistant voice.rs, card.rs; voice
  client.rs, tools.rs, config.rs; xp session.rs; the four state.rs; mc acts.rs.

### 2. Workarounds (section 2): 13 -> 9

| id | state | proof |
|----|-------|-------|
| W7 xp `ESCAPE_WAIT` | closed | red 8a332fc0 (`alt_escape_is_grabbed_when_the_holder_says_it_is_free`, failed: xp slept and grabbed at once); fix 6b918eee. xp grabs Alt+Escape once; the holder says `construct.escape_free` (construct modes.rs:403) after it gave the key up for `xp.switcher`; xp grabs again on it (wm/keys.rs:425, wm.rs:106). `ESCAPE_TRIES`, `ESCAPE_WAIT` gone; docs/configuration.md line of the switcher updated |
| W8 construct sleep before the tail | closed | red 31a03afd (`the_last_line_of_a_manager_that_exited_is_there_without_a_wait`: waited 150.3 ms, limit 140); fix 07eb024c. `Followed::settle` (cube/start.rs:91) waits until the reader of the error file has reached its end once more, and wakes it at once; cube.rs:754 calls it, no sleep at :749. The two sleeps at :1440 and :1447 are gone: `time_of(ms, elapsed)` takes the clock |
| W9 F27 part | closed | tests/gate-tiling-apps-click.sh:48 `until_ok` on a changed capture replaces `sleep 3`; the `sleep 1` before the check that alpha did not start is gone (zebra is up, so the click chose zebra). fuzzel is no node of the scene (`ctl query '*'` shows none, app_id:fuzzel is empty), so the capture is the event. 3 of 3 green on the head. F21 was closed before (gate-construct-pull.sh `until_ok`); 673 other `sleep` lines under tests/*.sh are not item W9 |
| N6 (FRICTION.md:57) | closed | `tests/headless-session.sh:82-86` prints the log and exits 1 on a start that ends before the socket is up, `:102-105` the last 20 lines of a core that ends later; FRICTION.md says so (c79a9d66) |

Left open: B2, B3, B4, B6, N1-N4 (known-red rows), N5. That is 9.

### 3. Hygiene (section 6): 16 -> 13

- H8 closed, 8c28bb04: README.md heading "Use of LLM Coding Tools" and the text of "Rules for agents" of
  git-conventions: "This project was developed with the help of LLM coding assistants. The authors reviewed and take
  responsibility for all content." (no vendor, no model).
- H12 closed, dev-rules.md (this branch): a merge that touches the IPC or security adds a D entry in the same
  branch, or its message has `No D entry: <reason>`.
- H10 closed, 9ab96135: tests/gate-mc-handover.sh `within_ms` 820 ms. Measured: five runs of mc-handover beside
  mc-paths and mc-sofa (the three steps that `merge-gate.sh --jobs 3` runs together), cells give / key /
  drag-dropped in ms:
  291/634/507, 281/590/514, 389/653/581, 507/628/631, 266/560/583; slowest 653 (key, run 3); +25 % = 816, rounded to
  820. The pull step back (gate-construct-pull.sh:220) waits for the stood-back state of the cube (face 0 not
  forward, 300 to 1100 px wide, as gate-construct-cube.sh:167) and asserts it; before the cell read one node
  after `sleep 3`. The pull gate passes (62 s).
- Left open: H2, H3, H4, H5, H6, H7, H9, H11, H13, H14 and the three review minors (13).

### 4. Evidence

- `cargo test --workspace`: 2,490 passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `tests/merge-gate.sh --jobs 3 --base 85764974` on c79a9d66: all steps passed, every step PASS (red-first, pre-commit,
  build 30 s, test 198 s, clippy, ci-check, 2d-guard, ..., pull, construct-hud, mc-handover 11 s, e2e 296 s). In the
  gate mc-handover took give 322, key 274, drag-dropped 513 ms, under the new 820 ms limit. Nothing was pushed by the gate.
- The remote ref of the train (origin/claude/autonomous-dev-session-3w-i33pg7) is at 05878d2a, older than train 65, so
  `--base` names 85764974.

### 5. Open for the PM

- A new client event `construct.escape_free` (plugin to plugin, documented in docs/configuration.md). By the new H12
  rule it wants a D entry; I left it out because D107+ belongs to DOCS-F8c. Number it at the merge.
- Refactor proposals: (1) `report()` x3 is a copy by plugin; one `kit::report(name, err)` would hold it
  (S, but eprintln! in the kit is outside the F4 command, so the count would fall without a change in the log).
  (2) `Followed` in construct cube/start.rs and the reader of a log file in services/voice are the same follow
  loop (M). (3) about 24 `disallowed_methods` allows, most in tests, say "the test connects to its fake core": a
  kit `testing::connect` would remove them (S).

## Review r1

Reviewed claude/debt-down at c79a9d66 against base 85764974 (10 commits, 67 files). Brief: briefs/review-debt-down-r1.md.

**Verdict: FIX** (one open MAJOR, rd-1).

Checks:
1. F4 counts, using the audit-f F4 commands on each tree: `#[allow]`/`#[expect]` total 395 -> 375 (RC 378). Without `reason`: 251 -> **192** (RC 238; the r1 table says 197). `eprintln!`: 147 -> 126 (RC 135). Workarounds by id: 13 -> 9 (B2, B3, B4, B6, N1-N5 open; RC 9). Hygiene by id: 16 -> 13 (H2-H7, H9, H11, H13, H14 and DB7-DB9 open; H12 is in dev-rules.md:83 on the log branch; RC 14). Every count is at or below RC. Wrapped reasons: the base already has 61 `#[allow(` lines whose `reason` is on a later line; the head has 75. So 14 new ones were wrapped, as the report says. The one-line grep counts all 75 as "without reason"; only 117 of the 192 really have no reason.
2. Red first holds. 8a332fc0 fails on its own tree (tests_moved.rs:403) and passes on 6b918eee. 31a03afd fails ("waited 150.1 ms") and passes on 07eb024c. All tests of xp, construct and mc pass on the head (253, 204, 132).
3. W7 design: see rd-1 and rd-2. With no holder, xp's first grab succeeds, so that case is fine.
4. Every hunk in crates/chora/src, chora-scene and chora-draw was read. They only add a `reason`, wrap an attribute or remove an allow. No code changed (removed allows: input.rs:590, capture.rs:391, blur.rs:69, node.rs five field allows, theme.rs:1166).
5. d3d70fcf: `report()` prints the same text as before ("chora-xp: {err}", "chora-mc: {what}: {message}", "chora-construct: {err}"). Every former call site still reports. Only the two watchdog lines were dropped. `cargo clippy --workspace --all-targets -- -D warnings` is clean on c79a9d66 (rc 0).
6. Gates: mc-handover 820 ms still fails on a hang and is tighter than 1000 ms, so a slowdown shows. apps-click `until_ok ... 80` fails after 20 s. The pull step-back now asserts the stood-back state and fails after 10 s. A click before the rows are drawn would start the wrong program, which the gate catches as red, not as a false green.

Findings:
- rd-1, MAJOR, plugins/chora-xp/src/wm.rs:106 (dispatch.rs:71, docs/configuration.md:2186): xp now listens for the construct's own event `construct.escape_free`. That makes one look depend on another look's event name. Any other client that holds Alt+Escape and gives it up on `xp.switcher`, which is what the docs promise, now never gets it grabbed back, so Escape no longer cancels the switch. The old 4 × 15 ms retry worked for any holder. Fix: make the event generic, with no holder name. KISS form: the core emits `input.ungrabbed {binding}` on every ungrab, and xp grabs again on it. The holder then needs no event of its own, and this IPC addition gets its D entry under the H12 rule.
- rd-2, MINOR, plugins/chora-xp/src/wm/keys.rs:419: while the construct holds Alt+Escape (its step-back binding), the first grab is refused on every Alt+Tab. That expected refusal goes to `crate::report`, so each switch now writes a "chora-xp: ..." error line that the retry used to hide. Fix: leave the refusal of this grab out of the report, since the free event retries it.
- rd-3, MINOR, plugins/chora-construct/src/cube/start.rs:91: `settle` waits for `passes + 1`. A reader whose `read` returned 0 before the manager's last write, but which takes the lock only after `settle` set its target, finishes the pass too early, and `last()` returns the old line. Fix: count only passes that start after the ask, e.g. the reader clears `asked` before its read and `settle` waits for `passes + 2`.
- rd-4, MINOR, reports/debt-down.md "## r1" table: the "without reason" count for the head is 192, not 197 (the 197 is from before 509b5b1f). Fix: correct the table, and note that 75 of the 192 have their reason on a wrapped line.
- rd-5, MINOR, services/voice/src/client.rs:548, tools.rs:320, config.rs:903 (d3d70fcf): these copies-0 files were edited (one `reason` line each), but the r1 report lists them as "Not touched". Fix: list them in the report so copies-0 can expect a one-line conflict, or drop the three hunks.
- rd-6, NIT, tests/gate-tiling-apps-click.sh:21: a 24th copy of `until_ok` in tests/*.sh. Fix: move `until_ok` into gate-lib.sh in a later hygiene lane.

## r2

Code: claude/debt-down at c8fe032b, 4 commits over r1 (c79a9d66): `git log c79a9d66..claude/debt-down`.
Brief: briefs/debt-down-r2.md. Fixes the findings rd-1 to rd-4 of "## Review r1".

| item | state | proof |
|------|-------|-------|
| rd-1 MAJOR xp knows the construct's event | closed | red 4f90c968 (`an_ungrab_is_told_to_the_subscribers` in ipc.rs, `alt_escape_is_grabbed_when_the_holder_gives_it_up` in xp tests_moved.rs; both failed), fix c8fe032b |
| rd-2 refused-grab line on each Alt+Tab | closed | c8fe032b; unit test `only_a_refusal_by_a_holder_is_expected` |
| rd-3 `settle` ends early | closed | red 07fe4d03 (`a_pass_that_ended_after_the_ask_but_began_before_it_does_not_settle`, failed: "the pass of before the ask settled it"), fix 9346e83d |
| rd-4 table says 192 without reason | closed | see below |

### rd-1

- The core emits `input.ungrabbed {binding}` on every successful `input.ungrab` to every connection that subscribed to it
  (an empty list hears it as any core event): `chora-scene/src/event.rs` (variant `InputUngrabbed`, `NAMES`),
  `crates/chora/src/ipc/grabs.rs:ipc_ungrab` calls `State::emit`. A refused ungrab and a grab that ends with its
  connection send nothing.
- xp subscribes to it (`EVENTS`) and, in `wm/keys.rs:ungrabbed`, grabs Alt+Escape again when the binding is Alt+Escape
  and the switcher shows without the key. xp knows no holder and no event name of another look.
  The test also pins that another binding given up (`Super+a`) does not make xp grab.
- The construct lost `ESCAPE_FREE` and `say_escape_free`; it ungrabs its step-back binding on `xp.switcher` as before
  and the core's event does the rest. `grep -rn escape_free` is empty (the repo, without `target/`).
- D entry in docs/decisions.md with the number `D-ungrab` as a placeholder (row after D106, "Decisions of the shared
  core"): **the PM numbers it at the merge** (DOCS-F8c holds D107 and on; the three places that name it are
  docs/decisions.md, docs/api.md (event table row and the `input.ungrab` row) and the Alt+Tab row of
  docs/configuration.md, which also says that the first refusal is not logged).
- Honest note on the red: the red test in ipc.rs failed at `events.subscribe` ("there is no event"), since the event did
  not exist; its filter read `e["event"]` where a test client keeps `e["params"]["event"]`. The fix commit corrects the
  filter, so the green test does check the event and its binding, and that a grab and a refused ungrab say nothing.
  The red commit therefore proves "no event" but not the shape; the green run does.

### rd-2

`grab_switch_escape` no longer reports a refusal with code REJECTED: for the grab of `Alt+Escape` that can only be
"grabbed already" (the safety-layer refusal needs Ctrl+Alt, and hello has been sent), which `input.ungrabbed` retries.
Every other error (a socket, a limit) is still reported. The decision is the function `held_by_a_client`, unit tested; the
log line itself is not captured by a test (`eprintln!`).

### rd-3

`Followed` now counts asks, not passes. The reader takes `begin_pass()` (the number of asks so far) before each read; at
the end of the file it records `covered = max(covered, began)`. `settle` makes ask number n and waits until
`covered >= n`, so only a pass that began after the ask counts. This is stricter than the review's `passes + 2`: it does not
depend on how many passes were in flight. The W8 test of r1 (`..._is_there_without_a_wait`, limit 140 ms) still passes.

### rd-4

On the head of r1 and on r2: `grep -rE '#\[(allow|expect)\(' crates plugins services --include=*.rs | wc -l` is 375 and
`... | grep -vc reason` is **192**; the r1 table said 197, which was the count before 509b5b1f. 75 of the 192 have their
`reason` on a wrapped line that the one-line grep does not see (the review's count: 61 in the base, 75 on the head), so 117
really have no reason. The table of "## r1" now says so. r2 changed no `#[allow]` line.
rd-5 and rd-6 (copies-0 files edited by d3d70fcf, the 24th `until_ok`) were not part of this round and stay open: the three
files with a one-line `reason` change are services/voice/src/client.rs, tools.rs and config.rs.

### Evidence

- `cargo test --workspace`: 2,493 passed, 0 failed (r1: 2,490; +3 tests).
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `tests/merge-gate.sh --jobs 3 --base 85764974` on c8fe032b: all steps passed, once, no flake
  (build 13 s, test 119 s, clippy 13 s, pull 59 s, mc-handover 9 s, e2e 283 s; 41 steps PASS). Nothing was pushed by the gate.
- Commit author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>, no attribution lines, no `--no-verify`.
  Two commit attempts failed on the pre-commit hooks (an inline comment in a test, a subject over 68 characters) and were
  redone after the fix.

### Open for the PM

- Number `D-ungrab` at the merge (sed over docs/decisions.md, docs/api.md, docs/configuration.md, the doc comments of
  `ipc/grabs.rs`, `event.rs` and `wm/tests_moved.rs`: `grep -rn D-ungrab`).
- The event goes to every subscriber including a confined client; it carries only the binding string. If that is too wide,
  the core could send it to unconfined connections only.

## Review r2

Reviewed claude/debt-down at c8fe032b against r1 c79a9d66 (4 commits, 12 files). Brief: briefs/review-debt-down-r2.md.

**Verdict: FIX** (one open MAJOR, rd-7).

Checks:
1. rd-1: the core emits `input.ungrabbed {binding}` on a successful `input.ungrab` only (grabs.rs:76). A grab that ends
   when its connection closes (exit or crash, ipc.rs:1165) or when `input.grab-any` is revoked (ipc.rs:402) sends nothing,
   and api.md and the D entry say so. That is narrower than the brief and the rd-1 fix ("on every ungrab"): see rd-7.
   xp grabs Alt+Escape on the event for any holder, and with no holder its first grab succeeds (keys.rs:418-445,
   dispatch.rs:71). `git grep escape_free` is empty. A confined subscriber learns only a binding string and no client
   name; `xp.switcher` already tells the same timing, so nothing new leaks. Race: two waiters that grab again on the same
   event get one success and one REJECTED "grabbed already" (grabs.rs:49). xp drops that refusal silently
   (`held_by_a_client`), and after hello the only other REJECTED of `input.grab` is the safety layer, which needs Ctrl+Alt.
   So rd-2 is closed. xp's own ungrab in `end_switch` comes before `xp.switcher false`, so the construct's grab-back is not
   refused. A stale event from xp's own ungrab only causes one more silent refusal.
2. Red first: 4f90c968 fails (`an_ungrab_is_told_to_the_subscribers`: "there is no event \"input.ungrabbed\"";
   xp `alt_escape_is_grabbed_when_the_holder_gives_it_up` panics at tests_moved.rs:409, the `from_value` of the unknown
   event). c8fe032b passes (chora 614, xp 254). 07fe4d03 fails at start.rs:268 ("the pass of before the ask settled it")
   and 9346e83d passes (construct 205). Both rd-1 reds fail on an event that does not exist, not on its behaviour, and the
   red ipc test had a wrong filter that the fix commit corrected (see rd-8). The red still shows that the behaviour was
   missing; the green test checks its shape.
3. rd-3: `settle` makes ask n and waits for `covered >= n`. The reader reads `begin_pass()` before each `read_line`, so
   only a read that started after the ask counts. A pass that began earlier wakes at once on `asked` and starts a new pass.
   There is no new fixed sleep in the code (the test's 50 ms only waits before a negative check). A hang still ends at
   `SETTLE` and does not settle. Closed.
4. Docs: api.md (event row :728, `input.ungrab` row), configuration.md Alt+Tab row and the D-ungrab entry (D116+ at the
   merge) match the code, including "a grab that ends with its connection sends nothing". For a plugin author that
   sentence is the surprise. They would expect a crashed holder's key to be told free too (rd-7).
5. rd-4 re-run on c79a9d66 and c8fe032b: 375 attributes, 192 without `reason` on the line. Of those, 75 (base 61) have it on
   a wrapped line, so 117 really have none. The report figure is right. Closed.
6. `cargo clippy --workspace --all-targets -- -D warnings` on c8fe032b: clean (rc 0). `cargo check --workspace`: ok.

Findings:
- rd-7, MAJOR, crates/chora/src/ipc.rs:1165 (and :402, docs/api.md:728, decisions.md D-ungrab): a grab that ends with its
  connection (holder exit or crash) or by a revoked `input.grab-any` sends no `input.ungrabbed`. A construct that exits,
  crashes or restarts while xp's switcher shows therefore leaves that switch without Escape. Any other waiter must also watch
  `client.disconnected` and guess, which the generic event was meant to spare, and the D entry fixes that gap as the
  contract. Fix: in `ipc_close` and `drop_free_grabs`, emit `InputUngrabbed` for each dropped non-`move` grab after the
  connection is gone, with a test, and drop the "sends nothing" clause from api.md and the D entry.
- rd-8, NIT, crates/chora/src/ipc.rs:4016 (4f90c968): the red ipc test filtered `e["event"]` and could not have passed, and
  the fix commit changed its body, so the committed red is not the test that went green (the lane noted this itself).
  Fix: none here; in later lanes, the red commit holds the final test body.

Known, not findings: rd-5 and rd-6 (deferred); the copies-0 and moved-tests conflicts in xp wm.rs and tests_moved.rs belong to the train.

## r3

Code: claude/debt-down at 121c30aa, 2 commits over r2 (c8fe032b): `git log c8fe032b..claude/debt-down`.
Brief: briefs/debt-down-r3.md. Fixes rd-7 of "## Review r2"; rd-8 (NIT) needs nothing, history not rewritten.

| item | state | proof |
|------|-------|-------|
| rd-7 MAJOR a grab that ends with its connection or by a revoke is not told | closed | red b41d2aab (two tests, both failed with an empty list), fix 121c30aa |

### rd-7

- Red b41d2aab: `ipc::tests::a_grab_that_ends_with_its_connection_is_told` (a holder with Alt+Escape, Super+a and Super+BTN_LEFT
  closes; the subscriber gets `input.ungrabbed` for each and then grabs Alt+Escape) and
  `input::tests::a_revoke_of_grab_any_tells_the_ungrab` (revoke of `input.grab-any` tells the free `Tab`, not the held-by-modifier
  `Alt+Escape`). Both failed on the behaviour: left `[]`, right the bindings. The red commit holds the final test bodies (rd-8).
- Fix 121c30aa: `IpcState::drop_free_grabs` and the new `drop_grabs` return the bindings of the dropped non-`move` grabs;
  `ipc_close` and `revoke_free_grabs` pass them to the new `State::tell_ungrabbed`, which `ipc_ungrab` uses too, so the
  three ways a grab ends say the same event. In `ipc_close` the event comes after the connection left `conns`, so the closing
  connection hears nothing and a waiter can grab at once.
- Docs: the clause "a grab that ends with its connection sends nothing" is gone from docs/api.md (event row, `input.ungrab`
  row) and from the D-ungrab entry, which now names the three ends and that a refused ungrab and a `move` grab say nothing.
  The doc comments of `event.rs` and `grabs.rs` follow. `D-ungrab` stays unnumbered, "Accepted, debt-down r3".
- xp needed no change: it already grabs on the event for any holder.

### Evidence

- `cargo test --workspace`: 2,495 passed, 0 failed (r2: 2,493; +2 tests).
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `tests/merge-gate.sh --jobs 3 --base 85764974` on 121c30aa: all steps passed, once, no flake (41 steps PASS, e2e 298 s).
  Nothing was pushed by the gate.
- Author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>, no attribution lines, no `--no-verify`.

### Open for the PM

- Number `D-ungrab` at the merge (`grep -rn D-ungrab`), as in r2.
- A `Super+BTN_LEFT` style button grab is told like a key grab; `move` is not. If a subscriber should not hear buttons, say so.

## Review r3

Reviewed claude/debt-down at 121c30aa against r2 c8fe032b (2 commits, 6 files). Brief: briefs/review-debt-down-r3.md.

**Verdict: MERGE-OK** (no open BLOCK or MAJOR; rd-7 closed).

Checks:
1. Red first: on b41d2aab both new tests fail on the behaviour (`a_grab_that_ends_with_its_connection_is_told`: left `[]`,
   right the three bindings; `a_revoke_of_grab_any_tells_the_ungrab`: left `[]`, right `["Tab"]`). On 121c30aa
   `cargo test -p chora` passes 616, xp 254. Both tests read `input.ungrabbed` from a subscriber's stream, not a helper, and
   the ipc test grabs the key afterwards; the red commit holds the final bodies (rd-8 heeded).
2. rd-7: `ipc_close` (ipc.rs:1187) and `revoke_free_grabs` (input.rs:348-364) now tell through `State::tell_ungrabbed`
   (grabs.rs:83), as `ipc_ungrab` does. In `ipc_close` the connection has left `conns` first, so it hears nothing of its own.
   Every end of a grab goes through one `retain` of `ipc.grabs`, so a binding is told once: an `input.ungrab` that ran first
   leaves nothing for the close, and the loop is single-threaded. A revoke followed by a close tells disjoint sets. The
   emergency reset closes through `ipc_close` and tells the clients that are closed next, which is harmless. `emit` only
   queues (`broadcast`), so no re-entry from `ipc_flush`. A `move` grab is per connection (`ipc_grab` counts holders of
   `move` only on the own connection), so nobody is ever refused it or waits for it: leaving it out is right. The close
   and revoke paths leave it out, but an explicit `input.ungrab("move")` still tells it (probe test, not committed:
   `[{"binding":"move"}]`), against the docs (rd-9).
3. Button grabs: told like key grabs, which is what a plugin author expects of "a grab ended". Only xp subscribes
   (keys.rs:437) and it acts on `Alt+Escape` only; tiling and kit do not listen, so no plugin grabs a button binding again.
   A shared button binding (D55) is told when one window manager lets go while another still holds it, so the event means
   "a grab ended", not "the binding is free" (rd-10).
4. Docs: api.md:728 and :896, the D-ungrab entry (decisions.md:156), event.rs and grabs.rs name the three ends; the
   "sends nothing" clause is gone (`grep` empty). D-ungrab stays unnumbered. The `move` sentence does not match the code
   for an explicit ungrab (rd-9).
5. `cargo clippy --workspace --all-targets -- -D warnings` on 121c30aa: clean. `cargo check --workspace`: ok.

Findings:
- rd-9, MINOR, crates/chora/src/ipc/grabs.rs:75 (docs/api.md:728, decisions.md:156): an explicit `input.ungrab("move")`
  emits `input.ungrabbed {binding: "move"}`, while the docs say a `move` grab is not told and the close and revoke paths
  leave it out. Fix: skip `Trigger::Move` in `ipc_ungrab` before `tell_ungrabbed`, with a test that an ungrab of `move` tells nothing.
- rd-10, NIT, docs/api.md:728: with a shared button binding (D55) the event comes while another window manager still holds
  the binding, so a waiter may be refused again. Fix: add to the event row that a shared button binding can still be held.

Known, not findings: rd-5, rd-6 (deferred), rd-8 NIT; conflicts with trains 69, 70 are the train's.
