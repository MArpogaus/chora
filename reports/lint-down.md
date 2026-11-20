
## r1

Code: claude/lint-down at 0d1394f1, from 79c8d2b5 (train 81). Gate (`--jobs 3 --only pre-commit,ci-check,build,test,clippy,doors,matrix`): all seven PASS (pre-commit, build, test, clippy, ci-check, matrix, doors); the merge of claude/autonomous-dev-session-3w-i33pg7 was already up to date.

The brief's counts (395, 251, 147) are from debt-r at 85764974 (train 65). At 79c8d2b5 the same commands give 360, 150 and 125, so the three targets (378, 238, 135) held before this round.

| count (section 8 command) | before (79c8d2b5) | after (0d1394f1) | target |
|---------------------------|-------------------|------------------|--------|
| `#[allow(` or `#[expect(` | 360 | 359 | 378 |
| same, without `reason` | 150 | 149 | 238 |
| `eprintln!` | 125 | 125 | 135 |

### 1. eprintln!: no change

The 8 in services/voice are 7 in services/voice/chat (chora-chat) and 1 in services/voice/src/config.rs (a test skip, allowed with a reason). The 6 in plugins/chora-notify (the report says 5) sit in main.rs, bus.rs and card.rs. chora-chat and chora-notify depend on neither `log` nor `tracing` (chora-voice has `tracing`, and it has no `eprintln!` outside that test). The brief says no new dependency, so they stay. Both mains carry the crate-level allow "a plugin logs to stderr, which the session keeps", and a `tracing` event with no subscriber in those binaries would vanish, so the move would also change what the user sees. Decision for the PM: add `tracing` plus a subscriber to those two crates (13 lines), or keep them.

### 2. allow/expect: 3a302a7a

Method: every `#[allow(` of the five crates (46 of them, 29 files) became `#[expect(` for one `cargo clippy --workspace --all-targets --all-features`. An unfulfilled expectation is a stale allow. One was: chora-kit/src/testing.rs `connect` (`clippy::disallowed_methods`; the clippy.toml bans only `Runtime::ungrab_all`). Removed; clippy stays clean with and without `--all-targets --all-features`.

The no-`reason` count cannot fall by adding reasons: all 44 lines of the five crates that the command counts (chora-scene 8, chora-draw 13, chora-kit 6, services/voice 13, chora-notify 4) are the first line of an attribute that rustfmt wraps, and every one has its `reason = "..."` on a later line (checked by reading all 44). The metric counts the wrapped ones; it does not find a missing reason. No attribute was moved or widened, no crate-level allow added.

| crate | allows removed | fixed | reasoned (already had one) |
|-------|----------------|-------|----------------------------|
| chora-scene | 0 | 0 | 0 new, 18 had |
| chora-draw | 0 | 0 | 0 new, 14 had |
| chora-kit | 1 | 0 | 0 new, 14 had |
| services/voice | 0 | 0 | 0 new, 29 had |
| plugins/chora-notify | 0 | 0 | 0 new, 5 had |

### 3. Ratchet: 74525d85 (red), 0d1394f1 (green)

`tools/lint-ratchet` (sh) prints the three counts, fails when one is above `tools/lint-ratchet.list`, prints "lower the list" when one is below, and has `--update` and `--root`. 74525d85 holds the list at 358, 148, 124 (the counts minus one): the script prints three `FAIL:` lines and exits 1. 0d1394f1 holds the list at 359, 149, 125: `ok: lint-ratchet (allow 359, no-reason 149, eprintln 125)`. Wired into the `ci-check` step of tests/merge-gate.sh and into .github/workflows/ci.yaml. The gate cell ci-check PASS (90 s) has the line `ok: lint-ratchet`; pre-commit PASS (2 s) on the range. No self-test file was written (the brief did not ask for one).

