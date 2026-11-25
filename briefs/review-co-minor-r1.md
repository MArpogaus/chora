# review co-minor r1 (rc 2: rcr1-1, co-78, co-77): seeded review of claude/co-minor-r1 before its merge

Model: opus, effort high, because the diff changes who gets a new window and the keyboard in construct (adopt.rs, focus with the key sheet open), a core-near routing path; the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-minor-r1 head dd1ac7e4, base 6f37120b (8 files, +227 -10). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, EXIT 0).
Read briefs/co-minor-r1.md (done criteria), reports/co-minor.md "## r1", findings/construct.md co-77, co-78 and reports/ (rcr1-1) on claude/session-log, docs/decisions.md D106; `git diff 6f37120b..dd1ac7e4`; captures media/2026-11-25/co-minor-r1/.
Checks, each with its command and files:
1) Red first: 6c040c09 (tests) before 7becf38f (fix) (`git log --oneline --stat 6f37120b..dd1ac7e4`); are the new cube-cell asserts and face_start_tests red at 6c040c09 by their own code? A behaviour fix with no red test before it is MAJOR.
2) rcr1-1 (plugins/chora-construct/src/{adopt.rs,plugin.rs,main.rs}): a terminal started from a face goes to the face manager, no pane; face_tokens bounded (FACE_TOKENS 8), a token that never gets a window leaks nothing and steals no later window; the `free == false` branch drops the token correctly; a normal Super+t outside a face still makes a pane.
3) co-78 (overlay.rs, camera.rs): with the sheet open and a face window taking the keyboard, the sheet takes it back; Escape closes the sheet and the window gets the keyboard; no key goes to the client while the sheet is open (shared RC bar: no stray keys); no focus loop or flicker between sheet and window.
4) Gate: tests/gate-construct-cube.sh new section and dd1ac7e4: no weakened assert, no new sleep (`tools/sleep-ratchet`), the subshell launcher cannot leave a stray process (`pgrep` at cell end), bounded waits.
5) co-77: the lane found no fault (Foot from Programs in a face is held by xp per D106, C3 passes 5/5). Does the pin cell really check C3, and is D106 the contract? Report only a wrong test, not the policy (PM decided: D106 holds).
6) Beyond the brief: removed `Mode` import leaves no dead path; docs naming the old behaviour.
Known, not findings: co-44, co-45, co-75/76/79 (recheck in P2-R10).
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rcm1-N, severity,
file:line, text, fix in one sentence). Append to reports/co-minor.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
