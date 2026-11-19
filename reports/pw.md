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

## Review r1

Review of claude/pw head 3ba07d9b against a9e7084d (review-pw-r1, sonnet high, read only on code). Verdict: **FIX**
(0 BLOCK, 2 MAJOR, 3 MINOR, 3 NIT; the two MAJOR are open).

Checks, each with its command:
1. Stand-ins: `bash tests/persona-selftest.sh` passes (5 ok lines). No path reaches the host's systemctl,
   wpctl, loginctl or grim: the fake dir is first in `session.sh` PATH (session.env, core environ, act.py's
   choractl); `choractl run` sees the empty `is-system-running` of the stand-in, so it starts no systemd-run
   unit; check.py runs no program; cleanup uses the captured real systemctl for the companion units only
   (none start now). Exception: rw-1 (the file a screenshot writes).
2. Red first: 2068ed4c holds selftest and session.sh together. With `git show a9e7084d:tests/persona/session.sh`
   in place, the selftest fails: `systemctl in the PATH of the session is /usr/bin/systemctl`. As claimed.
3. harness: `python3 -I tests/test_acts.py` OK (125 tests). `attach` sets `attached`, `__exit__` returns before
   the teardown; `setup` now calls `connect("acts")`, otherwise unchanged. See rw-7.
4. check.py: writes only breaches.jsonl, evidence/, check/ (no print on `after`, act.py sends it to DEVNULL);
   T4 ignores polls (`test_t4_a_poll_is_no_action`). Contract tests are not complete: rw-2.
5. round.sh: no scratchpad or home path (/tmp default, flock slots 1 2). `ROUND_DIR=$(mktemp -d)
   tests/persona/round.sh smoke`: tiling, xp, mc, construct each `task T7 pass breaches 0`, rc 0.
6. planted: persona_check run alone: `persona sound` none, `persona I7 planted` found.

Findings:
- rw-1 MAJOR tests/persona/session.sh:137-143: HOME and XDG_PICTURES_DIR stay the host's, so a Screenshot click
  (power.sh -> screenshot.sh) does `mkdir -p` and the grim stand-in `: >"$last"` writes an empty png into the
  host's real Pictures/Screenshots (shown: screenshot.sh with the stand-in grim and no XDG_PICTURES_DIR saved to
  $HOME/Pictures/Screenshots/...png); fix: export XDG_PICTURES_DIR=$dir/pictures (and an xdg-user-dir stand-in
  printing it) and add a selftest line that runs screenshot.sh and finds the file under $PERSONA_DIR.
- rw-2 MAJOR tests/test_acts.py (PersonaContracts): X2, L1, L2, L3, M2, C2, C3 have no test at all, so
  reports/pw.md's "each contract green on a sound end state and red on its breach" holds for 11 of 18 contracts;
  fix: one green and one red test for each of the seven (thin contracts still need them).
- rw-3 MINOR tests/persona/check.py:346-355: T6 passes when `blocks.notify` shown/closed/accepted is non-empty in
  the end snapshot only, so any notice from before the task makes it green; fix: compare the notice ids with
  those of `before` (plausible from the code, not run).
- rw-4 MINOR tests/persona/round.sh:33: `sha` is the repo HEAD, but the binaries are those of target/debug,
  which may be older than HEAD, so "one sha per round" can name a commit the binaries are not from; fix: record
  the sha only when `target/debug/chora` is newer than HEAD's tree, or write `sha256sum` of the copied binaries.
- rw-5 MINOR tests/persona/check.py:79,198: `Run.env_file` and `Ctx.since` are never read (dead); fix: delete both.
- rw-6 NIT tests/persona/round.sh:63: the status of `check.py start` is ignored, and an unquiet first snapshot
  leaves no `last` snapshot (the first `after` then only takes it); fix: `|| echo ... >&2` and retry once.
- rw-7 NIT tests/test_acts.py: no direct test that an attached Session leaves the folder (only planted and smoke
  show it by running on); fix: attach to a temp env in a unit test and assert the folder exists after `__exit__`.
- rw-8 NIT tests/persona/check.py:60: POWER omits `loginctl terminate-*`, so `T4:power` (cancelled) passes after a
  Log out; fix: add `loginctl terminate` to the pattern and a test line.

## r2

Head `5755b578887736cbb782e3e35051b39eaa6f3514` on claude/pw (merged origin/claude/autonomous-dev-session-3w-i33pg7 before the gate).

1. rw-1 fixed. Red first: cb67dd3b (selftest line: `bash tests/persona-selftest.sh` fails with `screenshot.sh wrote no file under ... new on the host: /root/Pictures/...`; the stray file was removed). Green: a8a6cddc (session.sh: XDG_PICTURES_DIR=$dir/pictures, xdg-user-dir stand-in). `bash tests/persona-selftest.sh`: 6 ok lines incl. "screenshot.sh in the environment of the core writes under the session directory"; `find /root/Pictures -name '*.png'` finds 0.
2. rw-2: one green and one red test each for X2, L1, L2, L3, M2, C2, C3 in PersonaContracts. Count: check.py's table has 17 task entries (T1-T7, X1-X2, L1-L3, M1-M2, C1-C3), T4 with four parts; r1 and the review said 18, I find 17. All 17 now have green and red tests (`python3 -I tests/test_acts.py`: 134 tests OK, 125 before: +7 contracts, +1 T6, +1 T4 power).
3. rw-3: T6 compares notice ids with `before` (`test_t6_an_old_notice_alone_is_red`). rw-5: Run.env_file and Ctx.since deleted. rw-8: POWER includes `loginctl terminate` (`test_t4_power_a_log_out_is_a_power_call`).
Backlog (no code): rw-4 (sha vs binaries of target/debug), rw-6 (status of `check.py start` ignored in round.sh), rw-7 (unit test that an attached Session leaves the folder).

Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,gate-lib,acts,doors`: all 7 PASS (pre-commit, gate-lib 67 s, build, test 182 s, ci-check 89 s, acts 181 s, doors 328 s), rc 0. `bash tests/persona-selftest.sh` ok. `ROUND_DIR=$(mktemp -d) tests/persona/round.sh smoke`: tiling, xp, mc, construct each `task T7 pass breaches 0`.
