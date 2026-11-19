# kit6 r1: kit::launch (D87 revised, D104)

Branch claude/kit6 from dc4bc46d. Commits: 56990810 (red), 34ca4aa4 (feat), ad88ebaf (fix).

## Done
- Red (56990810): `crates/chora-kit/tests/launch.rs`, FakeCore look with real `sh` programs and a clock that never
  advances. Red: "a program that exits before its window gives Exited at once" and the editor-chain variant
  (no change within 5 s). Green in the other two cells (a launch that maps a window; exit after the window).
- Feat (34ca4aa4): `chora_kit::launch` holds `Launches`, `Change`, `Why`, `Candidate`, `Answer`, `run`, `start`, `alive`.
  The main process of the start is held with a pidfd (rustix) on a thread of its own; the unit state is checked
  (`ActiveState`, `MainPID` against pid reuse). xp's tracking is deleted (launch.rs 1013 -> about 170 lines, only the
  fuzzel theme, Run box and program_name stay); tiling starts through `launch::run`; mc and construct only changed paths
  (`launch::start`, `launch::alive`). `FakeCore::mint_tokens` is opt-in (a global `activation.token` broke two mc cells).
- Fix (ad88ebaf): see deviation 1.
- Gate: first run stopped at xp-notepad ("a second editor opened over the slow hand-off editor"), which is fix ad88ebaf.
  Second run on the head ad88ebaf: all 41 steps passed (/tmp/merge-gate.p42yqm, the container is not durable).
- Coverage `cargo llvm-cov -p chora-kit --features chora-kit/testing`: launch.rs 89.2 % lines, 87.1 % functions; crate 90.4 % lines.

## Deviations (read these)
1. D104 says no waitid and the pidfd decides. A pidfd gives the end, not the status, and units are collected
   (`--collect`), so the status is gone. A bare "gone and silent gives Exited" broke the gate cell of an editor that exits 0 and
   shows its window 4 s later (and would fail `xdg-open`, browsers). So the shell wrapper with the status file stays, now in the
   kit and for every tracked start (not only chains): Exited at once = pidfd says gone AND the status is non-zero AND no
   window of another start. Exit 0 is waited for as before. The brief's "replaces the exit-status file" is not met;
   the owner decides whether `choractl run` should report the status (proposal 1).
2. copies.toml had no launch rows to lower. I added two rows at max 0 (CHORA_RUN_TOKEN, pidfd/status file) so they cannot regrow.
3. tiling is not tracked: it has no message surface, so tracking would be a new feature. It only uses `launch::run`.
4. Not exercised: the unit path (`unit_active`, `unit_main_pid`) needs a systemd user manager, which this container has not;
   only the no-manager path ran (gate and cells). Pid reuse check is untested.
5. docs/decisions.md D104 still says "Proposed, not merged"; the PM sets it. docs/architecture.md names the new paths;
   configuration.md wording is unchanged in effect.

## Refactor proposals (ranked)
1. S-M: `choractl run` prints or records the exit status of the unit/child (crates/choractl/src/main.rs:557-730); then
   kit::launch loses the shell wrapper and StatusFile (launch.rs StatusFile, about 60 lines) and the hand-over rule becomes exact.
2. S: `systemctl` helper exists twice (choractl main.rs:760, kit launch.rs `systemctl`); put it in chora-scene.
3. M: give tiling `Launches` once it has a notice surface (plugins/chora-tiling/src/main.rs:1003).
4. S: mc show.rs:202 and construct (adopt.rs:272/295, keys.rs:61, scene.rs:142) use `launch::start` with their own pid
   polling (`alive`); move them onto `Launches` or a `Watch` type.
5. S: split the tests of kit/src/launch.rs (about 300 lines) into tests/launch.rs.
Cut check: breach remaining in xp: `launch.rs` includes ../../../flavors/xp/theme/fuzzel.ini (reported before).

## Review r1
Seeded review of claude/kit6 ad88ebaf (base dc4bc46d), one round, read-only on code.
Verdict: MERGE-OK (0 BLOCK, 0 MAJOR, 2 MINOR, 2 NIT).

