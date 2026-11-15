# review logging r1 (claude/logging-r1 150db8f0)

Base 98291088. Read-only review in /home/user/wt/rev-logging-r1. Tests run there (own target/).

## BLOCK

None.

## MAJOR

**M1. Any second process moves the live log of the running one. The running log then ends up in an unlinked file.**
- Where: crates/chora-scene/src/logs.rs:56-68 (`open_in` always calls `shift` when `NAME.log` is non-empty). services/voice/src/main.rs:141 (`log::init()` runs before `Args::parse()`, so `chora-voice --version`, `--help`, `--print-config` and every subcommand open and rotate `voice.log`). dist/chora-report:52-58 runs `chora-voice --version`. tests/headless-session.sh:76 starts chora with the person's real `XDG_STATE_HOME`. Many gates use that script.
- Failure, reproduced: chora instance A ran with a private state home. Four more headless starts followed. After that, `/proc/A/fd/3` pointed to `chora.log.3 (deleted)`. A's log, including any later crash words, went into a deleted file. The same thing happens to a live session when a developer or agent runs the gate scripts, a nested chora, or `chora-report` three or four times. That defeats L5 ("the log of a crash stays next to the new run"). Every `chora-report` run also rotates the live `voice.log` once.
- Earliest gate: none. Missing gate: an integration test where a second instance starts while the first runs, and the first's fd must still be `chora.log`.
- Fix: take a lock (`flock` on `NAME.log` or `NAME.lock`) and rotate only while holding it. A second instance writes `NAME-PID.log` or no file. In voice, open the file only in the agent's run path. In tests/headless-session.sh, export a private `XDG_STATE_HOME`. tests/ belongs to the flakes lane, so coordinate that change.

**M2. chora-report leaves header secrets in block-style YAML.**
- Where: dist/chora-report:93. The key rule works on one line only. `api_?key` does not match `Api-Key`.
- Failure, reproduced: a fixture `voice.yaml` contained
  `llm_headers:` followed by the indented lines `X-Api-Key: abc123secretzz` and `x-goog-api-key: AIzaSyZZZZZZZZZZZZZZZZZZZZ`. The bundle had `llm_headers:(hidden)` and then both values in clear text. Block style is how people write YAML by hand. It is also the default output of `serde_saphyr::to_string`, which `chora-kit` settings use (crates/chora-kit/src/settings.rs:203). So a file saved through `settings.set` has this shape. `llm_headers` is exactly where the API keys of the voice agent go. docs/logs.md:73-77 promises these values are hidden.
- Earliest gate: crates/choractl/tests/chora_report.rs. Its fixture has only the flow form `{X-Key: secret-value}`. Add the block form and hyphenated `X-Api-Key`.
- Fix: in sed or awk, hide every more-indented line under a key that matches `headers|token|secret|password|api.?key|authorization`. Allow `[-_]?` in `api.?key`.

## MINOR

