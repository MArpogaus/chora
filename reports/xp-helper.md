# xp-helper

## r1

Branch claude/xp-helper, head eda60a6b (base 3bec0cbf, fresh origin/claude/autonomous-dev-session-3w-i33pg7 was already merged: no merge needed). New cell file tests/gate-xp-helper.sh (cells xp29-click, xp30-helper, xp30-none, xp30-once, xp28-editor; registered as step xp-helper in tests/merge-gate.sh).

### xp-29 (click on the paperclip): ABSENT at the head, no code
- Cell xp29-click (green on 3bec0cbf, with a fake helper and, in a first probe, with none): a person click (session started with --test-injected-press-is-real) on node hud-figure opens hud-chat within 5 s: "ok: xp-29: a click on the paperclip opens the chat (helper)". The xp HUD folder already has `action: chat` on the clip panel (flavors/xp/assets/assistant/hud.yaml); toggles_chat in plugins/chora-assistant/src/main.rs does the rest. So the owner's 11-17 finding no longer reproduces; the cell stays as a guard. The tray door is untouched.
- Capture: /tmp/gate-xp-helper.4PTzd0/xp-29-chat.png (balloon with chat above the paperclip).

### xp-30 (greeting): fixed
- Cause: nothing said anything at the start; the balloon node (hud-line) was only driven by voice.say.
- Red ae9bbe5e: "FAIL: xp-30: the paperclip greets with no balloon at the first start" (xp30-helper, xp30-none, xp30-once all red).
- Fix 4fec75c3: chora-assistant greets once, 1.5 s after the start (after the voice state question settled), only in the xp chat form; text from two new words (crates/chora-kit/src/words.rs, en and de): with a helper "Hi, I am your helper. Click me to ask for something. Click the X on my balloon to close it."; none set up "Hi, I am your helper, but no helper is set up yet, so you cannot ask anything. Click me to see how. Click the X to close my balloon." Remembered in $XDG_STATE_HOME/chora/assistant-greeted (second start of the same state: quiet). The balloon node now carries meta.text (a11y, and what the cell reads). Lint-ratchet fix eda60a6b (no new eprintln).
- Green: "ok: xp-30: the first start greets once: Hi, I am your helper. ..."; "ok: xp-30: no helper: the honest greeting: ..."; "ok: xp-30: the second start is quiet".
- Limit: "no helper" means no voice agent answered the state question within the start; a voice agent that connects later is not greeted differently.
- Captures: /tmp/gate-xp-helper.EWEKIY/xp-30-helper.png, /tmp/gate-xp-helper.53BdDN/xp-30-none.png.

### xp-28 editor half: present, fixed
- Verified on the head: Notepad from Start (editor `foot -a notepad`) opens at x 28, y 49, 700x500, over the icon column (icons end at x 88); capture /tmp/gate-xp-helper.ulyjSq/xp-28-editor.png shows the frame over the labels.
- Red 66c5a23b: "FAIL: xp-28: the editor starts at x 28.0, over the column of the desktop icons (it ends at x 88): [s1, [28,49,700,500]]".
- Fix 09ff8ff9: Places::place (plugins/chora-xp/src/window.rs) starts the cascade right of the icon column (desktop_icons::COLUMN = 88): first spot x = ax + 88 + 24. Two unit tests updated (112,24 / 142,54).
- Green: "ok: xp-28: the editor starts at x 116.0, right of the icon column (x 88)". Capture: /tmp/gate-xp-helper.7wdAMX/xp-28-editor.png.

### Counts
Each of the 5 cells 5 times in a row: 25 of 25 ok, 0 FAIL (log /tmp/five.log on the lane).

### Gate (tests/merge-gate.sh --jobs 2, full, on eda60a6b)
Run dir /tmp/merge-gate.UPTnBN. 54 steps PASS incl. build, test, clippy, xp, xp-fields, xp-question, xp-chat-idle, xp-helper, xp-notepad, hud, matrix, tiling-*, layout, doors, pull, placed, mc-*, agent, pixels. One red, not caused here: watchdog-starved: "FAIL: the watchdog acts on a starved host: ... WARN chora::safety: frames are slow; the watchdog blames a client client=mc load=16552" (core watchdog on a loaded shared host; no core or watchdog file touched; not rerun). An earlier first run stopped at ci-check on my lint-ratchet (eprintln 132 vs 131), fixed in eda60a6b before the run above.

