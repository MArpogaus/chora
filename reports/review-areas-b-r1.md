# Review areas b r1 (rc 6: mc, construct, assistant, voice)

Read-only review of `claude/autonomous-dev-session-3w-i33pg7` at head
66588880c9106f0276a438a6a827949f6a07b5f4. One round, four reviewers (one per area), each
finding checked against the code at the cited line; nothing was built or run. No HIGH found.
Format: id | severity | file:line | text | fix. `[sec]` = charter 11 / security.
Ids are global (mc 1-18, construct 19-38, assistant 39-59, voice 60-74). Per area, the old
ids in "earlier" refer to reports/review-plugins-r1.md.

Counts: mc 0 HIGH / 8 MEDIUM / 10 LOW; construct 0 / 10 / 10; assistant 0 / 12 / 9; voice 0 / 4 / 11.
Security MEDIUM: rab-60, rab-61 (voice). Security LOW: rab-9, rab-10, rab-70, rab-71, rab-72, rab-47 (sec-adjacent).

Charter 11: (a) confined choractl mcp holds (crates/choractl/src/mcp.rs:111,355; tools.rs:91);
(b) run_program confirmation holds for the default `confirm` only (rab-61); (c) harness spawns
only via `config.spawn`, `check_harnesses` rejects no `sandbox_cmd` without `unsandboxed` (tests
config.rs:1378,1412). mc, construct and assistant spawn no agents. Tests read, not run.

## chora-mc
1 | MEDIUM | main.rs:1-4546 | 4546 lines, `Mc` has ~45 fields; windows, docks, regions, agents, confirmations in one struct (MC-16) | Split into modules (windows/docks, regions, agents, page keys), tests in own files.
2 | MEDIUM | agent.rs:229-268 | `set_agent` commits fields before `show_agents`; a >4 KiB column fails `replace_panel` (main.rs:3160) and every later `agent.set` fails (MC-05) | Validate on a copy, commit after success, cut task/plan/question/log.
3 | MEDIUM | agent.rs:229-251 | agent count and fields unbounded (MC-20) | Cap count and field lengths.
4 | MEDIUM | agent.rs:686-710 (680) | `show_answers` removes `mc-answers` even if the group never existed, refusing the whole transaction (MC-07) | Track `has_answers`, remove only then.
5 | MEDIUM | fonts.rs:223 | `bearings` uncached: fc-match plus full font read on every geometry pass (main.rs:2180) (MC-11) | Cache like `metrics`.
6 | MEDIUM | region.rs:1844-1855 | `start_dock` drops the pid; a dock that never maps stays `starting` forever and blocks later starts (MC-10, MC-13) | Expire on pid exit, show failed starts.
7 | MEDIUM | main.rs:2791-2793, 3306, 3340 | `launch_scene` returns Ok when launch fails; `act` drops the result (MC-13) | Return an error or notice.
8 | MEDIUM | manage.rs:175-191, config.rs:360 | `default_scene` never validated; `remove_scene` can leave `current` on a removed scene (MC-12) | Require it in `Config::check`; update on rename.
9 | LOW [sec] | main.rs:609-612 | `scene.show` unprivileged but starts the scene's app; comment at 601 says program-starting methods are privileged (MC-17) | Make privileged, or document and pin in the test at 4187.
10 | LOW [sec] | main.rs:760-766 | `agent.remove`/`agent.focus` unprivileged, `agent.set` privileged (MC-18) | Make `agent.remove` privileged.
11 | LOW | agent.rs:546-547 | `n - 2` underflows for panel `agent-0` (MC-19) | `checked_sub`.
12 | LOW | agent.rs:205 / region.rs:362 | `URGENT` defined twice | One `pub const`.
13 | LOW | region.rs:322-330 | spawns `date` on the loop per tick/push/agent; failure silent (MC-21) | Format in-process.
14 | LOW | region.rs:334-350 | `to_next_minute` duplicated in construct doors.rs:222 | Move to chora-kit.
15 | LOW | agent.rs:165,594 | German Ja/Nein in an English page (MC-24) | Take from mc.yaml or English.
16 | LOW | collectors/system.py:177-199 | `--every 0` divides by zero (MC-25) | `parser.error` on <=0.
17 | LOW | collectors/morning.py:101-114 | cut cache or JSON without `current_condition` kills the script (MC-15) | try/except, validate before caching.
18 | LOW | collectors/system.py:169, tests.py:126 | first failed `panel.set` ends the collector (MC-26) | Skip a round, exit on repeated failure.
Earlier still open: MC-05,07,10,11,12,13,15,16,17,18,19,20,21,24,25,26. Fixed: MC-01,02,03,04,08.

