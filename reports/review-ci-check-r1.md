# Review ci check r1 (claude/ci-check d039a306, 4b697489), opus, read-only, 11-15 01:3x UTC

Verdict: MERGE-OK. Reconnects unchanged (no subscription state to replay; watch_slot subscribes again), no duplicate subscription, cube and chat host unchanged, gate step lists the check-job scripts.
| id | sev | file:line | text |
|---|---|---|---|
| F1 | MINOR | crates/chora-kit/src/runtime.rs:276 | all_outputs() equals outputs() for construct (no --output filter); could be dropped. |
| F2 | MINOR | tests/merge-gate.sh:144 | the gate lacks cargo test --doc and cargo doc of the CI check job. |
