# review core r1 (rc 6 core and security, rc 9a logging)

Base 498d9de. Review by reading; a finding marked (s) is suspected, not traced to the end.
Status: FINAL. SEC = security. `FIXED hash` = fixed on claude/review-core with a red test
before it; `OPEN` = not fixed, with the reason.

## HIGH

- C01 HIGH SEC FIXED c39f4e12 crates/chora/src/ipc.rs:127, inject.rs:467: a NUL in a key name panics in xkbcommon (`CString::new(..).unwrap()`); `input.ungrab {"binding":"Super+a\u0000"}` from any connection, without hello, ends the session.
- C02 HIGH SEC FIXED fa3909bf crates/chora/src/inject.rs:367, input.rs:123: injected keys use the keyboard source of the person, so an injected release of Ctrl or Alt lifts the modifier the person holds; a client with `input.inject` suppresses reset, quit, revoke and the VT switch.
- C03 HIGH SEC FIXED ad592ee1 crates/chora/src/text.rs:307 (from capture.rs:191): a node capture of a huge rect with a text child gives scale 0, cosmic-text asserts on line height 0; a client with `screen.capture` ends the session.
- V01 HIGH SEC FIXED 7f4a1a90 services/voice/src/agents.rs:1322, coder.rs:280: the agent and coder children inherit the environment with the API keys of `*_api_key_env`. Fixed for the agent, the coder and the MCP child; the capture and playback commands of audio.rs still inherit them (about ten call sites).

## MEDIUM, security

- C04 MEDIUM SEC FIXED 3e7ac56b crates/chora/src/ipc/clients.rs:318: the `screen:*` rule of a confined connection misses `attach`; a client socket puts its window in the overlay root.
- C05 MEDIUM SEC OPEN (decision) crates/chora/src/output.rs:267: `output.configure` and `output.reserve` have no gate; a client socket without capabilities rotates or scales the outputs. Not fixed: the MCP tool `configure_output` and its rule in the voice settings use `output.configure` on a confined connection without a grant, and `reserve_space` uses `output.reserve`; a gate with `methods.privileged` (as `input.configure` has) changes that product behaviour and needs the owner.
- C06 MEDIUM SEC FIXED 2a0365c4 crates/chora/src/ipc.rs:1401: grabs have no cap per connection; a client socket holds any number of Ctrl, Alt or Super bindings.
- C08 MEDIUM SEC FIXED 478696d3 crates/chora/src/methods.rs:200, ipc.rs:593: a caller floods an exported method with 1 MiB params; the output of the callee overflows and the core closes the callee (the window manager).
- C09 MEDIUM SEC FIXED 478696d3 crates/chora/src/methods.rs:134: `description` and `input_schema` of an export have no size limit; one client socket holds about 1 GiB in the core.
- C10 MEDIUM SEC FIXED 521d5f15 crates/chora/src/input.rs:414: quit, reset and revoke-all are not gated on the lock; Ctrl+Alt+Backspace at the lock screen ends the session.
- C11 MEDIUM SEC FIXED 58b6ac4a crates/chora/src/backend/headless.rs:221: `output.add` and `output.remove` of the headless backend have no gate for confined connections (s).
- C12 MEDIUM SEC FIXED b3c6a3a1 crates/chora/src/inject.rs:129: the injection mark sits at pointer + 8 px without a clamp; at the right or bottom edge it is outside the output.
- C13 MEDIUM SEC OPEN crates/chora/src/inject.rs:129: the mark is a child of the overlay root; a later overlay node of a client covers it (s). Not fixed: not traced to the end; the fix is to draw the mark after the overlay roots in the renderer.
- C14 MEDIUM SEC OPEN (decision) crates/chora/src/inject.rs:349: injected input counts as activity for idle-notify; a client with `input.inject` keeps the session from locking. Not fixed: the call is explicit, so it reads as intended (an agent that works keeps the screen on); the owner decides.
- C15 MEDIUM SEC FIXED 2626c63b crates/chora/src/text.rs:373: the warning "glyphs laid out but drawn by no font" logs the whole text of a client's node.
- K01 MEDIUM SEC FIXED c356054d crates/choractl/src/view.rs:242, mcp.rs:691: `save_view` rewrites a view file from a view that is not loaded (scene lost) and treats a file that does not parse as no file (`plugins`, `input` lost); not atomic. The write stays `std::fs::write` (choractl has no chora-kit dependency); a save with `--file PATH` of an unloaded view is not covered.
- K02 MEDIUM SEC FIXED 5f8df4d5 crates/choractl/src/view.rs:99, main.rs:1006: choractl takes an empty or relative `XDG_CONFIG_HOME`; views, plugin manifests (`exec` through `sh -c`) and clients.yaml come from the working directory.
- K03 MEDIUM SEC FIXED 5ef14cf2 crates/chora-kit/src/settings.rs:106: parse errors of a settings file quote the offending value into the journal and into RPC errors (a key at the wrong place leaks) (s).
- V02 MEDIUM SEC OPEN (decision) services/voice/src/core.rs:998, chat.rs:689, speech.rs:180, coder.rs:229: every utterance, reply sentence and coder task goes to the journal at info; docs/voice.md section 11 describes stage logs, a decision of the owner (the live gate reads these lines).
- V03 MEDIUM SEC FIXED ebdcfba4 services/voice/src/chat.rs:996, 1431: tool arguments are logged whole; `inject_input` text (a dictated password) and 512 KiB assets land in one info line.
- V05 MEDIUM SEC FIXED ea7b836c services/voice/src/core.rs:1105: a key press answers the current question; an agent replaces its question and the key meant for the first approves the second.
- V06 MEDIUM SEC FIXED 6931b7c2 services/voice/src/core.rs:1048: yes and no are matched before the echo filter; the tail of the spoken question can count as a yes (s).
- V07 MEDIUM SEC FIXED 039f9735 services/voice/src/core.rs:297, 885: one `physical` flag serves the chat question and the sub-agent question; a chat question clears it for a waiting physical one (s for the trigger).
- V08 MEDIUM SEC FIXED 039f9735 services/voice/src/chat.rs:1086, main.rs:541: `ask` stores a constant false into `tools.physical` and clears the flag of a showing key question; dead parameter.
- V09 MEDIUM SEC FIXED 67c65ed4 services/voice/src/llm.rs:179, health.rs:66, 113: error texts carry the full server URL (query, userinfo) into the log and into `voice.health`, which any client reads.
- V10 MEDIUM SEC FIXED 90926f26 services/voice/src/record.rs:23, 62: recordings of the room are created with default modes (0644 in 0755).

