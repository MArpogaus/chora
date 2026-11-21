# gate-path r1 (F5 harness: merge-gate-selftest breaks under a PATH wrapper)

Model: sonnet, effort low (two-line test-harness fix with its red check; no product code)
Lane: cloud, claude/gate-path at 598a34ff (session head). Cost at most 0.3 %; stop at 150 k context. One area: tests/merge-gate-selftest.sh.
Finding (PM 11-21 06:40, train 98 red on gate-lib, reproduced on 598a34ff): lines 428 and 439 link tools into a stripped PATH with `ln -s "$(command -v "$tool")"`. In hosted sessions `command -v rm` is a guard script first on PATH that runs "the next rm on PATH"; in the stripped PATH there is none, so ci-local.sh cannot remove its log dir and 3 checks fail ("an unknown job leaves no log directory: got '4'", "--help leaves no log directory", "a run without a failure removes its log directory"). Same cause as "gate-lib fails in lane containers".
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`; a failure: send_message the PM "gate-path BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
1) Red first, own commit: a selftest check that a wrapper first on PATH (a temp dir with a `rm` script that execs the next rm on PATH, put first on PATH for the selftest) does not change the nobin/nopc results; red on 598a34ff.
2) Smallest fix: one helper in the selftest that resolves each tool to a real binary (for example the first of `type -aP "$tool"` whose target is not a script, or /usr/bin then /bin), used at both lines. No change to ci-local.sh or merge-gate.sh behavior.
3) Gate: `bash tests/merge-gate-selftest.sh` green in your container, then `tests/merge-gate.sh --jobs 1 --only pre-commit,gate-lib` green. A red cell you did not cause: name it with its log line; do not rerun to green. Poll in the same turn; never end a turn to wait.
Done when: the new check is green, the selftest has 0 FAIL, gate-lib PASS.
Report: reports/gate-path.md "## r1" on claude/session-log (shas, the check, gate summary); send_message the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN at most 5 lines with the head sha.
Rules: push claude/gate-path and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
