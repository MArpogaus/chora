# kit5 r3 (F2): fix review r1 of claude/kit5b (3 MAJOR, 5 MINOR)

Model: opus, effort high (because rw-3 is a new step-aside behaviour across chora-notify and the tiling chat panel)
Lane: cloud, branch claude/kit5b (head 4e003748). One round, stop at 200 k. Cost about 3 %.
Run `git fetch --unshallow` first. Read reports/kit5.md "## Review r1" on claude/session-log; every finding id is a task.
0. Merge the current origin/claude/autonomous-dev-session-3w-i33pg7 (no rebase); resolve to both sides, name each conflict.
1. rw-1 MAJOR: on bus name `Taken` chora-notify keeps serving notify.show/notify.close with `bus: None`; never exit 0.
2. rw-2 MAJOR: construct target starts chora-notify (chora-construct-notify.service), mc slot gets a unit or a documented start; packaged in RPM and PKGBUILD.
3. rw-3 MAJOR: chora-notify steps aside of the tiling chat panel on `assistant.chat` (no notice covers the chat input); a pixel or state cell proves it.
4. rw-4..rw-8 MINOR as the report says (act expectations on `s.state("notify")`, mc clear ids, api.md notify.show/close, screenshot fallback to choractl, RPM asset row).
Each fix: its red test commit first (rules.md 8), then the fix.
Done when: `cargo test -p chora-kit --features testing -p chora-notify -p chora-mc -p chora-tiling -p chora-construct`;
`python3 tools/cut-check` ok; `tests/merge-gate.sh --only build,ci-check,acts,xp,hud,construct-hud,mc-handover,keys,chat-keys` green on the head (poll in the same turn).
Push claude/kit5b. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines:
head sha, finding -> commit, gate) and append "## r3" to reports/kit5.md on claude/session-log.
Rules as briefs/flakes-r2.md, push only your branch and that report; never end a turn to wait
for a long background job. Commits by the owner identity, no attribution lines, never --no-verify.
