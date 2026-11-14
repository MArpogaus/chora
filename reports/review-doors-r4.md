# Review doors r4 (claude/doors cb203c68..57df3822), opus, read-only, 11-14 22:4x UTC

Verdict: MERGE-OK. dr3-1 (helpers.is_empty(), follows voice.state/voice.remove, no stale cache), dr3-2 (agent_list ask gone), dr3-3 (shared::entries_in) fixed; red tests real.
| id | sev | file:line | text |
|---|---|---|---|
| dr4-1 | MINOR | tests/gate-hud.sh:635,643 | late-host and reconnected-voice cells switched to "working"; asserting "No helper" while listening would cover the new rule on that path. |
| dr4-2 | MINOR | plugins/chora-assistant/src/voice.rs:601 | a host started after a helper says "No helper" until that helper's next event (as xp); known issue. |
