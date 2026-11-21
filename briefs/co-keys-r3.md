# co-keys r3 (rc 2, F7): train 99 red on three construct acts after the merge with guide-checks

Model: sonnet, effort medium (construct hints and act rows; the failing cells are named)
Lane: cloud, claude/co-keys at f70d564f. Cost at most 0.6 %; stop at 250 k context. One area: construct hints.
Why: train 99 (1c599c74 + guide-checks b1b8b73a + co-keys f70d564f + ti-29 + mc-minor) failed the acts cell, three blocking rows, all construct:
- I5 in act "the hint Keys of the stood back cube names a key that opens the key list": 'Turn left' (construct-mode-turn-left-text) overlaps 'Only the keys that work now are listed.' (construct-help-body-note) at [413, 747, 55, 11].
- I6 same act: 'Turn right' (construct-mode-turn-right-text) contrast 1.00:1 ((246,247,250) on (246,247,249)), guide wants 7:1 (the key sheet covers the chip).
- act "the hint Keys of a face names a key that opens the key list": "no the hint Keys on the screen within 10 s"; the still shows the xp face on desktop 2 with no hint.
Train 99b without co-keys runs now; this branch joins the next train.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "co-keys BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Merge origin/claude/guide-checks b1b8b73a (or the session branch once 99b is green, whichever is newer). Known conflicts: plugins/chora-construct/src/camera.rs (keep the co-keys doc comment and `pub(crate) fn grab_keys`, input.rs:126 uses it; filter `key.contains('+') || key.starts_with("XF86") || *key == "F1"`), tests/acts/known-red.toml (take guide-checks' file; add only rows that a breach meets, a stale row blocks under guide-checks).
2) Reproduce: `tests/merge-gate.sh --jobs 1 --only build,acts` red on the merge with the three rows above; name the cause of each with file:line (the face act: is the hint not drawn, or the act looking on the wrong face or output?).
3) Fix KISS: the key sheet of the stood-back cube must not cover the Turn chips (co-39, the sheet and the chip row share the foot): move the sheet or the chips so I5 and I6 hold, no known-red row for it; the face act passes 5 of 5. A red test commit first where the fix is product code. No new lint allow.
4) Gate: `tests/merge-gate.sh --jobs 1 --only pre-commit,build,test,clippy,construct-hud,construct-extra,doors,acts` green. A red cell you did not cause: name it with its log line; do not rerun to green. Poll in the same turn; never end a turn to wait.
Done when: the three rows green 5 of 5 (acts --only construct loop), co-39 closed or its row exact, gate green.
Report: reports/co-keys.md "## r3" on claude/session-log (shas, cause per row with file:line, cells, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/co-keys and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