## chora-construct
19 | MEDIUM | input.rs:662-674, overlay.rs:32-34, list.rs:181 | row ids and `pressed_row` index are relative to the visible slice, `choose` indexes the whole list; click on a scrolled list picks the wrong window (CON-B-01) | Add `first` to k (or absolute node ids).
20 | MEDIUM | cube.rs:893-897 | `lifts` tuple order swapped; floating plane lifted off the wrong neighbour (CON-A-01) | Swap, add a test, drop the always-true guard.
21 | MEDIUM | cube.rs:174 | stderr thread ends on a non-UTF-8 line, manager gets EPIPE (CON-A-02) | `read_until` plus lossy decode, keep draining.
22 | MEDIUM | cube.rs:1416 | `close_cube` drops the rejection of `try_apply(removed)`, nodes stay unlogged (CON-A-03) | Log it before `close_modes`.
23 | MEDIUM | overlay.rs:145-153 | `bearing` says "behind you" for `far` and `aside` (CON-A-06) | Match all states, ideally an enum from `space::sight`.
24 | MEDIUM | scene.rs:163-194 | `leave()` resets only some per-visit fields; help swallows walk keys after next load (CON-B-14) | One `Default` per-visit struct.
25 | MEDIUM | float.rs:206-209 | `land()` asks every floating face with 400 ms each, stale `now` for `ask_again`; blocks when manager silent (CON-B-09) | Break at first unanswered, take `Instant::now()` after.
26 | MEDIUM | modes.rs:262-285 | `draw_modes` makes two blocking queries per regrab; chip loop never shows a chip (CON-B-07) | Remove dead chip path or early-return.
27 | MEDIUM | adopt.rs:271,293, scene.rs:142 | failed `launch` dropped silently, `launched` set anyway, next window forced into a pane (CON-B-11) | Notice on refusal, skip `Waiting`/`launched`.
28 | MEDIUM | camera.rs:328,404, voice.rs | `fly_in` skips fly-out (B-02), unbounded `conn.call` in `pane.focus`/`confirm.answer` (B-08), `retry_plugins` every FRAME timeout (B-04) | `fly_out` first, `call_within`, gate on a `RETRY` instant.
29 | LOW | glass.rs:25, hints.rs:421, hud.rs:117 | three identical rgb helpers | One shared.
30 | LOW | command.rs:755, glass.rs:324 | `chord` defined twice | Use `chora_kit::glass::caps`.
31 | LOW | modes.rs:104, list.rs:26 | copies of kit `Language::of_session` and `Conn::focused_surface` (B-29) | Use kit versions.
32 | LOW | cube.rs:764 | `thread::sleep(50 ms)` on the event loop (A-09) | Join stderr thread / read after exit.
33 | LOW | config.rs:331-357 | `keys` names unvalidated; `Shift+t` loads and never fires (A-05) | Parse with `Binding`.
34 | LOW | hud.rs:870,813 | hard-coded German Ja/Nein (A-21) | Use `Language::of_session()`.
35 | LOW | overlay.rs:1000 | dead `"cube"` arm in `hint_press` (A-26) | Delete.
36 | LOW | adopt.rs:81,189, cube.rs:174,1269, main.rs:29 | stale `too_many_arguments`, ~39 `eprintln!` with no tracing (B-24, A-11, B-22) | Drop allows, settle logging once.
37 | LOW | input.rs:286-367 | ~10 trailing `// injected:` comments against the no-inline-comment rule | One doc comment on `Construct::event`.
38 | LOW | pull.rs:32, cube.rs:38 | `construct-cube-face-` prefix written twice (B-32) | One constant.
Injected-input fixes hold (input.rs:51). Earlier still open: CON-A-01,02,03,05,06,09,11,21,26,30, B-01,02,04,07,08,09,11,14,22,24,29,32. Fixed: A-04, B-10.

