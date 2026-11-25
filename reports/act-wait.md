
## r1

Base 62a04be0, head 861d58d53 on claude/act-wait-r1 (red f2f57fcb, fix 861d58d5).

Cause: not the inject (0.008 s). check.py after waits in wait_quiet until no look is busy; tiling lists "chip" as busy while the 8 s hint chip (ti-56) shows, so after an unbound key the check ends 8 s + about 1.4 s later and act.py waited for it.

Fix: act.py starts the check as a process of its own (one at a time, chain lock) and returns. The next input action and check.py begin/end/start wait (flock on checks.pending) for every earlier check; a look does not wait. Every check still lands in breaches.jsonl, none skipped or weakened.

Diffstat:
 tests/persona-selftest.sh      |  3 ++
 tests/persona/act.py           | 73 +++++++++++++++++++++++++++------
 tests/persona/check.py         |  9 +++++
 tests/persona/test_act_wait.py | 91 ++++++++++++++++++++++++++++++++++++++++++
 4 files changed, 164 insertions(+), 12 deletions(-)

Timings of act.py key (headless tiling, 1280x800, PERSONA_CHECK=1, 5 runs, s):
- before: Super+q 9.44 9.49 9.44 9.44 9.49; Super+Return 2.62 2.72 2.67 1.61 2.76
- after:  Super+q 0.05 x5; Super+Return 0.04-0.05 x5

Capture: a look 0.3 s after Super+q returned shows the chip: media/2026-11-25/act-wait-r1/chip-0.3s-after-super-q.png (134 KB).

Red test: tests/persona/test_act_wait.py (run by tests/persona-selftest.sh in gate-lib), red on 62a04be0 (3 of 4 fail), green after.
Gate: CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2 -> EXIT 0 (all steps PASS). No new sleep lines in shell; pgrep chora --headless empty.
Note: check.py begin right after an unbound key now waits for the check (about 10 s); the persona's goal start is not on the look path.

## PM review r1 (2026-11-25 16:55 UTC; diff 62a04be0..861d58d5, 4 files, +164 -12, under 200 lines)
- Red f2f57fcb (test_act_wait.py, run by persona-selftest.sh) before fix 861d58d5: met.
- act.py starts check.py as its own process holding a shared flock on checks.pending (inherited by pass_fds, so the parent's close does not release it); the next input and check.py begin/end take it exclusive, so no check is lost and none reads a later scene: met. Every check still writes breaches.jsonl; the persona sees nothing: met.
- Timings: Super+q 9.4 s -> 0.05 s; the ti-56 chip shows on a look 0.3 s later (capture media/2026-11-25/act-wait-r1/): met. Gate EXIT 0, no new sleep: met (lane).
- raw1-1 (MINOR, harness): a look does not wait, but it also starts a check; two checks can then be in flight and checks.chain (flock) does not keep their order, so the look's check may run before the check of the action before it. Low effect (a look changes no scene). Backlog.
- Verdict: MERGE-OK 861d58d5 (0 MAJOR); joins the next train.