## MEDIUM, other (known issues, not fixed in this round)

- C20 MEDIUM crates/chora/src/ipc.rs:603: pending output is written only in `after_events`, no source waits for writable; a large answer on an idle session moves one socket buffer per heartbeat.
- C21 MEDIUM crates/chora/src/ipc.rs:803: the read limit per wakeup bounds bytes, not work; 64 KiB of short lines is thousands of `scene.query` or 2 GB of memmove in one turn.
- C22 MEDIUM crates/chora/src/ipc/clients.rs:94: the accept loop of a client socket ends on any error without a log or pause; EMFILE spins the main loop.
- C23 MEDIUM crates/chora/src/render.rs:495, mesh.rs:296: `size.w * size.h` in i32 for a backdrop rect of a client; overflow panics with overflow checks, miscounts the budget in release.
- C24 MEDIUM crates/chora/src/backend/tty.rs:978, devices.rs:211: session pause and device removal release no keys; a lost release leaves Ctrl+Alt down for the safety chords (s).
- C25 MEDIUM crates/chora/src/input.rs:240: a forwarded press does not clear a stale `held.keys` entry; its release is swallowed and the client repeats the key.
- C26 MEDIUM crates/chora/src/input.rs:473, security.rs:120: a confine region is not enforced, and any surface under the pointer may lock it without the keyboard focus; no mouse way out.
- C27 MEDIUM crates/chora/src/backend/tty.rs:590: a connector that fails once is not retried; `Connected { crtc: None }` is not logged.
- C28 MEDIUM crates/chora/src/gl/backdrop.rs:192: `while gl.GetError() != NO_ERROR {}` has no bound; a lost context spins the core (s).
- C29 MEDIUM crates/chora/src/inject.rs:361: an injected button press with `hold` is not released when the connection closes, the grant goes or the reset runs.
- C30 MEDIUM crates/chora/src/safety.rs:393: the reset leaves layer surfaces and a keyboard grab; an exclusive overlay keeps the screen and the keyboard after Ctrl+Alt+Escape.
- C31 MEDIUM crates/chora/src/lib.rs:127: a startup command that fails to spawn is logged and dropped; the core stays with a black screen, against architecture.md.
- C32 MEDIUM crates/chora/src/text.rs:78, lib.rs:121: the font thread starts before SIGUSR1 is blocked; the signal of the escape hatch in the first second kills the core (s).
- C33 MEDIUM crates/chora/src/images.rs:185: decode jobs in flight are not bounded; over 1024 distinct paths in one transaction reach EMFILE (s).
- C34 MEDIUM crates/chora/src/space.rs:317: `project` divides by `w` without the check that `point` has; NaN polygons reach the splitter of plane-split, which can panic (s).
- C35 MEDIUM crates/chora/src/images.rs:567: `x1 - x0` in i32 after saturating casts; overflow panics with overflow checks.
- K10 MEDIUM crates/chora-kit/src/settings.rs:236, runtime.rs:444: `reload` clears `broken` before the plugin applies; after a failed apply the next `settings.set` overwrites the hand-edited file.
- K11 MEDIUM crates/choractl/src/mcp.rs:195: `subscribe_events` during a running `wait_events` loses the subscription (the old socket holds the grabs).
- K12 MEDIUM crates/choractl/src/mcp.rs:703: `run_program` bypasses `Core::call`; after the core closed the connection it fails until another tool resets the client.
- V20 MEDIUM services/voice/src/coder.rs:343: `GROUPS` keeps the pid of an ended task; a reused pid gets TERM and KILL at exit.
- V21 MEDIUM services/voice/src/llm.rs:186: the body of a non-2xx answer is read without a timeout; a stalled server hangs the turn.
- V22 MEDIUM services/voice/src/tools.rs:227: `refresh` (spawn, handshake, tools/list) has no timeout (s).
- V23 MEDIUM services/voice/src/chat.rs:722: `self.answer` is an index into `history` that goes stale when the turn goes on; `cancelled` overwrites another message or indexes out of bounds (s).
- V24 MEDIUM services/voice/src/record.rs:30: sample counts are u32; `samples * 2` overflows after about 24.8 h.

