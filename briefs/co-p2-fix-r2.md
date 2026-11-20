# co-p2-fix-r2 (RC shared bar "no stray keys"): swallow without a timer, one list in the kit

Model: sonnet, effort high (input grab ordering in a plugin and a kit move; the PM reviews)
Lane: cloud, branch claude/co-p2-fix at 7b43b063 (continue on it). One round, stop at 250 k. Cost about 0.8 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "co-p2-fix r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/co-p2-fix.md "## r1" and "## PM review r1" (rv-1..rv-3). On the branch: plugins/chora-construct/src/swallow.rs, cube.rs (SWALLOW_AFTER), camera.rs swallow_unbound, the tiling swallow list (grep the tiling plugin and docs/configuration.md), crates/chora-kit/src/runtime.rs (start_grabs, end_grabs, action).
Items:
1. rv-1 MAJOR: remove SWALLOW_AFTER and the swallow state of cube::Host; the construct swallows only in a pane. Red test first: in gate-construct-cube.sh start the face manager 2 s late (or make it regrab after 3 s) and assert its own Super+d still works on a face; it must fail at 7b43b063. The face co-26 case (Super+F1 on a face) is then the face manager's: keep its cube test, mark it `known: face manager swallow (xp lane)` instead of deleting it, and name it in the report.
2. rv-2 MINOR (F2): one list in chora-kit: `chora_kit::swallow` with KEYS, `chords(modifier)` and a `Runtime` helper that grabs the unbound chords after a grab set (the core's first grab wins; it comes last). Construct and tiling use it; delete both local lists; a tools/copies.toml row for the list with max 0. Unit tests in the kit through FakeCore.
3. rv-3 NIT: chords of the configured modifier (not only Super), or one doc line in docs/configuration.md why only Super.
Do not touch plugins/chora-xp or crates/chora (other lanes).
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7 first, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,doors,construct-hud,cube,out,pull,matrix,tiling-look,gate-lib`; poll it in the same turn; never end a turn to wait. A red cell you did not cause: rerun it alone once and name it.
Done when: red before fix per item, gate cells pass, reports/co-p2-fix.md "## r2" on claude/session-log (per item: red sha, fix sha, test, gate cell).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/co-p2-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
