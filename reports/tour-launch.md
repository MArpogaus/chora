# tour-launch

## r1

Brief: briefs/tour-launch-r1.md (rml1-1). Branch claude/tour-launch-r1, base df3d1ccf, head 07eb539f.

- Red commit dfab9dcd (tests/test_tour.py `LauncherSupported`, both tests failed on df3d1ccf: fuzzel 1.9.2 from the /usr/bin link farm; no chora-notify started).
- Fix commit 07eb539f (demo/tour/launch.py): `supported_fuzzel()` takes the first fuzzel >= 1.11 of the shell PATH or LAUNCHER_PATH (same version parse as the gate's preflight) and links it as `<session>/fuzzel/fuzzel`, ahead of the /usr/bin farm; none found: RuntimeError, one line. `Launcher.start` runs `ensure_notify`: chora-notify is started in a look whose companions lack it and where `choractl call notify state.get` does not answer.
- Diffstat vs base: demo/tour/launch.py +37 -1, tests/test_tour.py +49 -1 (2 files, 86 insertions, 2 deletions).
- fuzzel: /root/.local/bin/fuzzel, 1.11.1 (the /usr/bin one is 1.9.2).
- `demo/tour/run.py mc`: 22 acts, 22 green, 0 red. Act 4 "Programs" (03-12) shows the launcher; the Power menu (ad hoc click on Power in the same mc session, no act exists) shows Power, Lock, Log out, Restart, Shut down, Screenshot. Media: media/2026-11-26/tour-launch-r1/mc-launcher-act4.png (117 KB), mc-power-menu.png (93 KB).
- Not changed (out of area): demo/tour/chapters/checks.py `fuzzel_log` still wraps `shutil.which("fuzzel")` of the shell PATH; fine where the shell finds 1.11.1 first.
- Gate: `CARGO_BUILD_JOBS=2 tests/merge-gate.sh --jobs 2`, EXIT 0: "all steps passed; no --push, nothing is pushed". No `chora --headless` left.

## Review r1 (PM, 01:49 UTC)

Base df3d1ccf, head 07eb539f, +86 -2 (demo/tour/launch.py +38, tests/test_tour.py +50). Red dfab9dcd before fix 07eb539f: yes. Done-when: mc take 22/22 with launcher and Power menu (media/2026-11-26/tour-launch-r1/), fuzzel /root/.local/bin/fuzzel 1.11.1, gate EXIT 0: yes.
- rtl1-1 MINOR launch.py ensure_notify: starts chora-notify but does not wait until `notify state.get` answers; the first act can run before it serves (a take-timing source like rtr2-1).
- rtl1-2 NIT fuzzel_version re-parses in python instead of reusing the gate's parse (the brief asked to reuse); the test copies the parse too.
Verdict: MERGE-OK (0 MAJOR). Harness branch: train148 with HARNESS-MINOR r1.
