# logging r2 (claude/logging-r1, fix round after review-logging-r1)

Base: merged claude/autonomous-dev-session-3w-i33pg7 be205f89 (874cd549). Head: see the last line.

| Item | Red commit | Fix commit | Cell (red → green) |
|------|-----------|-----------|--------------------|
| M1 lock, no rotate by a second writer | 894840e7 | 7afccdd2 | chora-scene `logs::a_second_writer_leaves_the_file_of_the_first` (red: second open succeeded); chora `log_file::a_second_chora_leaves_the_log_of_the_first` (red: no "no log file", first's fd moved); chora-voice `log_file::a_cli_call_leaves_the_log_of_the_agent` (red: `--version` moved voice.log) |
| M1 voice opens its log only as the agent; parse args first | 894840e7 | 7afccdd2 | same voice cell (--version, --help, --print-config, gate) |
| M1 tests/headless-session.sh private XDG_STATE_HOME | (script) | 7afccdd2 | gate steps that use the script |
| M2 + MINOR 4, 5, 6 chora-report | a009894e | dd23f466 | choractl `chora_report` (red: bundle 0644; then block YAML, Api-Key, AUTHORIZATION, block scalar, USER:USER, home with `+()`, symlinked tiling.yaml) |
| MINOR 2 O_NOFOLLOW, fix modes | 2e125b77 | 749f3bf4 | `logs::a_symlink_is_not_followed_and_old_modes_are_fixed` (red: dangling chora.log symlink opened) |
| MINOR 7 one subscriber function | (mutation) | 0668da38 | dropping `filter_fn(ours_or_info)` from `subscriber()` turns `log::tests::a_dependency_logs_no_key_press_at_trace` (chora) and `chat::tests::the_arguments_of_a_tool_call_stay_out_of_the_log` (voice) red |
| MINOR 1 rmcp capped at warn | 5c40776f | 651ebd5f | voice `log::tests::rmcp_logs_only_warnings_and_errors` (red: info line passed with trace filter) |
| MINOR 8 w16 lines 147, 153 | 8cc244eb, 48b68820 | — (test only) | w16 run, see below |
| MINOR 12 docs/logs.md | — | 19b84eb5 | docs only |

Notes and deviations:
- The dir chmod of MINOR 2 landed in the M1 fix (7afccdd2); the MINOR 2 red was still red on the
  symlink and the 0644 file.
- MINOR 7 is a refactor, so its evidence is the mutation run, not a red commit.
- Line 147 now looks for the question in the fake TTS log (`tts.log`); line 153 looks for a
  `tool run_program Nms` line after the `confirm run_program:` line of the question. In a run that
  lost sync, the first form counted the late control run (48b68820 fixes that).
- w16 run (twice on this branch, once on be205f89): from step 1 on it fails the same way on both
  (mc shows no No button for the physical question, then each turn lags one step). That is a
  pre-existing failure, not from this lane. In the run on this branch, the question "Shall I start the program
  foot -e sh?" is in tts.log, and `confirm run_program: The user declined…` comes with no
  run_program tool line after it, so line 153 stays silent. Line 147 reported a miss because of
  the lag. Backlog candidate: w16 step 1 (mc No button).
- Not run: shellcheck (not installed on this host); dist/chora-report passes `dash -n`.
- MINOR 3, 9, 10, 11 not touched (backlog L10-L13).

Gate: tests/merge-gate.sh (full) on 48b68820: all 28 steps passed (run dir /tmp/merge-gate.EZxNIx).
tests/packaging-files.py: 0 gaps. Head pushed: claude/logging-r1 48b68820.
