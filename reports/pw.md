# pw: P2 prerequisites (persona checker, stand-ins, round script)

Branch claude/pw, from the train 74 merge a9e7084d, head 3ba07d9b (two commits, tests and docs only,
no product code). Spec: reports/p2-plan.md items 1, 2, 4. Every claim has its command.

## r1

Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib,acts,doors` is
green: pre-commit 1 s, gate-lib 25 s, build 19 s, test 150 s, ci-check 72 s, acts 171 s, doors 320 s.
origin/claude/autonomous-dev-session-3w-i33pg7 was merged first: `git merge` said up to date (a9e7084d
is its head), so nothing came in.

1. Stand-ins (safety). Red first: with the old session.sh stashed,
   `tests/persona-selftest.sh` fails with `systemctl in the PATH of the session is /usr/bin/systemctl`;
   with the new one it passes. The selftest proves in a live mc session that `systemctl`, `wpctl`,
   `grim`, `loginctl`, `notify-send` resolve to `$PERSONA_DIR/fake/`, that this folder is first in the
   PATH of the core (`/proc/$CHORA_CORE_PID/environ`, which starts every program a click runs), that
   `systemctl suspend` is a line of PERSONA_CALLS, that `wpctl get-volume` answers `Volume: 0.50` and
   `set-volume` is logged, that `grim FILE` writes FILE. It also proves `persona-check` holds
   screen.capture and methods.call and not input.inject (`choractl grants`), and that
   `session.sh mc+tiling` starts both looks (client list; PERSONA_FLAVORS=mc+tiling). The gate step
   `acts` runs it (tests/gate-acts.sh). The cleanup of session.sh now stops companion units with the
   real systemctl captured before the PATH is set, else it would have called the stand-in.
2. check.py. `Session.attach(env_file)` and `Session.connect` are split out of `Session.setup`
   (tests/acts/harness.py); an attached session closes its connections only and never removes the
   folder. `test_acts.py` and the acts gate are unchanged and green. `tests/persona/check.py`
   (`start`, `after ACTION`, `end TASK...`): act.py runs it after each action when PERSONA_CHECK=1 (in
   the env or in session.env, which session.sh now writes); output only to
   `$PERSONA_DIR/breaches.jsonl`, a capture in `evidence/`, a crash in `check/errors.log`, never to
   the persona. I4 is skipped; every disconnect is declared, so I14 still reads panics and failed
   effects (the brief said skip I14, the plan keeps it: I followed the plan; say if the brief was
   meant). The contracts T1-T7 (T4:volume|clock|screenshot|lock|power), X1 X2, L1-L3, M1 M2, C1-C3 are
   Expect objects; the history of the run (what was open, the helper word per action, the key list
   words while it was open) is read from `check/history.jsonl`. Command: `python3 -I
   tests/test_acts.py` (125 tests, 18 new in `PersonaContracts`, each contract green on a sound end
   state and red on its breach).
3. round.sh (`tests/persona/round.sh snapshot|start|stop|smoke`): swarm.sh in the repo, no scratchpad
   paths (ROUND_DIR, ROUND_KEEP), binary snapshot with its sha, 2 slots by flock, PERSONA_KEEP per
   name, PERSONA_CHECK=1, the first `check.py start` done. `ROUND_DIR=... tests/persona/round.sh smoke`
   starts each of the 4 looks, runs `act.py move`, `check.py end T7`, stops: `tiling: task T7 pass
   breaches 0`, same for xp, mc, construct, then `smoke: 4 looks ...` (about 35 s). session.sh keeps
   breaches.jsonl, evidence/ and calls.log in PERSONA_KEEP now.
4. Planted breach. `tests/acts/planted.py` `persona_check`: in a persona session of mc a sound action
   leaves no line, a planted black screen leaves an I7 line in breaches.jsonl; in the acts log
   `cell selftest 'persona I7 planted' pass` and `'persona sound' pass`. tests/persona/README.md has
   the new section "Round and check (driver)".

## What the PM should know

- Size: check.py is 459 lines after ruff format (docstrings, the contract table), not about 180; the
  rest is as sized (round.sh 100, session.sh +60).
- A second look beside the first (`tiling+xp`) shows breaches on an idle pair: I3 "tiling and xp each
  say they hold the focus" and I5 overlaps of the xp menu items under the tiling key sheet (seen with
  `act.py key Super+F1` on `round.sh start t1 tiling+xp`). That is two looks drawing at once, as the
  acts harness has them; a T7 driver must hand the window over, and R1 triage should read such lines on
  `a+b` sessions as pair noise until a cell says otherwise.
- The looks poll the stand-ins (`wpctl get-volume`, `systemctl --user is-system-running` appear in
  calls.log without any click). T4 therefore reads only `wpctl set-volume|set-mute`, `grim`,
  `loginctl lock-session`, and for a cancelled power menu only suspend, poweroff, reboot, hibernate,
  halt, kexec; a test covers this (`test_t4_a_poll_is_no_action`).
- Contracts that read state keys I could not exercise end to end here (no persona ran): X2, L1, L2,
  M2, C2, C3 are thin (focus, window set, rect or page change). R1 should tighten each when its first
  persona runs it. T3 reads the answer from `$PERSONA_VOICE/entries.json` (a `role: agent` entry): the
  driver must seed it (session.sh still writes `[]`).
- Not done in r1: the replay of an action log for a breach seen once (plan risk 2); the checker waits
  quiet by `busy` as wait_quiet does, 10 s per action.
