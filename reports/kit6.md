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
