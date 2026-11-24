# review co-pull r2 (rc 2: co-70, co-71 MAJOR): seeded review of claude/co-pull-r2 before its merge

Model: opus, effort high, because the diff changes input modes in the construct look (accept/pass/block groups, a scrim that eats presses) where a wrong mode loses a click or traps the user under a sheet; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-pull-r2 head 280b9dcf, base be6e62ec (6 files, +164 -19). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-pull-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, EXIT 0).
Read briefs/co-pull-r2.md (done criteria), reports/co-pull.md "## r2", findings/construct.md co-64, co-68, co-70, co-71
on claude/session-log; `git diff be6e62ec..280b9dcf`.
Checks, each with its command and files:
1) Red first: 5316338c (co-70) and f9e6cfbe (co-71) before the fix 280b9dcf (`git log --oneline be6e62ec..280b9dcf`); each red test fails on be6e62ec for the stated reason. A fix without one is MAJOR.
2) co-70 (command.rs pane_hints, input.rs): the pull hint comes from the same predicate as the key (`pull_acts()`, give_key), no second rule; named in every state where Super+Alt+4 acts stood back, silent where it does not (co-68: pane beside, keyboard in the pane). `cargo test -p chora-construct`.
3) co-71 (overlay.rs, command.rs): root and body accept, the scrim accepts and closes the sheet, a block panel keeps a press on the sheet from the scrim. Does a press on a sheet row or on the Close still reach its handler? Can any press reach the space or a pane behind the open sheet? Does the scrim stay after the sheet closes (no dead layer eating clicks)? Escape still closes. Judge the core's mode inheritance (chora-scene descend) against docs; is the fix in the plugin only (no core change)?
4) The gate step in tests/gate-construct-cube.sh polls (until_ok), no new sleep; docs/cube-modes.md matches the behaviour.
5) Other looks: any shared key-sheet or overlay code changed (chora-kit)? If so, the effect on xp, mc, tiling.
Known, not findings: co-72; co-73 (hud-chat-stop over the Close, 1/8); co-74 (pull not named stepped in); co-44, co-45; rcp1-*.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcp2-N, severity,
file:line, text, fix in one sentence). Append to reports/co-pull.md on claude/session-log under "## Review r2".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
