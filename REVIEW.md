# Review round 2 for v0.9.0-rc.3 (99a6b9c)

Range: `v0.9.0-rc.2..99a6b9c` (94 files, +13476/−1048). This review only read code; nothing on `dev` was changed.

**Tests run.** The pinned toolchain 1.99 could not be installed in the review sandbox (rustup: "cargo … not applicable"). With `cargo +stable` (1.97), the unit tests of chora-voice, chora-kit, choractl, chora-tiling, chora-mc, chora-xp and chora-construct all pass: 33 + 153 + 28 + 15 + 31 + 50 + 116 tests, 0 failures. The core crate and the gate scripts were reviewed by reading only.

Severity scale: **breaks** / **endangers** / **simplifies** / **nit**. All line numbers are at 99a6b9c.

---

## 1. Bugs in services/voice

1. **mc.rs:161, :168: the `voice.confirm` id is a number, but chora-mc expects a string, so the question shows on no screen.** Severity: **breaks**.
   - Cause: `Report::Confirm { id: u64 }` (core.rs:194) is sent as `json!({"id": id, …})`. `plugins/chora-mc/src/agent.rs:84` declares `id: String` and parses it strictly. The mc test only ever sends `"c1"`.
   - Repro: run chora-mc (it claims the indicator), then say "start foot". mc rejects the event with "invalid type: integer, expected a string". Because mc claimed the indicator, main.rs:525 passes `hud: !self.claimed == false`, so the agent's own question node is suppressed too. The question is only spoken; the full value that docs/voice.md §9 promises on screen is shown nowhere.
   - Fix: send `"id": id.to_string()` at both places, or make mc accept a number. Add a cross-crate test, or move the event type into chora-kit.
2. **chat.rs:134, :146 with tools.rs:538 (`program`): `allow` and the "always" key match on the file name of argv[0] only.** Severity: **endangers**. See also 4.1.
   - Repro: with `allow: [foot]`, or after "Ja, immer" to "start foot", the LLM calls `run_program {"argv": ["/home/u/proj/foot", …]}` with a file the coder harness wrote. It runs with no question.
   - Programs and `CLIENT.METHOD` entries share one list, so an entry `mc.scene.show` also allows a program file of that name.
   - This contradicts docs/voice.md ("An entry matches exactly").
   - Fix: match the whole argv[0] exactly, or only bare names resolved through PATH, and refuse a `/` in both paths. Keep the program list and the method list separate.
3. **chat.rs:340, :392, :404-428: "always" permissions are never cleared**, not by `restart`, `settings.set`, nor by a stricter `confirm` or `always_words: []`. Severity: **endangers**.
   - Repro: answer "always" to `run_program foot`, then call `voice.restart` or tighten `confirm`. foot still starts with no question, because the check at chat.rs:904 runs before the question.
   - The docs say "until the agent restarts".
   - Fix: clear `allowed` in `configure()` whenever `confirm`, `allow` or `always_words` change, and always on `restart`.
4. **chat.rs:136 (`_ => return Some(tool.to_owned())`): for most tools "always" is far broader than the question asked.** Severity: **endangers**. See also 4.1.
   - "Always" to "Shall I type or click?" allows every later `inject_input` for the session. That includes typing into a terminal, which is arbitrary command execution.
   - "Always" to one `apply_commands` remove allows every later `remove` and `close`, although the docs say "`close` always asks".
   - `agent_start` (any project) and `configure_output` (any mode) behave the same way.
   - Fix: return `None` (a single yes) for `inject_input`, `apply_commands`, `code_task`, `agent_start` and `configure_output`, or put the target into the key.
5. **(plausible) core.rs:488-492 (`expire`): a yes spoken in time is lost when its transcript arrives after the deadline.** Severity: **endangers**.
   - Repro: say "Ja" at about 14.5 s of 15. The end wait plus ASR latency push the transcript past `confirm_until`. `answers()` is then false (core.rs:826), and "Ja" becomes a new LLM turn after "did not answer in time".
   - Fix: keep `confirm` while the gate records, or while a pending utterance has `onset <= confirm_until`.