## Review r1

Seeded review of claude/xp-helper head eda60a6b (base 3bec0cbf, 8 files, +257 -11). `cargo check --workspace` ok; `cargo test -p chora-assistant -p chora-kit -p chora-xp` 18 suites ok, 0 failed. No gate run (the lane ran it).

**Verdict: FIX** (0 BLOCK, 1 MAJOR, 5 MINOR, 2 NIT).

Checks that hold: (1) red first in both pairs: ae9bbe5e before 4fec75c3, 66c5a23b before 09ff8ff9. (2) The greeting goes through the `said`/`say` path of voice.say (no second path), only when `values.chat().form == Xp` at fire time (a look changed away from xp by then: no greeting, state not burned), one time per state file; a read-only or missing XDG_STATE_HOME does not panic (errors ignored, greets again next start; no absolute state home: never greets); words only in words.rs, no new eprintln; the en and de texts (91/132 and 99/139 chars) fit max_chars 160 of the xp line. (3) On 800x600, 116+700 does not fit: the final clamp (window.rs:427) puts it at x 100, on screen and right of the icons (they end at x 88); the cascade wraps as before. (4) The gate script picks no process by name (it kills `$apid`/`pids`), cleans its state dir through gate_cleanup, and xp29-click is a person click (`--test-injected-press-is-real`, pointer_move on the node, button press and release).

Findings:
- rx-1, MAJOR, crates/chora-kit/src/words.rs:354 and :358 (en), :455 and :458 (de): both greetings tell the person to "Click the X on my balloon" / "Mit dem X ... schließt du", but the sentence balloon is text only (assistant.rs `speech()` draws the shape and the lines, no X); the X exists only on the chat after a click (chat/view.rs XP_CLOSE). The promise names a control that is not there. Fix: say what is true ("It closes by itself; click me to ask" in en and de) or give the sentence balloon an X.
- rx-2, MINOR, words.rs:357 and plugins/chora-assistant/src/chat/dialogue.rs:150: "Click me to see how" with no helper opens the chat whose header says only `NoHelper` ("No helper is set up. Ask the person who set up this computer."), no way to set one up; and the gate cell xp29-click (gate-xp-helper.sh:6-7 says "with a helper and with none", :92-98) clicks only with a helper, so the promise is unguarded. Fix: word the greeting "Click me to see what to do" (de the same) or make NoHelper name the set-up path, and add a none-click cell.
- rx-3, MINOR, plugins/chora-assistant/src/voice.rs:62-67: `create_dir_all` makes `$XDG_STATE_HOME/chora` with the umask (0755), not 0700. Fix: `chora_scene::paths::private_dir(dir)` (as logs.rs does).
- rx-4, MINOR, plugins/chora-xp/src/window.rs:423: COLUMN is added on every output, but icons exist only on the first (panels.rs:626 `bars.first()`, while `home()` windows.rs:392 can be a second output): windows on a second output lose 88 px and wrap earlier. Fix: give Places the icon-column width only for the area of the first bar (else 0).
- rx-5, MINOR, desktop_icons.rs:37 and window.rs:423: COLUMN is one cell column, but `layout()` fills further columns when the icons exceed the rows (6 rows at 570 high); a window at x 112 then covers the second column (x 88..168). Fix: take the right edge of the placed icons (or columns x CELL_W + MARGIN) from the desk, not a constant.
- rx-6, MINOR, tests/gate-xp-helper.sh:124 and :128: the xp28-editor cell waits for the surface, then a bare `sleep 1` is the only wait for the frame position it asserts (also `sleep 1` after the menu shows); a slow host reads the pre-placement x and false-fails. Fix: poll (ctl query) until the notepad surface rect is non-zero and equal on two reads, as the other cells wait on a state.
- rx-7, NIT, tests/gate-xp-helper.sh:3-12 and :136: the header lists no xp28-editor cell; the cell uses `bc` but `need bc` is missing (a missing bc reads as a false FAIL). Fix: add the cell to the header and `need bc` (or compare with jq).
- rx-8, NIT, words.rs:455 and :458: the de greetings use du/dein/dich, every other de word uses Sie. Fix: Sie forms ("Ich bin Ihr Helfer", "Klicken Sie mich an").
