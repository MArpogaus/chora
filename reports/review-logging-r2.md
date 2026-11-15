# review logging r2 (claude/logging-r1 48b68820)

Scope: `git log 150db8f0..48b68820 --first-parent` without the merge 874cd549 (be205f89).
Read-only review in /home/user/wt/rev-logging-r2. Builds in /tmp/claude-0/rev-logging2/target
and target-red. Probes in /tmp/claude-0/rev-logging2/{m1,p1,red,mut}.

## BLOCK

None.

## MAJOR

None.

## MINOR

1. **The bundle keeps the mode of an existing `-o` file.** dist/chora-report:129. `umask 077`
   only applies to new files. `tar -czf` truncates an existing file and keeps its mode.
   Probe: `touch pre.tgz; chmod 644 pre.tgz; chora-report -o pre.tgz` gives `-rw-r--r--`.
   docs/logs.md says "only you can read the bundle (mode 0600)". Fix: `rm -f "$out"` before
   tar, or `chmod 600 "$out"` after. Gate: chora_report.rs checks a new file only. Missing
   gate: a pre-existing 0644 target.
2. **The secret key list still misses common shapes.** dist/chora-report:85, 96-121. Hostile
   fixture (p1/h): these stayed in clear text:
   - TOML table `[llm.headers]` then `X-Custom = "S19toml"`. The block rule is YAML only.
   - keys `cookie`, `private_key`, `credentials`, `auth`, `PASS=`, `api key` (with a space).
   - credentials in a URL: `postgres://bob:S9urlpass@db/x`.
   - a dotted key with a block value: `llm.api_key:` then an indented `S10dotted`. The awk
     rule is anchored and allows no `.` before the secret word. The sed rule hides only the
     key line.
   - a short key after `?key=` (under 36 characters). A 39-character Google key is hidden
     only because `key=` makes the run 40+ characters long.
   chora's own settings are YAML, and every voice secret key (`*_api_key_*`, `*_headers`)
   is covered, so this is not M2 again. docs/logs.md says "It cannot know every private
   word". Fix (cheap): add `cookie|credential|private[-_]?key|passw?|auth` and
   `//[^:/@]+:[^@]+@` to the rules. Gate: the chora_report fixture (YAML only).
3. **The panic test of the core log is flaky under load (old, not from this round).**
   crates/chora/tests/log_file.rs:89-92. `words::install` spawns the panicking thread before
   `log::init` and before "chora runs". The test waits only for the panic line and then
   requires "chora runs". Probe: my first `cargo test -p chora --test log_file` ran while
   another build used all 4 cores. It failed at line 92: the file had the panic line, but not
   yet "chora runs". Four later runs passed. Fix: wait for both lines. Gate: this test. It
   is in the merge gate, so expect rare red runs there.
4. **The folder chmod follows a symlink and can make the log fail.**
   crates/chora-scene/src/logs.rs:64. `fs::set_permissions(dir)` works on a path. If `logs`
   is a symlink, the target gets mode 0700. If the folder belongs to someone else, EPERM
   stops the log file, although the folder is writable. There is a small window between
   `DirBuilder::create` and the chmod. An attacker needs write access to `state/chora`, so
   this is defence in depth. The file side is fine: `O_NOFOLLOW` and then `fchmod` on the fd,
   with no window. Fix if wanted: open the folder with `O_DIRECTORY|O_NOFOLLOW` and `fchmod`
   it, and treat a failed chmod as a warning. Gate: none.

COSMETIC:
- docs/voice.md:779 still names the default filter `info,tract_linalg=warn,rmcp=warn`. The code
  (services/voice/src/log.rs:19) is now `info,tract_linalg=warn` plus the rmcp cap. The effect
  is the same.
- docs/glossary.md:25 says the log moves "at each start". Now that is true only for the lock
  holder.
- docs/logs.md names "a nested or a test session" as the second chora. A second real session
  of the same user (another VT, another `WAYLAND_DISPLAY`) also gets no file. Its log is then
  only in its journal. A state home on NFS without lock support gets `ENOLCK` and no file.
  Both are acceptable. One sentence would say it.
- hide_blocks over-hides: after `- token: x`, the next list item at the same indent
  (`- name: coder`) and its lines become `(hidden)`. That leaks nothing.
- tests/headless-session.sh:76 overwrites a caller's `XDG_STATE_HOME`. tests/chat.sh,
  voice-acp.sh and voice-events.sh set it, but their voice runs outside chora, so nothing
  breaks.
- `cp -RL` follows a symlinked folder anywhere, for example `config/chora/x -> ~`. That copies
  all YAML, TOML and JSON files under home into the bundle (scrubbed). It is self-inflicted
  and only matters with docs that say "read it before you send it".

## Checked and fine

- **M1 closed.** Probe (m1/): chora A ran headless with a private state home and `-- sleep 300`.
  Four more chora starts on other runtime dirs each logged
  `WARN chora::log: no log file: another process writes chora.log`. After that,
  `/proc/A/fd` still had `chora.lock` and `chora.log` (not deleted), and no `.log.1` existed.
  Three `chora-voice --version` calls and one `--print-config` left the folder unchanged. The
  child `sleep` held no log or lock fd (std opens with `O_CLOEXEC`). After `kill -9 A`, the
  next start got the lock and moved A's file to `chora.log.1`. So the kernel releases the
  flock on a crash, and a stale lock file does no harm.