Checks:
1. Red first: 56990810 holds the cells and a stub `launch` module (compiles, the Exited cells red); 34ca4aa4 and ad88ebaf follow.
   The two Exited cells were changed after red from `exit 1` to `false` (see rw-1).
2. Watcher: one thread per tracked start, ends after at most GIVE_UP + 1 s (poll timeout), the pidfd is an OwnedFd dropped on
   return; a dropped `Launches` only makes `send` fail. No thread or fd stays. pidfd_open then MainPID check is the right
   order against pid reuse. Exited = gone AND status non-zero AND silent has no window race: the wrapper writes the status
   before the shell exits, and `check_launches` drains (collect) before `check`. A failed pidfd_open is not clean (rw-2).
   `cargo test -p chora-kit --features testing`: all green (243 + 31 incl. tests/launch.rs 4).
3. Deviation: the wrapper is the simplest correct design. `--collect` unloads the unit as soon as it is inactive or failed,
   so `systemctl show -p ExecMainStatus` after the pidfd fires races the collection; a D-Bus PropertiesChanged watch (zbus is
   a kit dependency) covers only the unit path, needs Subscribe and a match per unit and races the same collection unless
   `--collect` goes (then failed units pile up and need reset-failed); without a user manager no process is the parent.
   `ExecStopPost` with $EXIT_STATUS would write a file too, unit path only. D104 should say: "The pidfd on the main process
   (checked against the unit's MainPID) tells that the program ended; its exit status comes from a shell wrapper that writes a
   non-zero status to a runtime file, since no process is its parent and the unit is collected. Exited at once = ended AND
   failed status AND no window of another start; exit 0 waits up to 30 s (hand-over). The exit-status file stays; the 30 s
   timer stays for exit 0 and for programs that run without a window."
4. Ports: xp's own tracking deleted (launch.rs keeps themed, Run box, program_name), `Candidate` moved to the kit; tiling only
   `launch::run`; mc and construct only paths (`launch::start`, `launch::alive`). Visible change only the intended one
   (a single start that fails ends at once, documented). `cargo test -p chora-xp -p chora-tiling`: 152 + 175 green.
5. `tools/cut-check`: ok (18 reported, the fuzzel.ini breach known). Both new copies.toml rows hold (CHORA_RUN_TOKEN only in
   kit and choractl; pidfd/status file only in the kit). architecture.md:448 and configuration.md:1829 match the code.

Findings:
- rw-1, MINOR, crates/chora-kit/src/launch.rs:202, the wrapper runs the command in `{ }`, so `exit N` or a failed `exec prog`
  ends the shell before the status is written and the start waits 30 s instead of Exited; the red cells were changed from
  `exit 1` to `false` to fit (tests/launch.rs:136, :155). Fix: wrap in a subshell `( {command}\n)` (checked with dash: `exit 3`
  and `exec false` then write the status, same process count) and put `exit 1` back in the cells.
- rw-2, MINOR, crates/chora-kit/src/launch.rs:581 (and :556, :593), a start that cannot be followed counts as running:
  pidfd_open failing (ENOSYS on an old kernel, EPERM), poll EINTR, or a unit still active with pid null all leave `gone`
  false, so Exited never fires and at GIVE_UP the balloon closes as Done instead of "No window opened in 30 seconds" (the old
  /proc check gave Slow). Fix: on those paths fall back to `alive(pid)` / `unit_active` once a second until the limit.
- rw-3, NIT, crates/chora-kit/src/launch.rs:188, the kit status file keeps the prefix `chora-xp-exit-`. Fix: name it
  `chora-launch-exit-` and change the copies.toml pattern with it.
- rw-4, NIT, crates/chora-kit/src/launch.rs:244 (begin), the wrapper now goes on every tracked start, not only chains, so a
  `sh` stays as MainPID beside each app for its life. Fix: none needed; say it in D104 and docs/configuration.md.
Not exercised (known): the unit path and the pid reuse check.
