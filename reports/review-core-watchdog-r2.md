Verified: HEAD 8313f011. cargo test -p chora safety 22/22 ok; clippy -p chora --all-targets -D warnings clean. Red commit e9db01f1 (Tasks::frame(0,10) must Blame) precedes the fix 14a8372c in git log.

1. MAJOR fixed: safety.rs stop_in takes max of per-thread wait deltas, then min(took). One thread's wait cannot exceed wall time, so a costly scene is blamed at any thread count.
2. Holes: a thread new in the second read gets before=0, so its delta is at most its lifetime, inside the frame (ok). A thread that vanishes drops out (stricter). A counter that goes backwards gives saturating_sub 0 (stricter). u64 ns cannot wrap. An empty or failed read gives Some(empty) or None, so waited is 0 (strict). An unreadable "before" gives waited 0 (strict).
MINOR crates/chora/src/safety.rs:~137 (unwrap_or_default): a thread that exists but was unreadable in the start read (or a reused tid) is credited its whole lifetime wait; capped by took, that could excuse a whole frame. Needs a transient read failure of a live thread, so practically near-impossible. Fix: skip threads missing from `before` (filter_map on before.get). Earliest gate: unit test with a thread absent from the first read.
3. Tests and clippy pass (above).
4. tests/agent.sh: "busy" is in the grep; a failed grim runs fail with the grim error; no wait or tolerance added (the 4 pre-capture grims are from r1, 875d0812). tests/gate-tiling-audit.sh still exports CHORA_FRAME_BUDGET_US=500000 and the header says why (a watchdog that drops effects at 2560x1600 takes the borders, glows and shadows with them). The r2 report gives the numbers: 1 of 3 loaded runs failed at the default budget.
Note: 3b883e0b subject says audit but carries the agent.sh edits (the report admits it); history is not rewritten, so cosmetic only.

Verdict: MERGE-OK
