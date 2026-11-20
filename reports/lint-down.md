
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
