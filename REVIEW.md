# Review round 4 for v0.9.0-rc.3 (d8d59c5)

Range: `git diff 618f967 d8d59c5` (40 files, +3009/−190). This review only read code; nothing on `dev` was changed. The old findings are those of REVIEW.md on `review/rc3-round3`, numbered as there. Line numbers are at d8d59c5.

**Tests run.** The pinned toolchain 1.99 installed itself; `cargo test` ran for chora-voice, chora-scene, chora-construct, chora-mc and chora-xp: all pass. The core crate `chora` does not build in the review sandbox (no `libudev`), so state.rs, ipc.rs and safety.rs were reviewed by reading, and the gate and battle scripts were not run (shellcheck is not installed; `python3 -m py_compile` of w16-fakes.py passes). For finding 1 I ran a temporary test (reverted) that feeds a default-sized settings answer with three header values through the real `trimmed` and `redacted`.

Severity scale: **breaks** / **endangers** / **simplifies** / **nit**.

---

## 1. Status of the old findings

| # | Status | Evidence |
|---|---|---|
| 1 `settings.set`/`reload` answer the headers | **partly** | `redacted` exists (tools.rs:352-357, :435-440) and `journal` hides a JSON result (chat.rs:1397-1405). But `trimmed` runs first and cuts the text at 6000 characters, and the answer is longer (the default settings alone are 6376 characters). A cut text is no JSON, so `without_secrets` returns it unchanged. Leak: new finding 1. The unit test uses a short answer. |
| 2 expired reservation leaves an ownerless window | **fixed** | state.rs:555-576 `expire_reservations`, :585-620 `hand_over` (giver, then the manager of a new window, then any claimant, only connected ones, never `except`), :625-634 `place_free`, scene.rs:385-397. The window of a dead manager goes the same way: `execute_gone` (state.rs:638-647), ipc.rs:1089 and :1377. Tests `a_reservation_that_nobody_attaches_goes_back_to_the_giver_then_shows`, `the_windows_of_a_dead_manager_go_to_a_live_one_until_none_is_left`. I found no path that ends with an ownerless, hidden surface; see "Checked and clean". |
| 3 `allow_programs` ignores the arguments | **fixed** | chat.rs:146-156 matches the whole argv, config.rs:1016-1031 allows an entry with single-space arguments, docs/voice.md:302-310. Test `allow_programs_matches_the_whole_argv`. |
| 4 `settings.reload` is a spoken question | **fixed** | chat.rs:1026-1044 `reload_is_security` (an unreadable file counts as different), :1058-1067 makes the question physical, :1082 skips `allow_methods`. Test `a_reload_with_a_new_security_key_asks_for_a_key`. The file is read at the question and again at the apply; both are inside one turn, so I count the gap as closed. |
| 5 pane "ahead" ignores whether it shows | **fixed** | construct main.rs:3087-3099 `ahead` takes only panes with `sight == "shown"`; `close_ahead` (:1623), `FlyIn` (:1721) and `focus` (:2133) all go through it; `sight` now sees the corners like the frame (space.rs:224-239). Test `no_pane_ahead_that_is_out_of_sight`. |
| 6 "physical" = holder of `indicator.claim` | **fixed** | docs/voice.md:385-390 says it. |
| 7 expiry tells every client | **fixed** | `hand_over` sends `surface.detached` only to the chosen manager (state.rs:611-619). |
| 8 "same shape" blocks other values | **fixed** | chat.rs:322-326 compares the whole args; the test runs a third call with other values. |
| 9 tiling `at` absolute → relative | **not fixed** | plugins/chora-tiling is not in the range. Position only. |
| 10 mc claims its whole output | **fixed** | mc claims the work area (mc main.rs:1224-1230) and only with `--manage-windows` (:1325). A tie with another manager on the same work area is still broken by node id order. |
| 11 `Colors` shim in chora-mc | **not fixed** | plugins/chora-mc/src/config.rs:72 `from = "Colors"`. |
| 12 an older utterance keeps a question open | **fixed** | core.rs:512-521 (`from`, `until`); test `an_older_utterance_does_not_keep_a_question_open`. |
| 13 unbounded confirm detail | **partly** | construct caps it (hud.rs:275-292, :1281). mc (agent.rs:502) and xp (wm/voice.rs:227) still show the whole detail, and the cap of construct hides the tail of a command: new finding 2. |