6. **config.rs:924-931: `allow` entries such as `" foot"`, `"Foot"` or `"foot "` pass the load check but can never match.** Severity: **nit**.
   - Fix: reject entries that are untrimmed or contain whitespace. Require a bare program name or exactly `CLIENT.METHOD`.
7. **config.rs:32-43: a typo like `indicator: {corne: top-left}` is silently accepted** and gives the default `bottom-right`. The untagged `Indicator::At` lacks `deny_unknown_fields`, while the rest of `Config` refuses unknown keys. Severity: **nit**.
   - Fix: add `#[serde(deny_unknown_fields)]`.

## 2. Bugs in crates/, plugins/, tests/, dist/

1. **plugins/chora-construct/src/main.rs:1772-1782 (`load`, overlay branch): `construct.load` in an overlay loses adopted windows.** Severity: **breaks** (violates "no window may ever be lost").
   - Path: the overlay is active, and `tiling window.give {"to":"construct"}` makes `adopt(Hold::Own)` attach the window under `construct-windows` at opacity 0. Then `construct.load other` runs. The overlay branch takes all panes, and `Pane::remove` (:227) removes only the pane and mirror nodes.
   - Result: the surface stays owned by construct, invisible and without a pane. It cannot be given back, because give needs a pane. It returns only on `construct.leave` or when construct exits.
   - Fix: keep the window panes for the overlay as the flavor branch does, or release them first with `Remove { to: None }`.
2. **A reserved window can stay hidden indefinitely.** Severity: **endangers**.
   - Where: give.rs:78; construct main.rs:1192, :1223-1226; mc main.rs:2034, :2071-2079; core transaction.rs:342.
   - Cause: the core accepts any connected `to` and sends `surface.detached` only to it. The receivers then drop the reservation silently: construct `adopt` returns when `!self.active`, mc `adopt` returns when `!self.built`, and both only log a failed attach. `choractl give voice`, or a give to any other long-lived client that is not a window manager, is accepted the same way.
   - Result: the window has no owner, no parent and no visibility until `to` disconnects.
   - Fix (core): refuse a `to` that holds no claim, or expire an unattached reservation after N seconds (unreserve, then broadcast `Detached`). Receivers that decline must attach, then `Remove { to: None }`.
3. **plugins/chora-tiling/src/main.rs:610 with wm.rs:808: the claim is the bounding box of all tiling outputs, so a drop between them never hands over.** Severity: **breaks** (the hand-over feature).
   - Path: tiling runs on outputs 1 and 3, xp on output 2. A drop on output 2 lies inside `self.claimed`, so the code returns early and never asks `claimant_at`. The floating window stays parked on xp's output, still owned by tiling.
   - Fix: test the point against tiling's own output areas, or take the smallest claim including tiling's own.
4. **plugins/chora-tiling/src/wm.rs:951 (`float` uses absolute `window.at` unclamped); main.rs:612-631: floating windows can end off-screen.** Severity: **endangers**.
   - `at` is reset only when the window's own output disappears (wm.rs:188, :208). It is not reset when:
     - an output moves or changes mode;
     - a window dragged onto another tiling output loses that output;
     - a hand-over fails and the window stays where the pointer left it;
     - after a restart, the `at` persisted in meta points outside every output.
   - Fix: store `at` relative to the output, clamp it into the area in `float()`, and reset it when a hand-over fails.
5. **tests/gate-construct-windows.sh:114, :127 (`no_lost`): the "neither shows nor is listed" check can never fire.** Severity: **endangers** (the test hides losses).
   - Cause: `window.list` (construct main.rs:1954-1998) lists every surface, those without a pane as `pane:"none"`. So every non-pane surface is in `listed`.
   - The gate therefore cannot detect the losses it exists for: reserved or ownerless surfaces (2.2) and construct-owned surfaces at opacity 0 (2.1).
   - Fix: flag `owner == null` or a set `to`, and any construct-owned surface that is not in `panes`, as lost regardless of `listed`.
