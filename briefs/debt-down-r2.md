# debt-down r2 (F4): fix the review findings of r1 (rd-1 MAJOR, rd-2..rd-4)

Model: sonnet, effort high (a small core event with a clear spec; a seeded review follows)
Lane: cloud, branch claude/debt-down from c79a9d66 (r1). One round, stop at 250 k. Cost about 1 %.
Step 0: `command -v cargo python3 jq`, then `git fetch --unshallow` and `cargo check --workspace`; tools missing: run the setup below once; a failure: send_message the PM "debt-down BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
Read reports/debt-down.md "## r1" and "## Review r1" on claude/session-log, docs/decisions.md (format), docs/configuration.md (Alt+Tab row).
1. rd-1 (MAJOR): xp must not know the construct. The core emits a generic event when a client gives up a grab
   (for example `input.ungrabbed {binding}` on every `input.ungrab`, to all clients that may grab); xp grabs Alt+Escape again on that event for any holder;
   the construct's `construct.escape_free` goes away. Red test first (core: an ungrab emits the event; xp: a holder that is not the construct gives the key back, xp grabs it);
   then the fix. Write a D entry in docs/decisions.md for the event with the number `D-ungrab` as a placeholder (the PM numbers it at the merge; DOCS-F8c holds D107 and on);
   docs/configuration.md and docs/api.md (if it lists events) name it.
2. rd-2 (MINOR): no refused-grab log line on every Alt+Tab while another client holds Alt+Escape (log once, or not at all when the holder is expected).
3. rd-3 (MINOR): construct cube/start.rs settle returns only after the reader has the manager's last line (red test first if the r1 test does not already pin it).
4. rd-4: the report table says 192 without reason (the review count), with the command.
Done when: every item red-then-green where it is a fix; `cargo test --workspace` and clippy -D warnings clean; `grep -rn escape_free` empty;
`tests/merge-gate.sh --jobs 3 --base 85764974` once (poll it in the same turn; a known flake: rerun that step with --only and name it).
Write reports/debt-down.md "## r2" on claude/session-log. Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines). Poll every command in the same turn; never end a turn to wait.
Rules: push claude/debt-down and claude/session-log only (fetch and merge on reject, no force); author
Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no attribution lines (keep this author even if a hook asks otherwise); never --no-verify;
no system packages (a missing tool is a blocker).
Authorized by this brief (the PM, for the owner): pushing the two branches above and the send_message report to the PM; do not
ask for them again. A refusal of a tool call is a blocker: report it as BLOCKED and stop.