---

## 2. New findings, most severe first

1. **tools.rs:352-357 (`redacted(trimmed(...))`): the answer of the agent's own `settings.set`/`reload` leaks `llm_headers` and `asr_headers`.** Severity: **endangers**.
   - Order of calls: `content` → `trimmed` (6000 characters, then `"\n… cut; ask for less"`) → `redacted`. The settings answer is longer than 6000 characters (kit settings.rs:213-224 answers `self.get()`, the whole settings, as an alphabetical map; the defaults alone are 6376 characters). The cut text does not parse, so `without_secrets` (tools.rs:435-440) keeps it as it is.
   - Repro (temporary test, reverted): the default settings with `llm_headers`, `asr_headers` and `tts_headers` each `{"Authorization": "Bearer sk-SECRETn"}`, through `redacted(trimmed("voice.settings.set", text), true)`: `sk-SECRET1` and `sk-SECRET2` stay in the answer, `sk-SECRET3` only because it lies after the cut. Redacting first gives no leak. In a session: header in the settings file, "set barge in to 300", yes; the tool result holds the keys and goes to the LLM history and the LLM server. This is round-3 finding 1 again.
   - Fix: call `redacted` before `trimmed` (or hide inside `content`). Add a test with a default-sized answer.

2. **construct hud.rs:275-292 and :1281 (`shown_detail`): the question shows only the first 400 lines or 64 KiB of the command.** Severity: **endangers**.
   - `detail` is the whole command (chat.rs:168-190, no cap) and the user answers by key or voice. An injected LLM can start with 400 lines or 64 KiB of harmless text and put the real command after the cut; the user sees the "…" and the prompt, and may agree. mc and xp show everything, so only the flavor with construct is affected.
   - Fix: refuse the call without a question when the detail passes the cap (the agent knows the size at chat.rs:1058-1085), or show the tail instead of the head, or say "N more lines" with the key that opens the rest.