## LOW

- C07 LOW SEC crates/chora/src/ipc.rs:320: only `surface.*` events are filtered for client sockets; `effect.failed`, `focus.changed`, `layers.dropped`, `client.connected` of other clients reach them. Rated LOW after a second look: api.md promises the filter for `surface.*` only, and the content of the compile log is not shown to leak a file.
- C40 LOW SEC crates/chora/src/ipc.rs:480: "same program" is the same `/proc/PID/exe`; every script of one interpreter may take the live name of another (api.md promises otherwise for scripts).
- C41 LOW crates/chora/src/ipc/clients.rs:201: a recreate with other capabilities sets the grants before `open`; a failed bind leaves grants without a socket.
- C42 LOW crates/chora/src/ipc/clients.rs:103: one failed `insert_source` leaves the other listener and the socket files; not logged.
- C43 LOW SEC crates/chora/src/ipc.rs:1021: client `a` may emit `a.b.c`, the name of event `c` of client `a.b`.
- C44 LOW SEC crates/chora/src/ipc.rs:246: `private_dir` compares the owner with the owner of the parent, not with the effective uid.
- C45 LOW crates/chora/src/ipc.rs:521: the connection cap is per pid; a forking client fills all 128 slots, the admin share too.
- C46 LOW crates/chora/src/ipc.rs:1458: an ended connection whose peer never reads keeps its slot and name.
- C47 LOW crates/chora/src/ipc.rs:869: seven methods need no hello, the others do.
- C48 LOW crates/chora/src/uploads.rs:172, 113: a failed write leaves `.NAME.part` outside the quota; `remove` forgets the file before the unlink.
- C49 LOW crates/chora/src/uploads.rs:30: the upload quota and `MAX_NODES` are global; one client takes the room of all.
- C50 LOW crates/chora/src/ipc/clients.rs:356: `pressed_by` spends the press in the check, before the transaction applies.
- C51 LOW crates/chora/src/ipc.rs:193: a failed admin bind leaves the main socket file and listener.
- C52 LOW crates/chora/src/backend/tty.rs:961: `schedule` drops a failed timer with `.ok()`; the output never renders again.
- C53 LOW crates/chora/src/gl.rs:174: the unsafe `DrawElements` trusts the indices without a check; `triangles()` maps an oversized index to `u16::MAX`.
- C54 LOW crates/chora/src/gl.rs:334: `EffectPrograms` is never pruned; `fs::metadata` per pass per frame.
- C55 LOW crates/chora/src/input.rs:72: touch, tablet, gesture and switch events are dropped; the eight gesture forwarders of `PlaneGrab` are dead.
- C56 LOW crates/chora/src/offscreen.rs:446: a pane of size 0 gives NaN positions into pointer motion and IPC events.
- C57 LOW crates/chora/src/lib.rs:54: the test switch `--test-injected-press-is-real` ships in the release binary.
- C58 LOW crates/chora/src/backend/tty.rs:516: an error after `gpus.add_node` leaves the node registered; `enable_output` changes the mode before `start` can fail.
- C59 LOW crates/chora/src/backend/tty.rs:718: `retire_global` is a shared helper in tty.rs, used by headless.rs and offscreen.rs.
- C60 LOW crates/chora/src/images.rs:324, transaction.rs:373: `open` and `stat` of a client path run on the event loop; a FUSE or NFS path stalls it (s).
- C61 LOW SEC crates/chora/src/protocols.rs:78: any live surface inhibits idle, mapped or not; inhibitors are not deduplicated.
- C62 LOW SEC crates/chora/src/state.rs:351: input-method-v2 is not blocked while locked; `commit_string` may reach a lock surface with text-input-v3 (s).
- C63 LOW crates/chora/src/images.rs:134: the decode worker has no `catch_unwind`; a decoder panic ends image loading for the session (s).
- C64 LOW crates/chora/src/process.rs:31: the startup child starts before the backend; a failed backend orphans it, a quit never signals it.
- C65 LOW crates/chora/src/inject.rs:88: each request with a key, shortcut or text compiles the keymap on the main thread.
- C66 LOW crates/chora/src/capture.rs:101: render, readback, copy and PNG encoding of up to 16 M pixels run on the event loop; the 5 MiB limit is checked after encoding.
- C67 LOW crates/chora/src/record.rs:137: the method list is checked twice.
- C68 LOW docs crates/chora/src/ipc.rs:572, 655, 729, 743: doc comments on the wrong items (`Conn::wants`, `core_namespace`, test `ipc_accept`); `ipc_accept_as` names a parameter that is gone.
- C69 LOW docs crates/chora/src/input.rs:55, 108; mesh.rs:1042; render.rs:1120; backend/winit.rs:330; shm.rs:1; time.rs:63: garbled or stale doc comments (`KeyAction::Grab`, "at most 8192" with `MAX_SIDE` 4096, "effects milestone", `check_effect` doc on `gles()`, "second place with unsafe", "`None` when ...").
- C70 LOW docs docs/architecture.md:337: the limits table omits exports, pending calls, emit, assets and client sockets; :475 the mark is not shown for `dry_run` or a pointer on no output; :73 names six host variables only.
- C71 LOW crates/chora/src/mesh.rs:164: `told` says "in this frame" but is never reset.
- K20 LOW SEC crates/choractl/src/mcp.rs:411: plugin tool names, descriptions and schemas reach the agent verbatim; dotted names are ambiguous, the first wins (s).
- K21 LOW crates/choractl/src/events.rs:140: the retry depends on the text "grabbed already" of the core.
- K22 LOW crates/chora-kit/src/effects.rs:283, lamp.rs:38: the lamp starts on `Instant::now()`, the tick gets the stepped `clock::now()`.
- K23 LOW crates/chora-kit/src/runtime.rs:360: `confined_call` and `caller` are not reset after a call.
- K24 LOW crates/chora-kit/src/runtime.rs:305: `ungrab_all` forgets grabs that the core still holds after an error midway (s).
- K25 LOW crates/chora-kit/src/volume.rs:233: the job queue is unbounded; key repeat with a hung notification server builds a backlog (s).
- K26 LOW crates/chora-kit/src/conn.rs:176: `node()` adds a missing `effects` that serde already defaults; dead code.
- K27 LOW crates/chora-kit/src/ask.rs:117, notify.rs:51, words.rs:206: `Place`, `Card::place`, `Notifier::at` and `ALL` are pub and used only by their own tests.
- K28 LOW crates/chora-kit/src/glass.rs:302: `plain(&self, _strong)` ignores its argument; `if !BACKDROP` is a constant branch.
- K29 LOW crates/chora-kit/src/icons.rs:30: the icon cache has no bound; names come from D-Bus (s).
- K30 LOW crates/chora-kit/src/desktop_entry.rs:63: the walk follows directory links without a depth check (s).
- K31 LOW crates/chora-kit/src/chat.rs:351: an entry is wrapped whole, then cut to 40 lines; no cap on the text.
- K32 LOW crates/choractl/src/main.rs:692: the doc of `start_child` sits on `RUN_TOKEN`; :3 the crate-wide `allow(unwrap_used, expect_used)` is stale.
- V04 LOW SEC services/voice/src/agents.rs:851: `is_physical` trusts `toolCall.kind` of the agent; the harness is the party that asks, so a hostile harness would not ask at all, but nothing checks the kind against `rawInput`.
- V30 LOW SEC services/voice/src/chat.rs:1341, agents.rs:1315: `voice-history.json` and the agent log have default modes; the agent log grows without limit (modes fixed with V10).
- V31 LOW SEC services/voice/src/llm.rs:82: redirects are followed; custom `llm_headers` go to another host (s).
- V32 LOW SEC services/voice/src/config.rs:845: no check of scheme or host; a bearer key goes over `ws://`; the mode of the key file is not checked.
- V33 LOW SEC services/voice/src/config.rs:678: `earcon_volume`, `voice`, `tts_model`, `agent_permission_s` are no security keys, `earcons` is.
- V34 LOW SEC services/voice/src/llm.rs:264: the `CHORA_VOICE_LLM_DUMP` file is opened without `O_EXCL` or `O_NOFOLLOW`; an existing 0644 file keeps its mode.
- V35 LOW services/voice/src/agents.rs:831: a no falls back to `reject_always`; section 18 says cancelled.
- V36 LOW services/voice/src/coder.rs:288, 444: a cancel kills the leader at once (docs: TERM, KILL after 5 s); the `git clone` child has no `kill_on_drop`.
- V37 LOW services/voice/src/agents.rs:982: the actor awaits `stdin.write_all`; an agent that reads no stdin survives `agent_stop` (s).
- V38 LOW services/voice/src/core.rs:511: question timeouts run only in `handle`, driven by mic frames.
- V39 LOW services/voice/src/chat.rs:1186: `removes_own` parses the cut scene text; over 60 nodes the rule never applies (fails closed).
- V40 LOW services/voice/src/chat.rs:146: `listed` and `spoken_program` drop argv items that are no strings; `detail` joins argv with spaces.
- V41 LOW services/voice/src/config.rs:379: the default `confirm` has no rule for `save_view`, `put_asset`, `reserve_space`.
- V42 LOW services/voice/src/live.rs:113: `chora-voice ask` sets no `own`; the own-client refusal and the redaction are off.
- V43 LOW services/voice/src/config.rs:997: the timeouts have no lower bound; 0 fails every request.
- V44 LOW SEC services/voice/src/coder.rs:325: `git clone` of a project runs outside the sandbox on a repository the agent can write (s).