1. **Dependency lines at info are not clamped.** services/voice/src/log.rs:21 and crates/chora/src/log.rs:18 let every target through at info and above. A set `RUST_LOG` (for example `RUST_LOG=debug`) replaces the default `rmcp=warn`. rmcp then logs `received notification ?notification` at INFO (rmcp-3.9.0 src/service.rs:1637), with the full payload of a progress or log notification from any configured MCP server. The same applies to `Service initialized ?peer_info`. Fix: cap `rmcp` at warn inside `ours_or_info`. Gate: none. The capture test mirrors the filter but sends no notification.
2. **The log file follows symlinks, and an existing folder keeps its mode.** crates/chora-scene/src/logs.rs:111-117 has no `O_NOFOLLOW`. Reproduced: a dangling `chora.log -> ../elsewhere.txt` made chora create and write `elsewhere.txt`. `DirBuilder::mode(0o700)` applies only to folders it creates. An existing `logs/` with mode 0755, or an existing empty `chora.log` with mode 0644, is kept. To exploit this, another user needs write access to the folder, so this is defence in depth. Fix: `custom_flags(libc::O_NOFOLLOW)`, plus `fchmod`/`set_permissions` on the dir and file.
3. **A failed rotate cascades.** logs.rs:72-77 and 81-83: if `create` fails after `shift`, `size` stays above `max`. Each later event shifts the old files again: `.1` goes to `.2` and then to `.3`, the oldest are lost, and then `rename(path)` fails with ENOENT. tracing-subscriber also prints an internal error to stderr for each event. Fix: on error, drop the file writer (`size = 0`, or an `Option` handle).
4. **chora-report misses a repeated user or host name.** dist/chora-report:91-92. The trailing boundary character is consumed, so with `g` the next match has no leading boundary. Reproduced: `root:root` became `USER:root`. `quote()` (line 86-88) escapes BRE characters, but sed runs with `-E`. A home path with `+ ? ( ) { } |` is not matched, so it is not scrubbed. Fix: use lookaround-free boundary handling (loop or `\b`-style via two passes) and escape the ERE set.
5. **chora-report keeps symlinks.** dist/chora-report:79-82 and 98-101. `cp -R` copies symlinked settings files (stow and other dotfile managers) as symlinks. `find -type f` skips them, so they are neither filtered nor scrubbed. Reproduced: the bundle held `tiling.yaml -> /…/dotfiles/tiling.yaml`. The target path is not scrubbed (it can hold home path and user name), and the file content is missing. Fix: `cp -RL`, or `find -type l -delete` after copying with `-L`.
6. **The chora-report bundle is world-readable.** dist/chora-report:103 creates the bundle under the default umask (0644 in the test), while the logs are 0600. Fix: `umask 077` at the top.
7. **Red first holds only for L1, L2 and L4.** L1 reproduced on a `git archive` of 0f5e1f08: 8 FAILED, 228 passed. The tool-call test output also held rmcp payload lines, which covers L2. L4 (cdf98806) is red by construction: it requires a line with "refused", and none existed. For L3 (27c5b16b), L5 (2ded0e4c, 5c1210a7) and chora-report (47f2ce7a), each test landed in the same commit as its fix. The report says they were run red, but the history cannot show it. Also, `log::tests` and `capture::all` test `ours_or_info` and a copy of the subscriber stack, not `init()`. Dropping `.with(filter_fn(ours_or_info))` from either `init()` keeps every test green. Gate missing: a test that builds the subscriber through the same function `init()` uses.
8. **tests/battle/w16.sh now gives false or silent results.** Line 147 greps `say: Shall I start the program foot`, which is now `say: chars=N`, so it always reports a false "endangers". The lane named this one. Line 153 (`tool run_program.*foot`, "foot ran after a no") can no longer match at all, because tool arguments are no longer logged. So a security check now passes silently. The lane did not name this one. w16 is not in the gate and tests/ belongs to the flakes lane, but both lines need another signal, for example the confirm event or a tool event with the program name only.
9. **The voice panic hook loses detail and can carry user text.** services/voice/src/log.rs:44 replaces the default hook with `panic: {info}`. That drops the thread name and the backtrace, even with `RUST_BACKTRACE=1`. A panic message can hold user text: a str slicing panic quotes up to 256 bytes of the string. With this branch that text reaches voice.log and the report. Chora's own hook has the same exposure. Fix: keep the thread name and backtrace, and cut the message.
10. **L5 is only partly closed.** Crash words from signals (`words.rs` signal handlers write to fd 2 only) still do not reach chora.log. Each restart also shifts the log once, so under a restart loop the first crash log is gone after 3 more starts. Say both in docs/logs.md.
11. **Refused requests can flood the log, and one refusal path is missed.** crates/chora/src/ipc.rs:1702: each refused request logs one line at info, with no rate limit. A client looping on a method it lacks fills 4 MiB and pushes useful history out. ipc.rs:384-394: the LIMIT_REACHED error that `respond` builds for an oversized answer is created after the `refused` call, so it logs nothing.
12. **docs/logs.md overstates.** Line 17 ("Each start moves…") is also true of every `chora-voice` CLI call (M1). Line 61 says "No level logs a token, a key or a password", but `agent NAME: COMMAND` and `mcp: restart with CMD` log argv at info (audit L8), and reqwest errors print the full URL, so a `?key=` URL lands in voice.log. `CHORA_VOICE_LLM_DUMP` (docs/voice.md) writes full transcripts and is not mentioned under "What a log never holds".

