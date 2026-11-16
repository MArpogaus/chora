# harness rc r1: report

Branch claude/harness-rc (74a6b037), on the session branch plus claude/fuzzel-click. Merge gate: all steps passed on the second full run (/tmp/merge-gate.W5VJDW, 28 PASS, e2e 300 s, runs 23 and 24 are the two Apps cells). The first full run stopped at doors (F26 below).

## 1. F25 (agent.sh step 6): changed, not reproduced, not proven
- Premise wrong: tests/agent.sh starts no waybar and no bar (pgrep waybar is 0; the capture shows no bar). A bar wait cannot work; my first commit (44ea4e0b) did that and is replaced by fe72a0b2.
- Red: none. With a waybar stand-in that sleeps 8 s the old script passes 3/3 (no bar involved). ci-f25-r1 did not reproduce it either. Earliest gate: the e2e job of CI (merge-gate e2e reads it).
- Change: grim before and after the capture; the capture counts only when the two host frames are identical, else retry (50 times, 0.2 s); then the compare uses the host frame after. No tolerance added (the old close_to of 8 stays). A host that is stable and differs still fails: that would be the core finding of ci-f25-r1 (glow blended twice on the host).
- Green: agent.sh 3/3 with the shim, and inside both merge gates.

## 2. F16/F26
- The audit already kept waybar's log and failed loud (429d9db3). New: the failure names whether waybar is alive (pid, alive or dead).
- F16 cause, reproduced here (audit first config via a nohup loop failed 3/3, direct runs passed): waybar dies without a word. chora-tiling sends SIGRTMIN+1 (tags changed), SIGUSR1 (fullscreen) and SIGUSR2 to waybar.pid as soon as waybar.sh writes it; waybar has no handlers yet, the default action ends it, the log stays empty, waybar.sh exits silently (wait, exit). The timing is the two foot windows opening right after the start.
- Red: 2330a869 tests/gate-tiling-bar-early-signal.sh (a stand-in waybar that installs its handlers after 1.5 s; the signals arrive at once): FAIL, waybar did not survive.
- Fix c5306456: waybar.sh starts waybar with USR1, USR2 and signal 35 (SIGRTMIN+1) ignored; waybar sets its own handlers later. Test green.
- Audit first config (ink-1280x800), 10 serial runs: 10/10 green (12 ok each). Before the fix: 3/3 red in the same loop. Full-gate tiling-audit: pass twice (332 s, 330 s).
- Doors step, 10 serial runs: 9/10. Run 8 failed: mc header-1024x768 "the page of the agenda" (Super+Home, no "1  AGENDA" in 5 s). That cell alone: 15/15 green. Not explained.
- F26 (mc screenshot-door and volume-door missing) is real: it failed in the first full gate, and 2 of 37 standalone runs of tests/doors/mc.sh (runs 10 of 12 and 12 of 25). In the two logged runs the words "Screenshot" and "Volume 50%" show, their hit nodes sit at y 756..784 with sane rects, nothing overlaps, and the click still does no grim and no wpctl set-mute. First-line doors (Lock, Power, Programs) never fail. Cause not found; the cell now dumps the door nodes and the surfaces on failure (e76c002d, 9f45a538, 74a6b037), so the next hit shows what stood under the pointer. The second full gate passed doors (329 s). Status: reproduced-not-fixed.

## 3. ti-1
- gate-tiling-apps.sh and gate-tiling-apps-click.sh are in the CI e2e job (44ea4e0b), so the merge-gate e2e step runs them (e2e-sequence reads the job): they passed in the gate.
- The rpm already requires fuzzel >= 1.11 (crates/chora/Cargo.toml requires, checked by tests/gate-package-deps.sh, a gate step that passed); nothing to add.

## Skips (did not run, environment)
desktop-file-validate (session-units), xkeyboard fkeys (0.7-6), pipewire (screencast), xwayland-satellite (x11), thunar (xp-token); all listed in tests/e2e-skips.txt.

## Notes
- A stop hook asked to re-author the commits as noreply@anthropic.com; not done, the brief names the owner's identity.
- setup.sh built fuzzel 1.11.1 now (the scdoc fix of fuzzel r2 is in).