6. **(plausible) plugins/chora-tiling/src/main.rs:595-603 (`drag` guesses the window with `window_at(point)`).** Severity: **endangers**.
   - Cause: the `move` button event does not carry the surface that asked for the move.
   - Result: with overlapping floating windows, or a construct overlay flown over a tiling window, a title-bar drag can move or hand over a different window than the one grabbed.
   - Fix: put the surface id into the `input.button` event for `move`.
7. **plugins/chora-tiling/src/main.rs:269-272: `grab_all` is not retried when the claim appears.** Severity: **nit**.
   - Repro: start tiling with `--output X` before X exists. The `Super+BTN_LEFT` grab is refused (no claim yet, xp holds it) and is never retried.
   - Fix: re-run `grab_all` when `claimed` goes from `None` to `Some`.
8. **tests/gate-handover.sh:69, :197: no `pipefail`, so `w=$(ctl wait … | jq …) || fail` takes jq's exit status.** Severity: **nit**.
   - Result: a `ctl wait` timeout does not fire `fail`; later steps fail with a misleading message.
   - Fix: `set -o pipefail`, or `[ -n "$w" ] || fail`.
9. **dist/arch/PKGBUILD:83: it lacks `/etc/fonts/conf.d/09-chora-tex-gyre.conf`, which the RPM ships**, against the build-arch.sh promise of parity. Severity: **nit**.
   - Neither package ships a session `.desktop` for the construct flavor.
   - Fix: add both, or document why not.

## 3. Simplicity and leftovers

1. **services/voice/src/config.rs:342-352 (`PRIVILEGED`): a hand-kept copy of the privileged methods, already out of date.** Severity: **endangers**. **Fix now.**
   - Missing: `region.toggle`, `region.put` and `region.solo` (privileged in chora-mc main.rs:544-575, because they start docked apps). They run without a question.
   - Listed but not privileged: `pane.remove` (construct main.rs:2392).
   - The comment at tools.rs:875 ("every privileged method asks") is false.
   - Fix now: add the three methods and drop `pane.remove`. Later: pass the export's `privileged` flag through `choractl mcp`, delete the list and the docs paragraph in docs/voice.md:509-518.
2. **services/voice/src/client.rs:313-318, :413-419: `restart` duplicates `settings.reload` but is unprivileged.** Severity: **endangers / simplifies**. **Fix now.**
   - A client with only `methods.call` can apply a changed settings file this way.
   - Fix: delete `restart` and point README.md:124 and docs/voice.md:360, :559 at `settings.reload`.
3. **plugins/chora-mc/src/config.rs:72-108 (`#[serde(from = "Colors")]`): a backwards-compatibility shim** that fills in `magenta`/`cyan` for old files. Severity: **simplifies**. **Fix now.**
   - It contradicts "a map in the file replaces the default map".
   - Fix: delete it, and reject an unknown `{NAME}` in `Config::check`.
4. **services/voice/src/config.rs:34-60: `Indicator` is untagged `On(bool) | At{…}`**, keeping the rc.2 `indicator: true/false` syntax next to the new one. Severity: **simplifies**. **After the release.**
   - Fix: use a single form, `Option<{corner, offset}>`.
5. **The "smallest claim under a point" rule exists three times**: chora-scene scene.rs:349-354, core ipc.rs:119 and give.rs:49. `MOVE = "move"` is defined twice (core input.rs:648, give.rs:19). Severity: **simplifies**. **After the release.**
   - Fix: one helper and one constant in chora-scene.