## chora-assistant
39 | MEDIUM | chat.rs:896,900 | dead field process restarted at once, error swallowed; a chora-chat that exits at start loops 10x/s silently (A-04) | Backoff, visible state, log.
40 | MEDIUM | chat.rs:1057 | new epoch clears `transcript` but not `self.sent`; echo stays undelivered (A-09) | `self.sent.clear()`.
41 | MEDIUM | voice.rs:652 | `drop_top` clears `card` without `chat.question_ended()` (B-01) | Call it.
42 | MEDIUM | voice.rs:866, chat/host.rs:152 | `conn.call` without timeout on the only loop; up to 10 s freeze (A-10, B-02) | `call_within`.
43 | MEDIUM | voice.rs:835,373 | failed `confirm.answer` only eprintln'd, no visible state (B-03) [sec-adjacent] | "not sent, press again" notice, log.
44 | MEDIUM | chat.rs:854,779,949,806 | results of `agent_start`, `chat.close`, `agent.cancel`, `place_field` dropped; cancel sets idle regardless (A-07, A-08) | Check results, set header on success.
45 | MEDIUM | card_glass.rs:116 | layout shows Yes/No on `buttons || keyed`, `card::hide` removes only on `buttons`; clickable stale nodes (A-02) | Use `shown.buttons` alone.
46 | MEDIUM | chat/host.rs:446,460,650 | start errors only eprintln'd; `open_chat` already set mode Chat (A-05, A-06) | One log helper, one failure path.
47 | MEDIUM | chat/host.rs:335 (9 sites) | same Bridge construction repeated nine times (A-23) | `fn bridge()`.
48 | MEDIUM | card.rs:232 | three card layouts duplicate measure and have drifted (rab-45) (A-11) | Shared `measure`.
49 | MEDIUM | main.rs:620 | `load_assets` duplicates chora-notify main.rs:330; HUD folder tooling only here (check.rs 451 lines) | Move into chora-kit hud.
50 | MEDIUM | voice.rs:40 | 830-line `impl Agent` over a 37-field struct, split over three files (B-06) | `card_host.rs`, sub-structs.
51 | LOW | chat/view.rs:1018 | dead glass code in `panel` (is_glass, glow, GLOW*) (A-21) | Delete.
52 | LOW | chat/view.rs:198 | `keep_newest` drain of `1..=extra` with extra 0 (A-22) | Delete.
53 | LOW | chat.rs:368 | `drawn` only `is_some()`; `Answered` is a bool; full-path `ChatStyle` | Bool, imported name.
54 | LOW | chat/glass.rs:207,215,492,510, chat.rs:561 | repeated bare sizes vs named constants (A-28) | Use constants.
55 | LOW | chat/glass.rs:510 | click nodes at fixed `HEADER + 9`, rows at `column.room()[0]`; clicks miss under a question card (A-03) | Derive from `room()`.
56 | LOW | chat/glass.rs:219 | newest message taller than room leaves transcript empty | Reuse `keep_newest`.
57 | LOW | chat.rs:743,960 | error text parses as empty list; out-of-range pick silent (A-17) | Test `is_error`, report.
58 | LOW | field.rs:272, host.rs:157,374 | three failures swallowed without a log (A-14, A-15) | Shared logger.
59 | LOW | assistant.yaml:20 | no example for `chat`/`output` rules | Add one.
Security: no gap; field spawn has no socket/token (field.rs:286), confined guard host.rs:113, injected presses refused main.rs:500. Fixed: AS-B-05 and injected-press refusal. Earlier still open: A-02..A-17, 21-23, 26-29, B-01..04, 06.

## services/voice
60 | MEDIUM [sec] | coder.rs:284, agents.rs:1342, config.rs:407 | `key_envs` removed only for the `choractl` child; coder and ACP agent spawn with full env, agent-sandbox.sh unsets only CHORA_SOCKET/WAYLAND_DISPLAY/DBUS (79-81); ASR/TTS/LLM keys reach the LLM-driven harness | `without_keys` on both Commands, `--unsetenv` in the sandbox script.
61 | MEDIUM [sec] | config.rs:1141 | `check` only requires a non-empty prompt per `confirm` rule; a `confirm:` list without `run_program` replaces the default and leaves run_program unconfirmed (charter 11b) | Require that `rule(confirm, "run_program", ..)` matches; keep `confirm` a security key.
62 | MEDIUM | agents.rs:1326, coder.rs:284, agents.rs:1495 | two copies of spawn-in-process-group-with-log plus group kill | `harness.rs`.
63 | MEDIUM | chat.rs:1, core.rs, agents.rs | 3061/2768/2182 lines holding several jobs | Split chat.rs into `gate.rs` and `history.rs`.
64 | LOW | main.rs:5 | crate-wide `allow(unwrap_used, expect_used)`; expects at agents.rs:377,998,1041,1352 (VO-A-11) | Remove allow, fix four expects.
65 | LOW | agents.rs:1250 (+7 sites) | char-boundary cut written ~9 times | `text::clip` in chora-kit.
66 | LOW | mc.rs:225, client.rs:895 | own connect/backoff loop; plugins call `conn.call` unbounded | `call_within` plus reconnect in chora-scene `Conn`.
67 | LOW | chat.rs:1124 | `physical` flag path dead (VO-A-08) | Delete or document.
68 | LOW | core.rs:963, main.rs:260 | unreachable arms (VO-A-27, A-28) | Remove.
69 | LOW | main.rs:676 | `projects` duplicates `Agents::projects` with different rules; plain files become ASR hints | Call the agents one.
70 | LOW [sec] | systemd/chora-voice.service:10 | no hardening on a process running model-issued tool calls | NoNewPrivileges, ProtectSystem=strict, PrivateTmp.
71 | LOW [sec] | gufo.sh:9,78 | `gufo-runtime:latest` unpinned, port published on all interfaces (quadlets pin and bind 127.0.0.1) | Pin digest, `127.0.0.1:$PORT:$PORT`.
72 | LOW [sec] | config.rs:893 | Bearer key sent to any URL, cleartext over http/ws to a non-loopback host | Reject in `check`.
73 | LOW | agents.rs:966 | agent ending in Err logged at info | Log at error.
74 | LOW | chat.rs:1081 (+core.rs:512, agents.rs:601, main.rs:144) | inline comments against project rule | Move to doc comments.
Charter 11 (a)-(c) hold as above. Fixed: VO-A-01,02,13,18, VO-B-02. Earlier still open: VO-A-08,11,23,27,28,29,37, VO-B-10..13.