## PM review r1 (2026-11-20 03:42 UTC)
Base 79c8d2b5..0d1394f1, 5 files +76 -6. Red first 74525d85. PM recount on 79c8d2b5 by the section 8 commands: allow 360, no reason 150, eprintln 125: the base already met RC (378, 238, 135); reports/debt-2026-11-20.md section 8 (395, 251, 147) was stale. PM fault: the brief took counts from the report without a recount (friction).
- rv-1 MAJOR (train order): the list holds the base counts, but MERGE-OK branches in the queue add eprintln (black-pane +3, mc-p2-fix +2); merged before them, the ratchet turns trains 83-85 red at ci-check. The list must hold the counts of the session branch with the queued merges.
- rv-2 MINOR: no-reason counts a rustfmt-wrapped attribute that carries its reason on a later line (44 in scope); the number does not say what its name says.
- rv-3 MINOR: a reasoned new allow can only pass by raising the list; the help does not say so.
- eprintln in chat and notify: kept (PM decision: their stderr is the plugin log; no new dependency, KISS).
Verdict (computed): open MAJOR -> FIX; r2 brief briefs/lint-down-r2.md.

## r2 (2026-11-20)
Branch claude/lint-down, 0d1394f1..2254d32a. Gate (tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy): all five cells PASS (pre-commit 2 s, build 136 s, test 175 s, clippy 16 s, ci-check 73 s). The session tip 79c8d2b5 was already in the branch, so there was no merge before the gate.
- rv-2 fixed: 41c18c02 (red, tests/lint-ratchet-selftest.sh over a fixture tree: a wrapped attribute with its reason on a later line, one without reason, one eprintln; the line count gave no-reason 2, want 1) then aaecfccd (green). no-reason now reads each #[allow( or #[expect( up to its closing )] over any number of lines (POSIX sh and awk). The selftest is on the ci-check line of tests/merge-gate.sh. Counts at 79c8d2b5: allow 360 (359 on this branch), no-reason 0 (was 149 by lines; a Python cross-check agrees with 0), eprintln 125.
- rv-3 fixed in aaecfccd: the help says no-reason and eprintln only fall, and a new allow with a reason raises `allow` in the list in the same commit, the commit message saying why.
- rv-1 fixed: 2254d32a, the list holds allow 360, no-reason 0, eprintln 130: the scratch merge of the session branch 79c8d2b5 with 741d6bca 5dbd0ccf a767790d 591e98f8 91e6a114 17d9a798 40b61f28, in that order. Base alone: 360, 0, 125; the queued merges add 5 eprintln. This branch holds one allow less (3a302a7a), so after its merge the ratchet prints "lower the list" for allow (359 vs 360) and eprintln (125 vs 130) and passes; lower the list then.
- Conflicts of the scratch merge (the list is the only thing it feeds, nothing of it is pushed): 741d6bca in tests/merge-gate.sh (mc-column and launcher-click, both kept; PM's 88258c87 used as the start), a767790d in tests/persona/README.md (HEAD's text, plus a767790d's sentence on the key list), 91e6a114 in flavors/tiling/wallpaper.sh, flavors/tiling/waybar/waybar.sh and tests/merge-gate.sh (both sides kept, git merge-file --union). No .rs conflict. The scratch merge commits were written with git commit-tree, so no hook ran on them; they are not on any pushed branch.
- A valid commit is not blocked (throwaway worktree of 2254d32a): a wrapped #[allow( with its reason on a later line, allow 359 to 360, passes (exit 0); an #[allow(dead_code)] without reason fails (allow 361 over 360, no-reason 1 over 0, exit 1).

## PM review r2 (2026-11-20, PM is not the builder)
Range: 0d1394f1..2254d32a (41c18c02 red selftest, aaecfccd fix and help, 2254d32a list 360/0/130).
- rv-1 r1 (list vs queued merges): fixed for the seven shas of the brief.
- rv-2 r1 (wrapped attributes): fixed; red selftest first. PM cross-check by a paren-matching scan of origin/claude/lint-down: 402 allow/expect openers, 6 without `reason =` (wobbly.rs:22, gl.rs:8, shm.rs, choractl main.rs, services/voice main.rs:6, tests/probe main.rs:471); each carries a comment reason on the line above or after, which the tool counts as a reason. Consistent with 0.
- rv-3 r1 (help): fixed.
- rv-4 MAJOR (new): mc-remote dac835dd (MERGE-OK 03:50, after the brief) adds one eprintln (plugins/chora-mc/src/main.rs step_back), so after train 85 the tree has 131 > list 130 and the lint-down train goes red. Fix in r3: recount the list on the session head after train 85 (one tool run, no other change).
- rv-5 NIT: the help should say that a comment next to the attribute counts as a reason (it does, and the RC row counted the same way).
Verdict (computed): open MAJOR rv-4: FIX; r3 starts when train 85 is green.