6. **`window.give` handling is copied into four plugins, with different answers.** Severity: **simplifies**. **Fix now** for the answers, **after the release** for the helper.
   - Tiling and mc answer the surface id; xp and construct answer `null`. docs/api.md:1014 promises the id.
   - The optional `surface` is never omitted by `choractl give`.
   - Fix: one answer everywhere, a required `surface`, and a kit helper for "claimant_at → give → else keep".
7. **The named-YAML store is duplicated**: chora-mc scene.rs:217-283 and chora-construct file.rs:68-185 (`check_name`, load, save, list). Severity: **simplifies**. **After the release.**
   - Fix: move it into chora-kit.
8. **Writing an embedded shader to the runtime dir exists twice** (mc main.rs:343-361, construct main.rs:374-389), and the runtime-dir lookup is repeated a third time in xp main.rs:179. Severity: **simplifies**. **After the release.**
   - The code uses a plain `fs::write` with a fallback to the shared `/tmp`, where another local user could plant a shader first.
   - Fix: one `chora_kit::xdg::runtime_file` that uses `write_file` and refuses to work without `XDG_RUNTIME_DIR`.
9. **The `indicator.claim` client state machine is duplicated** (mc main.rs:318/678/876, xp wm/voice.rs:80-97), and the `voice.confirm` payload is parsed two ways (typed in mc, raw `Value` in xp). This is how 1.1 slipped through. Severity: **simplifies**. **After the release.**
   - Fix: a shared `VoiceHud` helper and event types in chora-kit.
10. **services/voice/src/chat.rs:73-110 (`announces`): English and German word forms are hard-coded in Rust**, while every other word list is a setting. Severity: **simplifies**. **After the release.**
11. **Docs drift.** Severity: **nit**. **Fix now** (docs only).
    - docs/configuration.md:615 says construct is "step K2b", but docs/roadmap.md:500 defines K2b as GNOME and prompt panes.
    - docs/configuration.md:422 omits `lead`, which scenes/morning.yaml uses.
12. **Two ways to configure a drag modifier**: construct uses `modifier: Super` (config.rs:104, hand check at :115), while tiling uses a `Binding` (`MOD+BTN_LEFT: drag`). Severity: **nit**. **After the release.**

## 4. Security

How peers are authenticated: the sockets are 0600 inside a 0700 directory, and name squatting on the kit plugins is blocked by `resume` (ipc.rs:1060-1080). The realistic attackers are therefore confined client sockets and an LLM steered by injected text.

1. **chat.rs:119-154, tools.rs:538: "always" and `allow` ignore the arguments** (see 1.2 and 1.4). Severity: **breaks**.
   - Attack: the user once answers "yes, always" to an `inject_input` question. Later, text in a window title, in a file the agent reads, or in an mc notification makes the LLM type `curl …|sh⏎` into a terminal, with no question.
   - The same works with `run_program ["foot","-e","sh","-c",…]` once `foot` is always or allowed, and with `mc.app.open {command:…}` once that method is always.
   - Fix: never offer "always" for `inject_input` or anything that runs commands. Build the key from the method plus canonical args. Match the full argv[0] and the args.
2. **mc.rs:162 with core ipc.rs:309-334, :563: `voice.confirm` leaks secrets to every subscriber.** Severity: **endangers**.
   - `detail` (chat.rs:158) carries the full settings patch, for example `llm_headers: {Authorization: Bearer …}`. Every connection subscribed to `voice.*` receives it, including confined client sockets.
   - The journal redacts these values (chat.rs:1163-1175); the event does not.
   - Fix: apply the journal redaction to `detail`, or send `confirm` only to the claimers. Related, and older than this range: `voice.settings.get` returns `*_headers` unredacted.
3. **config.rs:310-352: `confirm` is a deny-list, and the export's privileged flag never reaches the agent** (see 3.1). Severity: **endangers**.
   - The privileged `mc.region.*` methods run unasked.
   - The sensitive but unprivileged `window.give`, `mc.agent.set` and `mc.notify.push` also run unasked.
   - Fix: confirm every privileged method by default, and add `window.give`.
