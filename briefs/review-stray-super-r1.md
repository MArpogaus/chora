# review stray-super r1 (rc 1: xp-46, mc-60 MAJOR): seeded review of claude/stray-super-r1 before its merge

Model: opus, effort high, because the diff changes the key path in two looks (xp, mc) where a wrong grab
sends stray keys or steals a joining look's chords (grab order, D118); the builder never reviews itself (rules.md 7)
Lane: cloud, read-only on code: claude/stray-super-r1 head e1183766, base d40142e4 (12 files, +229 -40). One round.
Step 0: `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-stray-super-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore.
Stop at 120 k context; cost at most 0.5 %; at most 8 findings. No merge gate run (the lane ran it green, 65 cells).
Read briefs/stray-super-r1.md (done criteria), reports/stray-super.md "## r1", findings/xp.md xp-46, findings/mc.md mc-60
on claude/session-log; `git diff d40142e4..e1183766`.
Checks, each with its command and files:
1) Red first: 664d2893 (unit tests, gate steps in tests/gate-xp-notepad.sh, mc-paths cell stray-super) before the
   fixes 9d82b085, e1183766 (`git log --oneline d40142e4..e1183766`). Does the e1183766 rule (swallow only from xp's
   first window) have a test that was red before it (matrix xp-first in tests/gate-join-keys.sh)? A fix without one is MAJOR.
2) xp (plugins/chora-xp config.rs, wm.rs, wm/startup.rs, wm/windows.rs): Super alone opens Start; an unbound
   Super chord reaches no client and does not open Start on release; swallowing starts at the first window
   (`Chords::Swallowed` in adopt). Gap: keys typed into an xp client with Super before the first window? Does the
   swallow end when the last window closes or xp hands over? `cargo test -p chora-xp`.
3) mc (plugins/chora-mc config.rs, keys.rs, lifecycle.rs): Super+Space, Super+F1, Super+a, Super+n and the remote
   keys (Right, XF86Back, XF86HomePage) still act; a claimed binding keeps its key; `Look::Swallow` serde skip
   does not change a saved config. `cargo test -p chora-mc`.
4) Joining looks: tiling joining while xp holds a window loses swallowed chords (the lane's stated cost). Judge
   it against POLS and D118 in docs/decisions.md; severity with reason.
5) No core change; the cube face (construct) unchanged; the gate steps poll (until_ok), no new sleep.
Known, not findings: rpx1-* other than xp-46; mc-61 card colours; rv9j-1..5.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rss-N, severity,
file:line, text, fix in one sentence). Append to reports/stray-super.md on claude/session-log under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines): verdict,
counts, BLOCK and MAJOR list. Never end a turn to wait; poll every command in the same turn.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
