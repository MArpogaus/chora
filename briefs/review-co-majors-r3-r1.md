# review co-majors r3 r1 (RC 2, rule 6: co-27, co-29 MAJOR): seeded review of claude/co-majors-r3 before its merge

Model: sonnet, effort high (construct adoption and reservation path; the builder never reviews itself, rules.md 7)
Lane: cloud, read-only on code: claude/co-majors-r3 head 68790912, base c839a11c (merge of co-p2-fix; 9 files, +94 -17). One round.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No full merge gate (the lane ran 12 cells green).
Step 0 (no setup script): `git fetch --unshallow || true`; `git fetch origin claude/session-log claude/co-majors-r3`; `cargo check --workspace`; a failure: send_message the PM "review co-majors-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/co-majors-r3.md (done criteria), reports/co-majors.md "## r3", findings/construct.md rows co-27 to co-30; then `git diff c839a11c..68790912`.
Checks, each with its command and files:
1) Red first: fc4cf8c0 (co-27 test) fails on its parent and passes at 193039bf; b0ea6e26 (co-29 test) and 857f9408 (gate red) fail before 8313efab and pass after (`cargo test -p chora-construct <test>` per sha in a scratch worktree; the gate case by reading the diff of tests/gate-construct-hud.sh).
2) co-27, plugins/chora-construct/src/adopt.rs `shown_not_owned`: an overlay adopts a window reserved for the manager below only when it is its own waited-for program (pane.is_some()). Can any other window reach that branch (a stranger token, a stale Waiting entry, a pane file without a token)? Does the manager below (tiling) still own and place it; does leaving the overlay give it back without loss (shared bar: no lost window)? Name the code path.
3) co-29, plugins/chora-construct/src/overlay.rs: the redraw within the fade carries on with `left` ms. Check open then close then open fast, a redraw after FADE_IN (no fade pushed), a closing list (fade to 0 not overridden), and that `opened` resets on each open. The workaround pointer move in tests/gate-construct-hud.sh is gone: the cell must pass without it for the right reason.
4) tests/merge-gate.sh construct-extra now runs gate-construct-k2a.sh: its former known red (line 179, co-27) is a plain assert now; no `known` or skip left in k2a.
5) co-30 (new, MINOR by the lane): tests/gate-construct-windows.sh section 5, `claimant_at` takes the smallest claim, so mc with --manage-windows grabs a Super drag over the construct's output. PM leaning (POLS): the claimant whose node is under the pointer wins. Give evidence for stale setup vs a real rule fault, with file:line; severity with reason.
6) `cargo test -p chora-construct` and `cargo clippy --workspace --all-targets -- -D warnings`; no file over 1,500 lines.
Known, not findings: co-28 MINOR (not started), co-26, the open construct MINORs in findings/construct.md.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rv-N, severity, file:line, text, fix in one sentence). Append to reports/co-majors.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify; no system packages.