4. **config.rs:487, :558: settings changes are weakly protected.** Severity: **endangers**.
   - The spoken question for a security key omits the value ("Shall I set allow?"). One "yes" from a user who is not looking at the screen lets injected text widen `allow` to `[sh, bash, mc.app.open]`. Because of 1.1, the value is not shown on screen either.
   - These keys are not security keys, so one session "always" covers them: `indicator`, `earcons`, `confirm_timeout_ms`, `ptt_key`, `wake_words`/`sleep_words`. Example: one "always" on an `indicator` change lets the LLM later silently set `indicator: false`.
   - Fix: refuse `allow`/`confirm` changes by voice, or speak the diff. Add the keys above to `security_key`.
5. **client.rs:170, :306 (`indicator.claim`), :267 (`mute`), :282/:405 (`talk`) are all unprivileged.** Severity: **endangers**.
   - Attack: a confined client with `methods.call` claims the indicator and stays connected, which hides the dot. It then calls `voice.talk {down:true}` or `voice.mute {mic:false}` to record silently; the audio goes to the ASR and the LLM. The claim is released properly on disconnect, but the client simply never disconnects.
   - Also, xp assistant.rs:70 shows no pose for "sleeping", so once xp holds the claim nothing shows that the mic still listens for wake words.
   - Fix: make `indicator.claim`, `talk` and `mute{mic:false}` privileged, or restrict them to named claimers. Show the sleeping state.
6. **core ipc.rs:1302-1318 (`ipc_grab`), :402-418 (`button_grab_for`): shared button grabs are granted to any client that holds a claim**, and confined clients may set `claim` (ipc/clients.rs:288). Severity: **endangers**.
   - Attack: a confined client creates a group whose claim is the output minus ε and grabs `Super+BTN_LEFT`. As the smallest claim it wins every Super-drag: the window manager's move and resize break, and the client receives the pointer stream.
   - Fix: refuse `claim` from client sockets, or allow shared grabs only to window managers.
7. **core handlers.rs:324 → input.rs:654-667 (`start_move`): the serial check accepts any stale press.** Severity: **endangers** (confirmed in code; the end-to-end effect is plausible).
   - `pressed()` only checks the client's *last* press serial. `start_move` then ends whatever grab is active.
   - Attack: a background client keeps an old serial and sends `xdg_toplevel.move` while the user drags in another window. The user's drag is cut off, and the attacker's window follows the cursor (click-jacking).
   - Fix: require `pointer.has_grab(serial)` and that the grab's focus is the requesting surface.
8. **give.rs:33: `window.give` is unprivileged, and the core accepts any connected `to`** (see 2.2). Severity: **endangers**.
   - Any `methods.call` client, or the LLM without a question, can make a window manager hand a window to a passive confined client or to `voice-mcp`. The window disappears from view, and the receiver can then focus, close or configure it.
   - Taking a window one does not own is not possible (scene.rs:413).
   - Fix: make `window.give` privileged and require `to` to hold a claim.
9. **xp wm/voice.rs:35-50, :162-166 and mc main.rs:845-855, :888: anyone can spoof the voice state.** Severity: **endangers**.
   - The `voice.*` subscription also matches a client named `voice.evil`. xp then stores it as the voice agent and sends the paperclip's mute toggle to it, so the balloon says "The microphone is off" while voice still listens.
   - mc trusts `name` in the event data and exports an unprivileged `mc.agent.set {name:"voice"}`, so any client or the injected LLM can set the state mc shows in place of the claimed dot.
   - Fix: require `client == "voice"` exactly, and refuse `name: voice` from any other client.

Checked and clean: settings writes (0600 via `write_file`); the `voice.confirm` method cannot set "always" (only a spoken word can); the `move` grab ends on release, lock and safety reset; construct adoption only takes windows given to it, and `construct.load`/`save` are privileged.