- The lock is `NAME.lock` in the logs folder, mode 0600, opened with `O_NOFOLLOW`. It is held in
  `File::_lock` for the life of the subscriber, which is the life of the process. With
  `chora.lock` as a symlink, chora warns
  `no log file: Too many levels of symbolic links (os error 40)` and goes on. The symlink
  target is not created.
- Voice: `log::init(args.command.is_none() && !args.print_config)` runs after `Args::parse()`,
  so clap exits on `--version` and `--help` before init. Probe: with `voice.lock` held by
  `flock(1)`, the agent warned `no log file: another process writes voice.log` and kept
  running. The old `voice.log` stayed unchanged.
- `File::try_lock` is in std from 1.89, the `rust-version` of the workspace. It is `flock(2)` on
  Linux.
- **M2 and MINOR 4, 5, 6 closed.** Hostile fixture (p1/), with home
  `h/a.b+c(d)|e{1}?x` and real user `root` and host `vm`:
  - hidden: block YAML (`llm_headers:` with indented keys, a quoted `"Proxy-Authorization"`,
    a comment line inside the block), block scalars `|` and `>`, flow maps
    `{Authorization: x}` and `{"X-Goog-Api-Key": "y"}`, `LLM_API_KEY`, `'Token'`,
    `headers :`, TOML `API_KEY =`, `export GITHUB_TOKEN=` in a .conf file.
  - `agents.env` is removed.
  - `root:root:root:root` becomes `USER:USER:USER:USER`, `vm,vm` becomes `HOST,HOST`, and
    `rootx` is kept.
  - the home path with ERE characters becomes `~`.
  - a symlinked file and a symlinked folder go in as content.
  - a symlink loop (`loop -> .`) prints "some settings files could not be read". The other
    files are still copied.
  - the bundle is 0600 and its folders are 0700 under `umask 022`.
- **MINOR 1 closed.** `ours_or_info` caps every target that starts with `rmcp` at warn, and
  `filter_fn` is a global layer, so `RUST_LOG=trace` cannot lift the cap.
- **MINOR 2 closed** for the file (`O_NOFOLLOW`, `set_permissions` on the open fd, so no race).
  A dangling `chora.log` symlink fails the open and nothing is created. The folder: see MINOR 4.
- **MINOR 7 closed.** Mutation (mut/): I removed `.with(filter_fn(ours_or_info))` from both
  `subscriber()` functions. Then `chora log::tests::a_dependency_logs_no_key_press_at_trace`,
  `chora-voice chat::tests::the_arguments_of_a_tool_call_stay_out_of_the_log` and
  `log::tests::rmcp_logs_only_warnings_and_errors` all fail. Both `init()` functions build
  their stack only through `subscriber()`.
- **MINOR 8, w16 (simulated, not run).** I did not run w16 itself. The lane reports a failure
  at step 1 that was already there before this branch.
  - Line 147 now greps the fake TTS log (w16-fakes.py:145 writes each text). The text is the
    run_program rule question `Shall I start the program {program}?` (config.rs:391). It
    stays quiet when the question was asked. It fires when it was not.
  - Line 153 awk on synthetic logs: a "declined" confirm with no tool line stays quiet. A
    `yes` or `allowed for the session` confirm followed by `tool run_program 15ms` fires. A
    late control run before the question stays quiet. `tool NAME Nms` is logged only after a
    passed confirm (chat.rs:1003-1010). The `in allow` case is caught by line 150.
- **MINOR 12 closed.** docs/logs.md now names argv at info, URLs in errors,
  `CHORA_VOICE_LLM_DUMP`, the lock, the voice CLI calls, O_NOFOLLOW and the modes. The
  chora-report text matches the script. The timestamp example now has microseconds.
- **Red first** (each test commit on a `git archive` of its own tree):
  - 894840e7: chora-scene `a_second_writer…` fails; voice `a_cli_call_leaves_the_log_of_the_agent`
    fails at line 30; chora `a_second_chora_leaves_the_log_of_the_first` fails at line 60.
  - a009894e: fails on the bundle mode. With `umask 077` added by hand, it still fails on the
    fixture content (line 171).
  - 2e125b77: `a_symlink_is_not_followed…` fails at line 235.
  - 5c40776f: `rmcp_logs_only_warnings_and_errors` fails at line 166.
  - 0668da38 is a refactor, so the mutation above is its evidence.
- Green at 48b68820: chora-scene `logs::` 6, choractl `chora_report` 1, chora-voice `log_file` 1
  and all 236 bin tests, and chora `log_file` 2 (4 of 5 runs, see
  MINOR 3).
- No new log call in production code. The only new text is the error
  `another process writes NAME.log`. io errors from these fs calls carry no path. The new
  `tracing::` lines are in tests only.

## Gates missing

- chora-report: no test with an existing `-o` file (MINOR 1), and no TOML table or URL
  credential shape (MINOR 2).
- No test of a symlinked or foreign-owned logs folder (MINOR 4).
- w16 is outside the merge gate, so lines 147 and 153 are checked only by hand.
- shellcheck is still not installed on this host, so dist/chora-report has `dash -n` only.

## Verdict

MERGE-OK. M1 and M2 are closed, and so are MINOR 1, 2, 4, 5, 6, 7, 8 and 12. Each fix had a red
test first, and I confirmed each red. Four MINOR remain. MINOR 1 (bundle mode of an existing
`-o` file) and MINOR 3 (flaky panic test) are a few lines each and fit the next round or the
backlog.
