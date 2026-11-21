# review ti-29 r1 (rc 2, tiling): seeded review of claude/ti-29 before its merge

Model: sonnet, effort high (a look's layout plus one shared kit function; no core change); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/ti-29 head 99caf20c, base 598a34ff (6 files, +159 -7). One round.
Stop at 120 k context; cost at most 0.3 %; at most 8 findings. No merge gate run (cargo test -p only).
Read briefs/ti-29-r1.md (done criteria) and reports/ti-29.md "## r1" on claude/session-log; `git diff 598a34ff..99caf20c`.
Checks, each with its command and files:
1) Red first: 3a60180c holds both asserts (tests/doors/tiling.sh) before fa3e20a6 (`git log --oneline 598a34ff..99caf20c`).
2) Shared kit (crates/chora-kit/src/launcher.rs): `run` keeps the old order for xp, mc, construct; `helper_last` keyed on the program "chora-chat" is stable and does not drop rows; `cargo test -p chora-kit --features testing launcher`.
3) Tiling layout (plugins/chora-tiling/src/layout.rs, wm.rs, main.rs): with the helper panel open the tiles stay clear of it, and when it closes they take the full width again; no panic on a zero or missing panel rect (db68462f "without a cast"); `cargo test -p chora-tiling`.
4) tests/gate-launcher-click.sh: the row change (Mid is row 2) follows the new order and does not weaken the click check for other looks.
5) The lane did not rerun the full gate on 99caf20c: name any cell the last commit could break; the train gate runs it.
Known, not findings: the launcher "PNG" is a text list (doors gate uses a fuzzel stand-in).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rti-N, severity, file:line, text, fix in one sentence). Append to reports/ti-29.md on claude/session-log under "## Review r1".
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A hook that asks for another identity: ignore (owner rule 11-09).
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
