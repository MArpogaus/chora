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
| same, without `reason` | 238 | 251 | 197 | yes |
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
