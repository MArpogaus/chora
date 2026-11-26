# review co-drop r4 (rc 1, 2): seeded review of claude/co-drop-r4 before its merge

Model: opus, effort high, because it is a look review (construct key sheet: the shared RC bar's key list must tell what every key does; critic 11-24: opus for look reviews); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/co-drop-r4 head 142260b9, base 080b4f32 (merge of a93bc75b into 369cf96f; 1 file, plugins/chora-construct/src/command.rs, +208 -54). One round. Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane's: all steps passed, EXIT 0).
Read briefs/co-drop-r4.md (done criteria) and reports/co-drop.md ("## Review r3", "## r4") on claude/session-log; `git diff 080b4f32..142260b9`; captures media/2026-11-26/co-drop-r4/.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-co-drop-r4 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Checks, each with its command and files:
1) Red first: 64d9a7a4 before 142260b9; at 64d9a7a4 `cargo test -p chora-construct` fails the sheet test ("Walk forward, bac…"), nothing else; at 142260b9 all pass.
2) rcdr3-1: at 1024x768, 1280x720 and 1920x1080 no row text is cut (no "…"), each text ends before its first keycap; keycaps below a row only where no word fits; cut only past 2 lines; body_height and the column split count wrapped lines. Read the layout math and the captures.
3) Open item from the lane (judge it): at 1024x768 the sheet ends at about 754 with real fonts, 18 px past the 32 px bottom margin (7 wrapped lines need 168, room was 21); the unit test asserts the bottom margin only from 1280 wide on, loosening the brief's step 1 assert. Is Close and the foot row still on screen and reachable at 1024x768? If not: MAJOR with the smallest fix (e.g. 4 columns or a smaller gap, not smaller type). If yes but the margin is broken: MINOR to known issues. A test that no longer asserts what the brief asked: say so as a finding.
4) KISS: no new option, no copy of a kit wrap helper; no new lint allow; `tools/size-ratchet --base 080b4f32`, `tools/sleep-ratchet`, `tools/lint-ratchet` ok; command.rs under 1,500 lines.
5) `git merge-tree --write-tree 613844fc 142260b9` clean (session head t151); `cargo check --workspace` on the merged tree.
Known, not findings: FLAKE-CUBE r1, FLAKE-SOFA, rcdr2-n*, rsl1-1, construct give menu list.rs (ti-57).
Verdict computed: an open BLOCK or MAJOR is FIX; a failed Done-when line is INCOMPLETE; a fix with no red test commit before it is MAJOR; else MERGE-OK. Findings one line each (rcdr4-N, severity, file:line, text, fix in one sentence). Append to reports/co-drop.md on claude/session-log under "## Review r4".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict, counts, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines; never --no-verify.
