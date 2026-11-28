# review tiling-minor r1 (rc 1b tiling; rpt2-2, rpt3-1..4): seeded review of claude/tiling-minor-r1 before its merge

Model: sonnet, effort high (routine: five MINOR/COSMETIC word and state fixes on one look, 502 lines)
Red on head: none (a review; each fix of the lane has its red test commit before it)
Lane: cloud, read-only on code: claude/tiling-minor-r1 at bd641acb, review 7822a66c..bd641acb (17 files, +502 -57; chora-tiling, chora-kit, chora-assistant chat and voice, flavors/tiling/waybar). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-tiling-minor-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: briefs/tiling-minor-r1.md (done criteria), reports/tiling-minor.md "## r1 (rc 1b ...)" (this lane's report), reports/rc1-pass.md rows rpt2-2, rpt3-1..4 and their text.
Checks, each with its command and files (no merge gate; `cargo test -p chora-tiling -p chora-kit -p chora-assistant`, `cargo clippy -p chora-tiling -p chora-kit -p chora-assistant`, `tests/gate-tiling-chip.sh`, `tests/gate-tiling-notice.sh`, `tests/gate-tiling-bar.sh` once on head):
1) Red first: each test commit (0b558a6a, 9fa77480, 6f97804d, 5ba7ff37, 01b30501, 170d1d38, ee2a56c9) fails on its parent and passes on head; a fix with no red test before it is MAJOR. 387bac39 is a test move: confirm it removes no assert.
2) rpt3-1: chip and key list spell a chord one way through one function (KISS), not two formatters.
3) rpt3-2: bar helper word and chat header read one state source; no 1-2 s disagreement path left (read the code; the lane says before-frames missed the window, so the red test is the proof: judge that test). bd641acb: state.get keeps the helper word; check no other caller of state.get changes meaning.
4) rpt2-2: the Looks picker offers moves only with a focused window; no tool name ("choractl") in any notice a person sees; words follow the look's other notices.
5) rpt3-3: the volume tooltip shows only with a reason; rpt3-4: the picker box fits its rows (no clipped or empty line) at 1280x720 and 1920x1080.
6) POLS: no other tiling path loses a word, key or notice (chora-assistant chat/voice change: other looks keep their helper word; tiling-look, tiling-empty, chat-keys green on head).
7) The lane's gate ran in three parts (usage limit, restart), all 68 steps PASS on bd641acb; not a finding (the train gate reruns it in one run).
Known, not findings: rpt1-7, rpt1-8, rth-6, rksr1-4, sh-37..sh-47.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rtmr1-N, severity, file:line, text, fix in one sentence). Append to reports/tiling-minor.md under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
