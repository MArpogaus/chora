# ti-r4: tiling persona r4 (T1, T3)

## r1

Lane: cloud, claude/ti-r4 from 1c9e64f7 (session head; no merge needed, it is an ancestor). Brief briefs/ti-r4-r1.md. Step 0 green (unshallow, `cargo check --workspace`, `cargo build --workspace`). Commits on claude/ti-r4: red 6f300250, green e86b3cfa.

| Finding | Verdict | Cause line | Test | Fix |
|---|---|---|---|---|
| ti-32 MAJOR | **setup, not product** | tests/persona/session.sh: the second look (xp) joins last and its desktop icons hold the keyboard (plugins/chora-xp/src/wm/state.rs `node_focused`, "the icons that hold the keyboard with no window of another look focused stay"); the pointer is at 0,0, outside the claim of tiling (claim [0,80,1280,690]); `choose_manager_of` (crates/chora/src/state.rs) chooses a look for its grabbed key only when its claim holds the pointer or the keyboard focus; rule 3 of docs/api.md (Window managers) then gives the new window to the owner of the keyboard focus, xp | `tests/persona-selftest.sh` section "tiling+xp: Super+Return opens a tile of tiling" red 6f300250 ("opened a window of xp, not of tiling"), green e86b3cfa | tests/persona/session.sh `first_look_takes_the_keyboard`: after the companions, one injected click on the empty claim of the first look (needs `--test-injected-press-is-real`, which round.sh sets); no product change |
| ti-33 MINOR | **works, closed** | - | `tests/gate-chat-keys.sh` "with one helper, typed text and Enter reach the helper" (green, log reports/evidence/ti-r4-r1/gate-chat-keys.log) and the persona repro below | none |

### ti-32 repro (persona session tiling+xp 1280x800, binaries of 1c9e64f7, CHORA_TEST_FLAGS=--test-injected-press-is-real as round.sh; script reports/evidence/ti-r4-r1/repro-ti-32.sh)

Without the click (as the persona run), `choractl query kind:surface` after Super+Return, then `state tiling`, then Super+2:

    surface s1 owner xp parent xp-win-s1 focused True
    tiling {'holder': None} []
    after Super+2: surface s1 owner xp visible True

At the start of the session `choractl state xp` says `focus: {holder: xp}` (before any key), `state tiling` says `holder: null`. The persona saw a floating xp window that no tag hides: it is xp's window.

With one click on the empty claim of tiling first (the fix), same keys:

    surface s1 owner tiling parent tc-s1 focused True
    tiling {'holder': 'tiling', 'surface': 's1'} [(False, [1])]
    after Super+2 tiling [{'output': 'HEADLESS-1', 'shown': [2]}] [{'desks': [1], 'floating': False, 'shown': False, 'hidden': None}]

so it is a tile (floating False, desk 1) and Super+2 hides it (`shown` false, no rect). The same runs with origin/claude/keys-route e16d558e merged locally (merge 8264b74a, not pushed, built in a separate target dir) give the same two outputs: owner xp without the click, owner tiling with it. keys-route changes nothing here.

Why it is not a product bug: docs/api.md "Window managers" rules 2 and 3 say the key of a look chooses it only when its claim holds the pointer or the keyboard focus, else the owner of the keyboard focus gets the window ("opens where the person is"); the person of that run had the keyboard on the xp desktop (xp joins last, takes the icons) and the pointer in the corner. After a real click on tiling (or Super+Alt+1 give: s2 then opens as a tiled `tiling` window, checked too) the same key opens a tile. Notes for the PM, no change made: (1) a person with a real keyboard only, in a tiling+xp session, who never touches the mouse, gets the xp window for Super+Return (rule 3, xp owns the keyboard on an empty desktop); that is the documented rule, not a defect of this round; if the PM wants "the first look wins at session start", it is a core or xp decision (D55). (2) input.rs `key()` line 174 chooses a look for a grabbed key only when `!injected`, so `--test-injected-press-is-real` does not make an injected key choose a look, unlike an injected button (`choose_manager` honours the flag at input.rs:713); the selftest does not need it, since the click chooses.

### ti-33

Persona session tiling 1280x800 (flag set, PERSONA_CHECK=1): Super+a opens the panel, `act.py type hi` puts "hi" in the input (capture reports/evidence/ti-r4-r1/ti-33-chat-hi.png, the input shows `hi|`), Return sends it: `voice/entries.json` is `[{"seq": 1, "role": "user", "text": "hi"}, {"seq": 2, "role": "agent", "text": "Hello"}]` (reports/evidence/ti-r4-r1/ti-33-entries.json). So the chat takes the keys after Super+a; Kemal never typed (r4 Kemal: "the persona found no key way into the input"; the placeholder "Type a message, then Enter." is the only hint). No product change; T3 red of that round was the persona not typing, a persona result, not a defect. No red test needed; the proof is gate-chat-keys.sh above.

### Gate

`tests/merge-gate.sh --jobs 1` (no --only) on claude/ti-r4 at e86b3cfa (the session head 1c9e64f7 is an ancestor, so no merge): red-first, identity, pre-commit, gate-lib, ci-check, build, test, clippy, 2d-guard, package-deps, package-lists, xp, matrix, virtual, xp-faces, xp-paperclip, xp-fields, xp-question, xp-notepad, hud, layout, tiling-audit PASS; **tiling-look FAIL** at the first stop: the known red, not mine: `bar-font` the clock is 120.0 wide, not Inter 14 px weight 500 (100-119), in ink-1280x800, ink-2560x1600@2, paper-1280x800, paper-2560x1600@2 (lane clock-width); not fixed, not rerun. The gate stops at the first failed step, so the rest ran as `--only tiling-empty,chat-keys,doors,acts,last-words,cube,pull,placed,construct-hud,construct-extra,out,winit,wm-pane,pane-keys,walk,keys,pixels,mc-paths,mc-sofa,mc-column,mc-handover,launcher-click,voice-acp,voice-events,agent,e2e` (a second run, never a push): all PASS, `acts` included (it holds the new tiling+xp section of tests/persona-selftest.sh; the known acts tiling I14 red, lane sh-32, did not break the step). Deviation: the second run is `--only`, because the first stopped on the known red.
