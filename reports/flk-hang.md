
## r1
- Cause: crates/chora/src/hang.rs:337 (9 s stop Timer deadline = creation + 9 s) was created before `start = Instant::now()` at :344, so an on-time timer ended the loop just under 9 s after start (assert at :346).
- Red a2d68432 (test: start after a timer exists misses its 9 s bound, no sleep); green 022fa50c (start taken before the sources). Head 022fa50c.
- `cargo test -p chora --lib hang` x20: 20/20.
- Gate: tests/merge-gate.sh --jobs 2 full, merged with origin/claude/autonomous-dev-session-3w-i33pg7 (already up to date): 57 PASS, EXIT 0, no foreign reds. No chora --headless left.

## Review r1 (PM; 1 file, +3/-2, 6b01c976..022fa50c)
- Fix: `start` taken before the 9 s stop timer is inserted (hang.rs:329); the bound `start.elapsed() >= 9 s` now holds by construction. ok.
- fh-1 NIT: the red commit a2d68432 is a model test of Instant ordering, not the watchdog test itself, and the fix commit removes it; the real test cannot be made red deterministically, so accepted. 20/20 runs after.
- Full gate EXIT 0, 57 PASS (lane report).
Verdict: MERGE-OK (0 MAJOR); joins t112.
