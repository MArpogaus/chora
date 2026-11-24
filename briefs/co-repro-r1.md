# co-repro r1 (rc 2: hand-reproduce co-77 and co-78 from p2-r12-co before routing)

Model: haiku, effort high (scripted hand runs and state dumps; tests)
Lane: cloud, branch claude/co-repro-r1 at 280b9dcf (binaries of p2-r12-co). Run only: no product, test or design changes. Stop at 150 k. Cost at most 0.4 %.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo build --workspace`. A failure: send_message the PM "co-repro-r1 BLOCKED: <line>" and stop.
Read on claude/session-log: reports/p2-r12-co.md (all; "New rows proposed" 1, 2), findings/construct.md co-77, co-78; media/2026-11-24/persona-r12-hand/co70.sh, co71.sh (start from them); media/2026-11-24/persona-r12-ends/max-C3-dump.txt; tests/persona/README.md on the branch.
1) co-77, 5x each from a fresh compositor (construct 1280x800): (a) Max C3 path: step into a face, start a terminal (Super+t), then Foot from the face's Programs menu; (b) the same in an empty face. Per run record: holder of Foot (dump line), core focus, front face, whether typed text `echo r$N` reaches Foot (read it back from the screen capture or the foot output), Foot visible at full size (capture). The question to answer, as a fact table: does the person get a focused, usable Foot (only the contract's expectation differs), or do keys go elsewhere?
2) co-78, 5x: row, Return, Super+F1 (sheet open), Super+t, then Escape. Record: sheet closed or not (`open` in the dump), focus, capture. Then 5x the same without Super+t as control.
3) A rate table per row; each capture under 2 MB, at most 20, under media/2026-11-24/co-repro/ on claude/session-log. Before ending: `pgrep -fa 'chora --headless'` empty. Never end a turn to wait; poll in the same turn.
Done when: both tables with rates and captures; one line each: "co-77: keys reach Foot N/5" and "co-78: Esc closes N/5 (control M/5)".
Report: reports/co-repro.md "## r1" on claude/session-log; send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines.
Rules: push claude/session-log only (fetch and merge on reject, no force; change only your report and media); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash.
