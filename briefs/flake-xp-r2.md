# flake-xp r2 (rc 3, F5: the review r1 findings of the key-grab order fix)

Model: sonnet, effort high (a routine fix of a plugin start order with a known cause and a named fix)
Red on head: rfxr1-1 on 7fc05748: `chora-xp --output HEADLESS-2`, then after 3 s `chora-construct`; the core log says `chora-construct: 5 bindings are grabbed already (Super+BTN_LEFT to Alt+Super+4)` (8b6f35dc: 4, no Super+BTN_LEFT). Step 1 makes it a test.
Lane: cloud, branch claude/flake-xp-r2 at 7fc05748 (flake-xp r1 head). Area: the key-grab order at look start (chora-construct scene.rs, chora-xp startup, chora-kit binding rule). One area. Stop at 250 k context. Cost at most 0.6 %.
Why: review r1 FIX (reports/flake-xp.md "## Review r1"): MAJOR rfxr1-1, MINOR rfxr1-2, -3, NIT rfxr1-4. Read that section first.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "flake-xp-r2 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) rfxr1-1 red first: a fake-core or gate test that starts the construct beside a window manager holding Super+BTN_LEFT and asserts the construct holds its pane drag. Commit it red. Then fix: keys that need no claim before `build`, the drag (and any key that needs a claim) after the claim exists, as xp does since 08b2fb35.
2) rfxr1-2: one `needs_claim` in chora-kit, matching crates/chora/src/ipc/grabs.rs:28-49, used by xp, tiling and the construct; a unit test per rule (media key, shared button, plain key).
3) rfxr1-3: one cube-off predicate for the early and the late xp grab; the late grab lets go early cube keys that no longer apply. Red test first.
4) rfxr1-4: tests/gate-lib.sh doc says "the keys that need no claim", no sh-47.
5) After: gate-handover, xp-run, launcher-click 10 times each beside a `cargo build --workspace` load: 0 misses; the live check of step "Red on head" shows no Super+BTN_LEFT; then the full gate once (`tests/merge-gate.sh --jobs 2`, no --only).
Never end a turn to wait; poll any long command in the same turn. Before ending: `pgrep -fa 'chora --headless'` empty.
Done when: red commits before each fix, rfxr1-1..4 closed with the cell or test that shows it, miss counts, full gate green.
Report: reports/flake-xp.md "## r2" on claude/session-log (logs under media/2026-11-28/flake-xp-r2/, each under 500 KB); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/flake-xp-r2 and claude/session-log only (fetch and merge on reject, no force; on claude/session-log change only your report and media folder); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages; never a bare git stash (a WIP commit instead).
