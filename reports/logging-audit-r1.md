# Logging audit r1 (rc row 9a, backlog L1)

Base: claude/autonomous-dev-session-3w-i33pg7 at 98291088. Branch: claude/logging-r1.
Method: every `tracing` call, `eprintln!`/`println!` and panic path read per component
(grep of the call sites, then the lines with variables read in context), plus the
debug/trace call sites of smithay (rev cf43a02) and rmcp that the filters let through.

Ranking: secrets, typed text, chat text or voice transcripts in a log = BLOCK; a crash
or refused request without a log line = MAJOR; noise at info or unstructured lines = MINOR.

## Components

| Component | How it logs | Levels and target | Where it goes (before r1) | Where it goes (r1) |
|---|---|---|---|---|
| chora (core) | `tracing`, 65 call sites; own panic hook and signal last words (`words.rs`) to fd 2; hang watchdog | info: start, end, lock, grants, layouts; warn: a failed output, frame, client, screencopy; error: no image thread, failed startup command; debug: import skips. Target: module path `chora::<area>` | stderr: journal (`chora@.service`) or the terminal | stderr plus `$XDG_STATE_HOME/chora/logs/chora.log` (4 MiB, 3 old files, 0600); panic words also to the file (target `chora::panic`) |
| chora-scene | 2 `eprintln!` (protocol or build mismatch, once per process) | no level | stderr of the client | unchanged |
| chora-kit | 15 `eprintln!`: refused grab (once per key), unreadable settings, notify and launcher failures, volume thread | no level, prefix `chora-NAME:` | stderr of the plugin: journal of its unit | unchanged |
| chora-draw | 4 `eprintln!`: font and render failures | no level | stderr of the plugin | unchanged |
| plugins (tiling, xp, mc, construct, notify, assistant) | 128 `eprintln!` with the prefix `chora-NAME:`; panics: the default hook | no level, no target | stderr: journal of the flavor units | unchanged (journal; `chora-report` bundles it) |
| services/voice | `tracing` through `log.rs` (one call site per level), 90 call sites | info: turn timings, confirm outcomes, tool names, agent states; debug: details; target `chora_voice::log` for all lines, the first word names the stage | stderr: journal (`chora-voice.service`) | stderr plus `voice.log`; panics through `tracing` |
| choractl | `println!` results, `eprintln!` errors; no log | n/a (a CLI) | the terminal | unchanged |
| dist scripts (`chora-session`, `chora-flavor`, `chora-xwayland`) | `echo ... >&2` on failure | n/a | journal | unchanged |

## Findings

| # | Rank | Component | Finding | State |
|---|---|---|---|---|
| L1 | BLOCK | voice | At debug the log held the text of each utterance (`turn=N text="..."`), of words to a sub-agent, over a reply and queued, each reply sentence (`say: ...`), each LLM round, a cut answer, a coding task and its result, a dropped ASR echo, and the first 300 characters of the arguments and result of each tool call. A log file or report would carry them. | fixed d4666f3e: lengths only (`chars=N`); tool calls log name and time |
| L2 | BLOCK | voice | rmcp logs the arguments and result of each tool call at debug and trace (`CallToolRequestParams { arguments: ... }`); `RUST_LOG=debug` let them through. | fixed d4666f3e: below info only `chora_voice` targets |
| L3 | BLOCK | chora | smithay logs the keysym of each key press at trace (`input/keyboard/mod.rs:1168`, `Calling input filter sym=...`): `RUST_LOG=trace` wrote typed text, passwords too. | fixed 27c5b16b: below info only `chora*` targets |
| L4 | MAJOR | chora | A refused IPC request (missing capability, admin socket only, client socket rule, limits, invalid params) answered the client and left no log line. | fixed ab747722: one line per error answer, `refused conn= client= code=`, at info; codes 2 and 5 (races) at debug; never the message or the parameters |
| L5 | MAJOR | chora, voice | No log file: outside a systemd session (nested, `chora-session` from a TTY without journal) a crash left its words only on a terminal; the voice panic went only to stderr. | fixed 5c1210a7, 7cbd7456: rotating files; panics in the file |
| L6 | MINOR | plugins, chora-kit, chora-draw | 149 `eprintln!` lines: no level, no timestamp, no target (the journal adds time and unit). | open: backlog (move to `tracing` with one target per plugin) |
| L7 | MINOR | voice | One target `chora_voice::log` for all lines (the wrappers in `log.rs` exist for the clippy complexity guard); the stage is the first word of the message. | open: documented in docs/logs.md |
| L8 | MINOR | voice | `agent NAME: COMMAND in DIR` and `mcp: restart with CMD` log the argv of a configured command at info; a key passed in argv (not in env) would show. | open: settings docs say keys go in env or `*_file`; `chora-report` hides token-like strings |
| L9 | MINOR | chora-kit | `notify "SUMMARY": err` prints the summary of a failed notification; all callers send chora's own words (checked: xp, volume, mc, tiling). | open, no change |

Checked and not a finding: compositor info lines (start shows 4 lines at info in 3 s
headless: keyboard, locale, socket, start); `process.rs` logs the argv of the startup
command (`chora-flavor NAME`); `grants.rs` logs capability changes (no secret); the voice
info lines (turn timings, confirm outcomes by tool name, settings path).

## Rules and their checks

| Rule | Check |
|---|---|
| No spoken or chat text at any level (voice) | 8 tests capture every level (`log::capture::all`): `utterances_stay_out_of_the_log`, `a_request_during_a_reply...`, `new_words_over_the_speech...`, `a_turn_runs_the_tool_calls...`, `the_arguments_of_a_tool_call_stay_out_of_the_log`, `a_transcript_that_is_the_prompt_is_dropped`, `a_task_runs_in_a_clone...`, `playback_waits...` |
| No dependency payload below info | `log::tests::a_dependency_logs_no_key_press_at_trace` (chora); the voice capture runs through the same filter (the rmcp line failed the tool test until the filter) |
| A refused request logs, without parameters | `ipc::tests::a_refused_request_logs_its_client_and_code_without_its_parameters` |
| Files rotate, 0600, under the state home | `logs::tests::*` (chora-scene, 4 tests) |
| Panic words reach the file once | `crates/chora/tests/log_file.rs` (headless core, `CHORA_TEST_PANIC`) |
| Report scrubbing | `crates/choractl/tests/chora_report.rs` (fixture: user name, home path, host name, Bearer, sk-, ghp_, base64, yaml keys) |

No compile-time check: the rules are about values, not macros; a `disallowed-macros`
ban would not tell a transcript from a stage name. The tests above run in the gate's
`test` step.
