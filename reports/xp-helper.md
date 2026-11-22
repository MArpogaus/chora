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
