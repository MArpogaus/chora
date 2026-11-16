# Re-review security-a r5 (148f79da)

1. COSMETIC tests/injected-optout.py:1-14 (docstring): still says `injected: name` is accepted; it is now refused. Update the text. No failure scenario.
2. COSMETIC plugins/chora-assistant/src/main.rs:~250: stale `// injected:` comment on PointerEntered arm still true (hears_pointer filters first), fine.

Verified:
- r4 MAJOR closed. New tests drive Agent::event on a fake core socket (hello/output.list answered). Mutation: chat_pressed ROW branch no-op -> row test fails; toggles_chat forced false -> figure tests fail (new and existing). Restored.
- r4 optout MINOR closed: refuses `ref _x`, `x @ _`, `(_)`, `_ | true`, plain names; accepts `injected`, `injected: true/false`, expressions. python optout over repo rc 0; guards.sh "all guards behave".
- r4 hover MINOR closed: `if false` in place of hears_pointer in assistant and mc event -> both hover tests fail; kit test covers hears_pointer. Restored.
- clippy --workspace --all-targets -D warnings: clean. Tests: only the two known host-bound failures (chora-kit shared, chora-tiling theme).

Verdict: MERGE-OK
