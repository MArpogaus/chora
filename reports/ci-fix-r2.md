# ci-fix r2 (rv-1, rv-2, rv-4) - branch claude/ci-fix, from cd98d2c8

Commits (author Marcel Arpogaus, no attribution lines):
- 3954ee2 test: expect cleanup to end a client that ignores TERM (red)
- 4178e3b test: expect packaging-files.py to flag the missing tiling HUD (red)
- 6bcc9bc fix: wait for killed clients in gate-0.8-4 and gate-0.7-4 (rv-1, rv-4)
- fbcf605 fix: compare every shipped plugin data dir in packaging check (rv-2)

Red / green
- rv-4: gate-lib-selftest.sh case with a pid that ignores TERM: red on 3954ee2
  ("remove_after_exit did not end a client that ignores TERM"); green after 6bcc9bc,
  where remove_after_exit sends KILL after the 5 s budget.
- rv-1: gate-0.8-4.sh and gate-0.7-4.sh source proc-lib.sh and use remove_after_exit.
  `grep -n "trap.*kill.*rm " tests/*.sh` now shows only the helper call.
- rv-2: new tests/packaging-files-selftest.sh feeds packaging-files.py (new option
  --pkgbuild FILE) the PKGBUILD of d7be70b9; red on 4178e3b (script passed it), green
  after fbcf605 (names plugins/chora-assistant/huds/tiling/hud.yaml). shipped() now
  walks plugins/*/hud*, assets and collectors; HEAD: 165 files, 0 gaps. The self-test
  runs in the ci-check step of the merge gate.

Runs
- tests/gate-0.8-4.sh: 3 runs rc 0, no gate4-rt dir left behind.
- Full tests/merge-gate.sh (no --push) in one run: all steps passed (exit 0).

Open / notes
- gate-0.7-4.sh needs podman and was not run here (the gate-lib self-test covers
  the helper). rv-3, rv-5, rv-6 stay in the backlog as the brief says.
- ci.yaml still runs only packaging-files.py; the self-test runs in the merge gate.
- A Stop hook asks for committer noreply@anthropic.com; I kept the author the brief
  names, so GitHub may show the commits as Unverified.
