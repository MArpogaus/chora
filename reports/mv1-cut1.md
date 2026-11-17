# mv1-cut1 r1 (F4, F6)

Branch claude/mv1-cut1, commit 6d2ca7c2 (on c78eca98). Pushed; not merged.

- MV1 `tools/size-ratchet` + `tools/size-ratchet.list` (10 files over 1,500 production lines, counted up to the first inline `#[cfg(test)] mod`; test files 0). A listed file may not exceed the list or its count at the base; a new file over the limit fails; `--update` lowers the list. Stricter than the brief: no growth at all.
- CUT1 `tools/cut-check` + `tools/copies.toml`: plugin-to-plugin dependency blocks (cargo metadata); assets, names, packaging, copies report (a copy row at max 0 blocks). On the head: 18 reports, none blocking.
- Selftests `tests/size-ratchet-selftest.sh`, `tests/cut-check-selftest.sh`: planted breaches fail or are reported; both pass. Wired into `ci-check` of tests/merge-gate.sh and into ci.yaml. Both tools and selftests run in 1.4 s together, pass on the head.
- Gate, one full run on the head (after `git fetch --unshallow`: the first run stopped at 2d-guard, R18, shallow clone). Passed: red-first through ci-check, all steps up to mc-handover. FAILED: e2e, script 24 gate-tiling-apps-click ("a click on the Zebra row did not start zebra"; hard-coded y 533 and `sleep 3` at lines 36 and 39, the known F27). It failed again in `--only e2e`. The diff touches no Rust, so it cannot cause this; I did not run it on the base. SKIPs in e2e: gate-0.9-session-units (no desktop-file-validate), gate-0.7-6 (no fkeys option).
- The gate is therefore not green here; the PM decides on the merge.
- Debt-rc holds no line-count command; I defined the rule above (totals match the plan: xp wm.rs 4,272).

## Refactor proposals
- tests/gate-tiling-apps-click.sh:36,39: replace y 533 and `sleep 3` with a lookup of the row and `until_ok` (F27), S.
- tools/size-ratchet and tests/packaging-files.py are Python without `.py`/shared helpers; fold into one tools/ dir convention, S.