## Logging audit (rc 9a)

No secret is logged by the core (no tracing call with keysyms, key codes, typed text or
tokens; `node.key` goes to the owner only). Secrets in logs: C15, K03, V02, V03, V09 above.

- L01 MEDIUM crates/chora/src/backend/tty.rs:749, offscreen.rs:90, winit.rs:196, render.rs:817, backdrop.rs:436: a lasting render error warns once per output per frame; no rate limit.
- L02 LOW crates/chora/src/time.rs:71, ipc.rs:765: a failed timer is logged twice.
- L03 LOW crates/chora/src/ipc.rs:611, 630: read and write errors close a connection without a log; hello, a refused hello and a close are not logged at any level.
- L04 LOW crates/chora/src/ipc.rs:718: `disable(..).ok()` and `enable(..).ok()`; a failed enable leaves the listener off without a log.
- L05 LOW crates/chora/src/ipc.rs:758: a refused connection warns without pid or socket; a loop floods the log.
- L06 LOW crates/chora/src/ipc/clients.rs:61, 250; grants.rs:97: `clients.create` and the close of a client socket are not logged; `grants.set` logs before its validation.
- L07 LOW crates/chora/src/uploads.rs:51, 141; ipc.rs:266; clients.rs:434: failed cleanups are silent (`let _ =`).
- L08 LOW crates/chora/src/screencopy.rs:232, image_capture.rs:143: each failed frame warns, "the session is locked" too; up to 4 lines per refresh during a lock.
- L09 LOW crates/chora/src/images.rs:551, 276; text.rs:597: "an image failed to upload" per cell per frame without node or path; `reload` and the text upload drop errors.
- L10 LOW crates/chora/src/cursor.rs:75: a missing cursor theme gives an invisible cursor without a log.
- L11 LOW crates/chora/src/devices.rs:66: settings that a device rejected log at debug while `input.configure` answers success.
- L12 LOW crates/chora/src/safety.rs:428, 376: "emergency reset" does not name its trigger; `revoke_all` logs nothing without grants.
- L13 LOW crates/chora/src/process.rs:50, lib.rs:147: the exit of the startup command warns for status 0 too; a failed start is reported three times.
- L14 LOW crates/chora/src/render.rs:237, offscreen.rs:407, backend/mod.rs:155: errors dropped with `.ok()` or `let _ =`, never logged.
- L15 LOW crates/chora-kit/src/runtime.rs:521: a core that is gone logs "I come back" about 60 times in 30 s.
- L16 LOW crates/chora-kit/src/lib.rs:81, conn.rs:77, volume.rs:160: `launch`, `activation_token` and spawn errors are dropped without a log.
- L17 LOW crates/chora-kit/src/settings.rs:106, services/voice/src/main.rs:126: broken settings are logged twice.
- L18 LOW crates/chora-kit/src/runtime.rs:375, 460, conn.rs:160: three prefixes; the answer error names no method.
- L19 LOW crates/choractl/src/mcp.rs:97: an Io error of a call is replaced by `Closed`; the reason is lost.
- L20 LOW services/voice/src/tools.rs:201, agents.rs:628: command lines of the settings are logged whole.
- L21 LOW services/voice/src/main.rs:440, 480, client.rs:535: HUD and recording failures log at debug or not at all.

