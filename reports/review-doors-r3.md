# Review doors r3 (claude/doors dd8669a4..cb203c68), opus, read-only, 11-14 21:5x UTC

Verdict: FIX (dr3-1). dr2-2, dr2-5, dr2-4, dr2-10, xr-3, F1, dr2-3, dr2-7, dr2-8 fixed; no merge conflict with 4e3c019e (merge-tree clean); ci r2 drops selection-radius as doors needs.
| id | sev | file:line | text |
|---|---|---|---|
| dr3-1 | MAJOR | plugins/chora-assistant/src/voice.rs:625, :152, :222 | the construct island says "No helper" only when agent_list is empty (no helper and no project); xp, mc and opener.rs say it whenever no helper runs (POLS). no_helper stays Some(false) after a helper existed: after the last helper exits the island says "Listening" until voice reconnects. |
| dr3-2 | MINOR | voice.rs:80-93 | on an Io error ask_helpers asks again on every voice state and blocks up to 500 ms each. |
| dr3-3 | MINOR | crates/chora-kit/src/launcher.rs:169 | entry_dirs repeats the rule of shared::data_dirs_in. |
| dr3-4 | MINOR | 2a858099, 357f9d7d, 7beef6ed | test in the same commit as the change (not red first). |
| dr3-5 | MINOR | report | ti-1 not clicked with a real fuzzel after ci r2. |