3. **chat.rs:900-906: the note of dropped turns is a second `system` message after the first.** Severity: **endangers** (depends on the LLM server's chat template; I could not test one).
   - Chat templates that accept a system message only at index 0 (several llama.cpp/Jinja templates answer 400/500 with "System message must be at the beginning") reject the request. That happens at the first dropped turn of a long session (`history_tokens` 24000), and the agent then says `llm_down` for every following request until the process restarts, since `dropped` is never cleared.
   - Fix: append the note to the text of the first system message, or send it as the first `user` message. Also, the note holds the assistant's earlier answers (`you:` lines) and the user texts as `system`, which raises their rank; as part of the system text this stays as it is, but say so in the docs.

4. **tests/battle/lib-wm.sh:136 and tests/gate-construct-windows.sh:129: `shows` is not defined.** Severity: **breaks** (the test, not the product).
   - The gate lost its `shows = ...` line in this range (diff of gate-construct-windows.sh at :120), and lib-wm.sh never had one. An ownerless surface hits `if not shows:` and the Python raises `NameError`. So the new state that the core produces on purpose (an ownerless surface shown at the root) is always reported as a lost window by `no_lost`, in W13-W15 and in the gate, instead of being checked for being shown.
   - Fix: `shows = n["visible"] and n["opacity"] > 0 and n["rects"]` in the loop of both scripts.

5. **mc agent.rs:502 and xp wm/voice.rs:227: the confirm detail is not capped** (rest of old 13). Severity: **nit**. A 1 MB detail makes one text node per line in mc and one balloon text in xp. The cap needs the opposite rule from finding 2: show a bounded part and refuse above it.

6. **speech.rs:341-344 and :240: a `Cancel` of a prompt that never reached the player is kept in `cancelled`, and the next sentence with the same text is skipped.** Severity: **nit**.
   - With a muted speaker the question is never queued (core.rs:787, :855), the answer pushes `Cancel(prompt)`, and the same question after the unmute (for example "Shall I run foot?" again) is not read aloud; the HUD still shows it. Fix: clear `cancelled` at the start of each turn, or send `Cancel` only when the speaker is not muted.

7. **mc main.rs:849-853 and :1235-1245 (`pane.focus`): not privileged, and does not check that the scene shows.** Severity: **nit**.
   - Any client with `methods.call` can move the keyboard to the app window of any scene of mc, also one that is hidden: a keyboard that types into a window nobody sees. The construct passes the name of the pane's scene, while the node that the pane shows is the root `mc`, whose tag follows the current scene (mc main.rs:2036); after a `scene.show` the pane shows another scene than the one it focuses. Fix: refuse a scene that is not the current one, or mark the method `privileged`.

8. **construct main.rs:2578-2583 (`detached`): a pane that exists makes the event of a freed window no news.** Severity: **nit** (by reading; needs a flavor and an overlay or a manager that frees a window that the construct only mirrors).
   - If the core hands such a window to the construct (`hand_over`, chosen) the construct ignores it, the 5 s pass, and the core shows the window at the top left of the output, over the space, while the pane keeps mirroring it. Nothing is lost, but the window is drawn twice and unmanaged. Fix: ignore the event only for a pane whose window the construct owns.

9. **chat.rs:898-906 and :1226-1237: the note of dropped turns is not in `fixed`.** Severity: **nit**. Up to 3000 characters (about 1000 tokens, the default `reply_reserve_tokens`) are sent beyond the estimate, so the budget is exceeded by that much just after a trim. The cut at a character (not a line) is noted in the code.

10. **mc main.rs:979-981: the doc comment of `page_id` ("The id of the page of a scene…") now sits above `pane_tag`.** Severity: **nit**.

**Checked and clean (the delta):**
- *Core rule "every free window goes to a definite manager or shows at the root".* New toplevel (handlers.rs:208) → `reserved` for every toplevel, with or without `to`; the deadline hands it on; `chosen` ends the chain at the second deadline with `place_free`; `free()` (no owner, no parent) guards every step, so an attached or closed surface is skipped; `execute` drops the entry of an owned surface (scene.rs:676 sets the owner on attach). A manager must be connected (`has_client`), hold a claim, and a client socket only manages its own windows. The watchdog victim stays in `conns` until its answer is out, but its claim nodes are gone, so `holds_claim` keeps it out of `hand_over`. A slow only manager (over 5 s) gets its window shown by the core and can still attach it (`to` is cleared, scene.rs:394).
- *Watchdog message* (safety.rs:248-254, ipc.rs:1318-1331): one error, then the close without grace (`keep_nodes` and `resume` off); the 150 ms transaction deadline cannot hold the connection open.
- *Trimmed history:* `fit` still keeps the last turn and cuts at user messages; `drop_old_images` keeps the newest picture; `NUDGE` and `STOPPED` stay out of the note.
- *`call_method` of the own `settings.set`:* `unwrap_patch` runs before the question (chat.rs:971), so `own_settings_set` sees the real keys and the security rule applies; a wrapper with more keys is not unwrapped and its security keys still count.
- *Physical confirmations:* reload (finding 4 of round 3) is fixed; `listen` is no security key by the owner's documented decision (docs/voice.md table).
- *Nested title* (winit.rs:323-331) and its docs; *hidden adoption* (construct main.rs:1434-1456, :1497-1516): an owned window without room is attached with opacity 0, kept in `hidden`, and its attach is not repeated; the destroyed event clears it; `leave` frees the group, which the core hands on.
- *`pane_ahead`/`sight`, `pane.focus` of the plugin pane* (snap, fly-out, removal of the pane while flown): the keyboard returns with `fly_out`.
- *mc claiming only with `--manage-windows`:* every session file, gate and battle script that needs mc as a manager passes the flag; without it mc attaches its own windows and claims nothing.
- *xp theme lookup* (xp main.rs:117-132): explicit path first, then the installed one, then the source tree.
