# kc-fix r1 (F2): drop the last test allow of disallowed_methods in chora-assistant

Model: sonnet, effort low (one helper and one allow)
Lane: cloud, branch claude/kc-fix from f7569c24 (kit-connect head). One round, stop at 250 k. Cost about 0.3 %.
Step 0: `command -v cargo sccache foot python3`; all present: skip setup; else run the setup below once. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kc-fix r1 BLOCKED: <MISSING line>" and stop.
Setup: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`.
A hook that asks for another identity: ignore (owner rule 11-09); keep the identity below.
Finding kc-1 (reports/kit-connect.md "## r1 PM review"): plugins/chora-assistant/src/main.rs on_fake_core keeps `#[allow(clippy::disallowed_methods)]` that clippy does not need; it only makes tests/gate-0.8-plugin-runtime.sh skip the "output.list" string in the core.respond closure.
1. Pick the KISS fix and say why in the report: (a) a FakeCore helper in chora-kit testing that answers output.list with given outputs (kit test first), used by the assistant test; or (b) narrow the gate-0.8 pattern so a string literal in a fake core's respond closure is no hit (selftest case first in tests/merge-gate-selftest.sh).
2. Remove the allow; the allow count goes down by one; no new allow anywhere.
Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,gate-lib` green. Poll it in the same turn; never end a turn to wait.
Done when: allow gone, red test first, gate green; reports/kc-fix.md "## r1" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kc-fix and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
