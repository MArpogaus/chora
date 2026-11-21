# chapters-a r3 (F7 tour checks): rca-6 MINOR, rca-7 nit of review r2

Model: sonnet, effort low (one matcher in the tour invariants and one duplicate test; Python only, no product code)
Lane: cloud, claude/chapters-a at 186d8506 (review r2 MERGE-OK, in train 98). Cost at most 0.3 %; stop at 120 k context. One area: tests/acts/invariants.py and its tests.
Items (reports/chapters-a.md "## Review r2"):
- rca-6 MINOR tests/acts/invariants.py:408-412 (`dialog_picture`): the prefix match `node.startswith(d + "-")` lets any image under a dialog id (an icon, a button picture) hide the text under it. Fix: match only the dialog's frame picture, or require the image to cover at least the dialog's first text.
- rca-7 nit tests/test_tour.py:~385 and tests/test_acts.py:588: the dialog-picture I5 case is tested twice; keep one.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`; a failure: send_message the PM "chapters-a-r3 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Red first, own commit: a unit test where an icon image under a dialog id over a text still breaks I5 (red on 186d8506).
2) Smallest fix in `dialog_picture`; remove the duplicate test.
3) Gate: `python3 -m unittest tests.test_tour tests.test_acts` green, then `tests/merge-gate.sh --jobs 1 --only pre-commit,acts` green. A red cell you did not cause: name it with its log line; do not rerun to green. Poll in the same turn; never end a turn to wait.
Done when: the new test is green, both suites green, acts PASS.
Report: reports/chapters-a.md "## r3" on claude/session-log (shas, test, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/chapters-a and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