COSMETIC: the docs/logs.md:31 example timestamp has milliseconds, but fmt writes microseconds. dist/chora-report sits at the top of dist/, while the other session scripts are in dist/session/.

## Checked and fine

- The clamp cannot be bypassed through `RUST_LOG`. `filter_fn(ours_or_info)` is a global layer combined with `EnvFilter` (both must allow). `RUST_LOG=smithay=trace` or per-target directives cannot re-enable a below-info line from a non-chora target. `log` records go through LogTracer `enabled()` with their real target. smithay key spans are trace or debug, so they are filtered too.
- No remaining voice call site prints an utterance, reply, task, tool arguments or result. They print lengths, names, timings or confirm outcomes (`yes/no/unclear`, the rule's refusal text). `agents.prompt` returns status words only. chora's own trace and debug lines carry no keysym or text. smithay's info and warn lines carry no key data.
- With an unwritable state home (`chora` is a file), chora warns `no log file: Not a directory` and keeps running (run). The same holds without `$HOME`/`$XDG_STATE_HOME`.
- A newly created file has mode 0600 (umask can only narrow it), and the folder is 0700 when it is created. The rotation numbering is correct. Disk use is at most 4 × 4 MiB per name, 32 MiB in total.
- Chora panic hook order: the words go to stderr first, then to the file through the `chora::panic` target. That target is kept out of stderr (log_file.rs checks that stderr has them once). `words::install` runs before `log::init`, and a panic before init is a no-op in the file.
- chora-report: `set -eu`, quoted variables, `mktemp -d` with a `trap` cleanup, no network access, `-o` with a relative path works, `dash -n` is clean (no shellcheck on this host). It scrubs home, user, host, Bearer/Basic, sk-/ghp_/glpat/xox, strings of 40+ characters, `?key=` in URLs, and one-line secret keys. Non-settings files are removed. It is installed 755 by both dist/arch/PKGBUILD:56 and crates/chora/Cargo.toml:79 (RPM assets). `--version` exists for chora, choractl and chora-voice.
- L4 logs the client and code as structured fields (`client` is Debug-quoted, so no line injection), never the message or the params. Codes 2 and 5 match `RpcError::GONE_SURFACE` and `UNKNOWN_NODE`.
- Tests at 150db8f0 (all pass): chora-scene `logs::` 4, choractl `chora_report` 1, chora lib `log::`/`refused` 12, chora `log_file` 1, chora-voice 235.
- Other users of the removed text: live-gate.sh dropped its two greps. tests/gate-handover-matrix.sh:401 and w16 `llm: stop` still match `llm: stop chars=N`. The `confirm: …`, `in allow, no question` and `mcp: restart with` lines are unchanged. Docs voice.md, logs.md, the glossary and the mkdocs nav are consistent with the code apart from MINOR 12.

## Gates missing

- No test with a second instance or a CLI call that rotates a live log (M1).
- The chora-report fixture lacks block-YAML secrets, hyphenated header names, symlinks and a repeated user name (M2, MINOR 4, MINOR 5).
- The tests check `ours_or_info` and a mirrored stack, not `init()` (MINOR 7).
- tests/packaging-files.py checks only flavors/ and plugins/*/hud, so its "0 gaps" says nothing about dist/chora-report.
- shellcheck skips dist scripts without `.sh` (the lane named this).
- There is no compile-time check, and that is fine: these rules are about values.

## Verdict

FIX. There are 2 MAJOR findings: M1 (log rotation by any concurrent process or CLI call) and M2 (block-YAML header secrets in chora-report). Also fix MINOR 1, 2, 5, 6 and 8 (line 153) in the same round, because each is a few lines.