## Result and handover

Head eff84702 of claude/review-core (37 commits on 498d9de, pushed). Counts: 4 HIGH, 22
MEDIUM security, 25 MEDIUM other (L01 included), 82 LOW.

- Fixed, each with a red test before it: C01 C02 C03 V01 (all HIGH), C04 C06 C08 C09 C10
  C11 C12 C15 K01 K02 K03 V03 V05 V06 V07 V08 V09 V10. C15 has `No test:` (one log field).
- Open security MEDIUM: C05 and C14 and V02 need a decision of the owner, C13 is suspected
  and not traced. The other MEDIUM and the LOW are reported only, as the brief says.
- Left inside fixed ids: V01 audio.rs children still inherit the key variables; K01 the
  write is not atomic and `--file PATH` is not covered; V10 old files keep their mode.
- Behaviour that changed: quit, reset and revoke do nothing while locked (VT switch and
  SIGUSR1 stay); a confirm key counts only 1 s after its question showed; 256 grabs per
  connection; an export has at most 64 KiB of description and schema; a `methods.call`
  fails with code 6 when the output of the exporter is full; headless `output.add` and
  `output.remove` refuse a confined connection. Docs changed with each.
- Merge gate on eff84702: every step passes except `cube`, which fails in about half of
  its runs with "the construct does not answer while the window manager is deaf" (2 of 2
  full runs, 1 of 3 alone). The base 498d9de fails the same step in 4 of 6 runs, so it is
  no effect of this branch; it belongs to the construct lane. `pull` to `pixels` passed
  with `--only`. No single full run printed that every step passed.
- Other flakes seen once: `words::tests::a_stack_overflow_leaves_its_words` at the base
  (4 reruns pass) and chora-xp `the_speaker_has_ink_and_the_no_sign_is_the_palette_red`
  (theme cache directory missing, 3 reruns pass).
- Not run: the session scripts of the voice service (`tests/voice-*.sh`); a script that
  presses the confirm key within 1 s of its question now needs a wait.
