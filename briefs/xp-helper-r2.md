# xp-helper r2 (rc 1 shared bar "the helper (mouse and keys)", POLS; review r1 FIX: rx-1 MAJOR, rx-2..8)

Model: sonnet, effort medium (word and placement fixes in two plugins, each red first; no core change)
Lane: cloud, claude/xp-helper at eda60a6b (r1 head, review r1 FIX). Cost at most 0.8 %; stop at 250 k context. One area: plugins/chora-xp, plugins/chora-assistant, crates/chora-kit/src/words.rs, tests/gate-xp-helper.sh.
Why: reports/xp-helper.md "## Review r1" on claude/session-log (read it first; each finding has file:line and a fix).
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace`; a failure: send_message the PM "xp-helper BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) rx-1 MAJOR first: red step (the cell reads the greeting text and fails while it names an X the balloon lacks), then the fix: the greeting says only what is true (KISS: it closes by itself; click me to ask), en and de; no new art, no new X.
2) rx-2: no-helper greeting promises nothing the chat does not give ("Click me to see what to do" or NoHelper names the set-up path, whichever is honest); add the none-click cell (red first if it fails).
3) rx-3: state dir through chora_scene::paths::private_dir.
4) rx-4, rx-5: the icon offset comes from the desk (right edge of the placed icons) and only on the output that shows icons; a unit test each (red first).
5) rx-6, rx-7: xp28-editor polls the frame rect (non-zero, equal on two reads) instead of a bare sleep; header names the cell; `need bc` or jq.
6) rx-8: de greeting in Sie forms like every other de word.
7) After: each cell of gate-xp-helper 5 times in a row green (paste counts).
Gate: full `tests/merge-gate.sh --jobs 2` (no --only) on claude/xp-helper merged with fresh origin/claude/autonomous-dev-session-3w-i33pg7, detached: `setsid nohup bash -c '(tests/merge-gate.sh --jobs 2 > /tmp/gate.log 2>&1; echo EXIT $? >> /tmp/gate.log)' >/dev/null 2>&1 </dev/null &`, poll /tmp/gate.log in the same turn; never end a turn to wait. A red cell you did not cause: name it with its log line; do not rerun to green.
Done when: rx-1 red then green with lines; rx-2..8 fixed (or one line why not); 5/5 each cell; the gate green or its foreign reds named.
Report: reports/xp-helper.md "## r2" on claude/session-log (shas, red and green lines, counts, gate summary, one capture path for the greeting); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 4 lines with the head sha.
Rules: push claude/xp-helper and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
