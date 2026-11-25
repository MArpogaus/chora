# p2-r11-tiling (RC 2 persona round 11, tiling)

## r11

Lane: cloud, branch claude/p2-r11-tiling at 79ed970f (train143 16977e22 + act-wait r1 + session-layout r1). **Binaries of the round: sha 79ed970f0068d2740fabfceea6ebff9f0c7c770d** (`round.sh snapshot`; no product or test code changed). Step 0: `git fetch --unshallow`, `cargo check --workspace`, `cargo build --workspace` green; `round.sh --help` read; smoke on tiling+xp (`round.sh begin smoke`, `check.py end smoke`): "check.py: no contract for smoke" (expected; the old `round.sh smoke` T7-on-one-look failure is a different command and was not run), stopped clean. fuzzel 1.11.1 (ti-1 not in play). Sessions 1280x800 per output, 2 slots, PERSONA_CHECK=1, helper transcript seeded per session (user "Hi", agent "Hello"), haiku personas via Agent (one agent per persona run, goals sent one at a time, no coaching; the T7 goal got one added request, "say in your own words how you decided which place was the xp look"), act.py only, one `begin` before each goal and `check.py end` after it (outputs in media/2026-11-25/persona-tiling-r11-checks). I did not audit the subagents' tool calls beyond their reports.
Layout: tiling alone for Kemal and the beginner; ricer tiling+xp = tiling HEADLESS-1, xp HEADLESS-2 (session-layout r1). Waybar: pgrep count 3 per session at each start (3, 6, 6, ...), none left after `round.sh stop`; `pgrep -fa 'chora --headless'` empty at the end.
Goal words: **T7 from the README "Tiling goals" word for word: "Send the terminal over to the xp look (the other desktop that is running)"**; the others r10's words (Kemal T1 "Open a terminal.", L1 "Get two terminals open next to each other, move over to the other one, then close one of them.", T3 "Ask the helper what time it is."; beginner T1 "Open a program so you can type commands (a terminal).", T2 "Find out which keyboard shortcuts this desktop has.", T4 "Open the power menu (the one for shutting down or locking) and then close it again without choosing anything.", T6 "Try to start a program called frobnicate."; ricer T1 "Open a terminal.", T7, L3 "Go to a second workspace and then come back to the first one.", T2, L2 "Change the gap between windows from 12 to 20 in the settings and make the desktop use it."). Ricer order T1, T7, L3, T2, L2 (README).
Caps written before the run: Kemal T1 6, L1 10, T3 9 (25; T5 scripted by the driver, 4 more); beginner T1 5, T2 4, T4 5, T6 6 (20); ricer T1 6, T7 10, L3 8, T2 5, L2 12 (41). Used (act.py log): Kemal 1 25, Kemal 2 23, beginner 17, ricer 1 27, ricer 2 22 actions (looks 13, 12, 9, 14, 12); no failed action, no retry past 3. Every task of every persona ran.
Captures: media/2026-11-25/persona-tiling-r11-{kemal1,kemal2,beginner,ricer1,ricer2} (shots, actions.jsonl, calls.log, breaches.jsonl; ricer evidence trimmed to 3), media/2026-11-25/persona-tiling-r11-checks/*.txt.

| Persona | Task | (a) contract | (b) breaches | (c) reached |
|---|---|---|---|---|
| Kemal run 1 | T1 | pass | 0 | yes (Super+Return) |
| Kemal run 1 | L1 | pass | 0 | yes: Super+Return, Super+Right (workspace 2, ti-12/28), Super+1, Super+Tab, Ctrl+d; no unbound chord, key list not opened |
| Kemal run 1 | T3 | pass | 0 | persona: no ("Hello", fake answer, known); bar word "atlas · idle", chat panel, typed |
| Kemal run 1 | T5 (scripted) | pass | 0 | n/a |
| Kemal run 2 | T1 | pass | 0 | yes |
| Kemal run 2 | L1 | pass | 0 | yes: Super+Return, Super+Right, Super+1, click, Ctrl+d |
| Kemal run 2 | T3 | **FAIL** helper never worked (idle x6) | 0 | persona: no; clicked the Listening badge, then typed the question into the terminal ("bash: what: command not found"); never found the bar word or Super+a |
| Kemal run 2 | T5 (scripted) | pass | 0 | n/a |
| beginner | T1 | pass | 0 | yes (Apps, click Foot) |
| beginner | T2 | pass | 0 | yes (Keys button, 2 actions) |
| beginner | T4:power | pass | 0 | yes (power icon, Escape) |
| beginner | T6 | pass (Nothing found row) | 0 | persona: no (reads "Nothing found" as failure) |
| ricer run 1 | T1 | pass | I6 xp only | yes |
| ricer run 1 | T7 tiling -> xp | **pass** | I6 xp only | persona: reached, not verified (clicked "Looks: xp", picker, "Move window to the xp desktop") |
| ricer run 1 | L3 | **FAIL** expected the focus with tiling, holders ['xp'] | I6 xp only | yes (tag 2, tag 1) |
| ricer run 1 | T2 | pass | I6 xp only | yes (Keys click) |
| ricer run 1 | L2 | **FAIL** focus holders ['xp'], gap_inner still 12, no focus after reload | I6 xp only | persona: no (12 actions; Super+s x2 did nothing, click, Super+S opened the editor with a swap-file warning) |
| ricer run 2 | T1 | pass | I6 xp only | yes |
| ricer run 2 | T7 | **pass** | I6 xp only | persona: reached, not verified (same path, Return on the entry) |
| ricer run 2 | L3 | **FAIL** holders ['xp'] | I6 xp only | yes |
| ricer run 2 | T2 | pass | I6 xp only | yes |
| ricer run 2 | L2 | **FAIL** gap_inner still 12 | I6 xp only | persona: no (Super+S opened the editor under the key list that T2 left open, three Escape did not close the key list, persona stopped) |

Breaches outside known-red: Kemal 0 / 0, beginner 0, ricer 1 I6 x51, ricer 2 I6 x30, all the xp desktop (tray clock 3.21; task buttons "foot" xp-task-s1/s2/s3 3.86, a new element name). No I1, I2, I3, no tiling row.

### Noise check (Kemal and the ricer twice; both runs above)
Kemal: T1, L1, T5 identical in contract across the two runs (pass), L1 closed by the shell (Ctrl+d) in both, never a close key, and in both Super+Right (a bound key: workspace) cost one action; neither run pressed an unbound chord. T3 differs: run 1 reached the helper by the bar word (contract pass, answer "Hello" known), run 2 never found the helper (contract red); so T3 is not a stable pass. Ricer: T1, T7, T2 pass in both, T7 now green 2 of 2 where r10 was red 2 of 2; L3 red in both with the same line, L2 red in both (run 1 with the focus line as well, run 2 with the key list that Escape did not close). The L3/L2 focus red is not noise.

### Fix check against r10
| Row | Result | Capture |
|---|---|---|
| ricer T7 (ti-55: window moved to xp; the persona's own words on how it found xp) | **pass, 2 of 2 runs** (contract: a window changed look). Run 1: "The top bar says 'Looks: xp', which names the look. No other desktop was visible ... I was guessing from the labels"; run 2: "The only text on screen that matched was 'Looks: xp'. Clicking it showed the menu entry 'Move window to the xp desktop'". Neither used workspace 2. Caveat: I took no capture of HEADLESS-2 (act.py look is the first output); the persona itself says "reached, not verified" | ricer1 shots 004-006, ricer2 shots 004-006, checks/ricer{1,2}-T7.txt |
| Kemal L1 (ti-56: after an unbound Super chord a look within 8 s shows the hint; does the persona open the key list) | **chip: pass (driver)**; persona side **not reached**. act.py now returns from Super+q in 0.01 s; the look right after shows "Super+Q does nothing here. Super+F1 shows all keys." in both Kemal sessions; the look after Super+x shows the Q chip fading, no X chip. No persona pressed an unbound chord in L1 (both used Super+Right, bound), key list not opened | kemal1 shots 012-013, kemal2 shots 011-012 |
| helper row (ti-50) | **pass**: the Apps list has "Chat with a helper (atlas)" as its first entry (beginner T1, persona noted it, did not click) | beginner shot 002 |
| beginner T4 (ti-27) | **pass**: T4:power pass; T4 began with the terminal showing, power icon, Escape | beginner shots 004-007, checks/beginner-T4_power.txt |
| one bar per output (sh-41) | **pass**: 1 waybar per session at each start (the tiling output of the ricer has its bar; xp has its own taskbar, no second waybar), none left after stop | start counts, ricer1 shot 001 |
| beginner T6 (ti-31 known) | contract pass, persona still reads "Nothing found" as not reached; ti-31/ti-49 stay known | beginner shots 008-009 |
| rtd4-1: tag 1 shows no xp taskbar | **pass**: ricer 1 first look (tag 1 empty): the tiling bar and the dark wallpaper, no xp taskbar, no xp desktop; ricer 2 persona: "empty desktop with a top bar, no windows", and the terminal fills tag 1 | ricer1 shot 001, ricer2 shots 001-002 |

### triage.py (the three r11 files only, run on a copy in a scratch dir; FORMAT OK)
```
== tiling / I6 (1)
  2026-11-25-p2r11ti-ricer.md: Xp I6: tray clock and "foot" task buttons under the contrast bar (0 days)
== tiling / contract L3 (1)
  2026-11-25-p2r11ti-ricer.md: L3: after a window was sent to xp, the keyboard focus stays with xp (L3 red 2 of 2, L2 red in run 1) (0 days)
== tiling / contract T3 (1)
  2026-11-25-p2r11ti-kemal.md: T3: helper not reached; run 2 never finds the helper word (contract red again, harness reads the fake answer) (0 days)
== tiling / none (10)
  2026-11-25-p2r11ti-beginner.md: ux: ti-50 holds: the Apps list has "Chat with a helper (atlas)" as its first entry (0 days)
  2026-11-25-p2r11ti-beginner.md: ux: an unknown program name: "Nothing found" only after Return, the launcher stays open, read as a failed start (0 days)
  2026-11-25-p2r11ti-beginner.md: ux: the power menu preselects Lock; the key list is dense and small (0 days)
  2026-11-25-p2r11ti-beginner.md: ux: the terminal fills the screen and shows no frame (single window) (0 days)
  2026-11-25-p2r11ti-kemal.md: ti-56: unbound chord hint chip seen on a look right after the key (driver); no persona pressed an unbound chord (0 days)
  2026-11-25-p2r11ti-kemal.md: ux: Super+Right switches workspace instead of moving focus (ti-12, ti-28 known) (0 days)
  2026-11-25-p2r11ti-kemal.md: ux: L1 closes by the shell (Ctrl+d), not a close key; locale warning in every terminal; the prompt is root (0 days)
  2026-11-25-p2r11ti-ricer.md: T7 passes: the persona finds the xp look by the bar word "Looks: xp" (ti-55) (0 days)
  2026-11-25-p2r11ti-ricer.md: ux: the persona cannot see the xp desktop by a look (act.py look is the first output) (0 days)
  2026-11-25-p2r11ti-ricer.md: ux: Super+S opens the raw config file in a vim-style editor; the swap-file prompt and the dense key list slow the goal (0 days)
waiting 13, overdue 0, missing ux 0
OK
```
Feedback: feedback/2026-11-25-p2r11ti-{kemal,beginner,ricer}.md (each holds both runs where there are two).

### Stop rule (p2-plan.md)
**No: tiling does not meet round 11.** Not all tasks reached with a green contract: ricer L3 red 2 of 2 and L2 red 2 of 2 ("expected the focus with tiling, holders ['xp']"; L2 also gap_inner unchanged, persona not reached), Kemal run 2 T3 red (helper never found). New check-backed finding: after T7 sends the window to xp, the keyboard focus stays with xp (L3 contract, both runs); in ricer run 1 the first two Super+s presses did nothing visible and the xp taskbar then listed three "foot" buttons, in run 2 Escape did not close the key list (a reading, not confirmed by a HEADLESS-2 capture). It is a MAJOR candidate (workaround: click the desktop) for the PM to classify; cause open between product (focus follows the moved window; a tiling tag click does not take it back) and harness (order T7-first, or the click that session.sh uses to choose the first look). ti-55: T7 green 2 of 2 for the first time since r6, with the layout and order changes of this round (harness changes plus act-wait; no product fix): ti-55 can be closed on this evidence, with the caveat that the persona cannot verify the xp desktop by a look. Fix checks holding: ti-56 chip (driver), ti-50, ti-27, sh-41, rtd4-1, T1, T2, T4, T5, T6, L1 contracts. Open for the PM: (1) classify the L3/L2 focus red (new), (2) T3 for Kemal is not stable (found by the bar word in run 1, not at all in run 2), (3) the ti-56 persona side still has not been observed, (4) act.py look cannot show the second output, so T7 stays "reached, not verified" for a persona. No ledger edits.
