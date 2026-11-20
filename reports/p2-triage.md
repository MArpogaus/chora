# p2-triage (RC 2, P2 plan row 2)

## T1 (2026-11-20, sonnet high, ledger work on claude/session-log, code checkout read-only)

Step 0: `git fetch --unshallow` and `cargo check --workspace` green on b97369d1. Input: the twelve feedback/2026-11-20-p2r1-*.md (48 findings: xp 15, tiling 8, mc 12, construct 13), the four round reports, p2-plan.md sections 2 and 3, the six ledgers. Captures read to confirm: cook 006/008/011 (mc-42, mc-43), nurse 004/005/009/013 (mc-44, mc-45), Erika 017 (xp-34). Code read for names only: plugins/chora-xp/src/apps.rs, plugins/chora-tiling/tiling.yaml:92, tests/persona/check.py.
Feedback `status:` lines are left to the PM (feedback/README.md); each outcome is in the tables below.

Rules used: severity by p2-plan 2 (BLOCK = check red on a shared-bar task or main path, or lost window; MAJOR = check red with a workaround); `verify` = check none or red but no capture confirms; harness faults named. Known in flight (brief item 4): launcher click (lane launcher-click), black pane (sd-h1, lane black-pane), stray keys (sh-23, sh-24, review stray-grab), construct C2, T5, xp start timeout (lane p2-harness, `rejected: harness`). No finding touched sd-h1 or sh-23.

### triage.py (run on the inbox before any status change; no file argument exists, so the lines of the twelve files are shown, 48 of 75 waiting)
```
== construct / I7 (2)
  2026-11-20-p2r1-demo.md: I7 on every output through the whole run (0 days)
  2026-11-20-p2r1-explorer.md: I7 on every output through the whole run (0 days)
== construct / contract C1 (2)
  2026-11-20-p2r1-demo.md: Every XP face says "did not start" while the notice says "The other faces still work" (0 days)
  2026-11-20-p2r1-explorer.md: The cube row leads to a "Starting XP" spinner, then a failure notice; the cube only came by Alt+Escape (0 days)
== construct / contract C2 (1)
  2026-11-20-p2r1-demo.md: A window cannot be pulled out of its desktop by the gesture a demo viewer tries (0 days)
== construct / contract T1 (1)
  2026-11-20-p2r1-explorer.md: Programs > Foot opens the helper chat instead of a terminal (0 days)
== construct / contract T4:lock (1)
  2026-11-20-p2r1-demo.md: Top-bar Lock click and Super+Shift+Escape show no lock to the viewer (0 days)
== construct / none (6)
  2026-11-20-p2r1-demo.md: Esc closes the key list and steps out of the pane too (0 days)
  2026-11-20-p2r1-explorer.md: Step into a face needs a click and Return; scroll does nothing (0 days)
  2026-11-20-p2r1-explorer.md: The chat panel stays open and covers the key list (0 days)
  2026-11-20-p2r1-max.md: Typed text in a pane goes to the walk keys, not the window (0 days)
  2026-11-20-p2r1-max.md: Super+D, Super+Q and Super+Tab reach a client as text (0 days)
  2026-11-20-p2r1-max.md: The helper panel opens only from the top bar (0 days)
== mc / contract M1 (1)
  2026-11-20-p2r1-cook.md: Arrow keys and Escape do nothing on the terminal page: no way back by remote (0 days)
== mc / contract T1 (1)
  2026-11-20-p2r1-sofa.md: A click on the launcher row Foot opens the helper chat (0 days)
== mc / contract T2 (2)
  2026-11-20-p2r1-cook.md: Super+F1 shows no key list, yet the sheet counts as open (0 days)
  2026-11-20-p2r1-sofa.md: Super+F1 shows nothing; mc-keys left open (0 days)
== mc / contract T3 (1)
  2026-11-20-p2r1-nurse.md: Focus is with nobody after the helper chat closes (0 days)
== mc / contract T4:volume (1)
  2026-11-20-p2r1-sofa.md: No way to change the volume found: no entry in the launcher, no key, no door (0 days)
== mc / contract T6 (1)
  2026-11-20-p2r1-nurse.md: A program that does not exist starts silently: no notice (0 days)
== mc / none (20)
  2026-11-20-p2r1-cook.md: Agenda is empty, nothing to open (0 days)
  2026-11-20-p2r1-cook.md: ux: page 5 is a root shell and its footer promises a way back (0 days)
  2026-11-20-p2r1-nurse.md: Agenda is empty, nothing to open (0 days)
  2026-11-20-p2r1-nurse.md: ux: helper answers only "Hello" to any request, no confirmation (0 days)
  2026-11-20-p2r1-sofa.md: ux: Escape closes the chat before it turns a page; chat opens with an old exchange (0 days)
== tiling / contract T1 (1)
  2026-11-20-p2r1-beginner.md: Clicking Foot in the Apps menu opens the helper chat instead of a terminal (0 days)
== tiling / contract T4:power (1)
  2026-11-20-p2r1-beginner.md: Key list stays open after the power menu is cancelled (0 days)
== tiling / contract T6 (1)
  2026-11-20-p2r1-beginner.md: Unknown program in the Apps box gives "Nothing found", no start and no notice (0 days)
== tiling / none (11)
  2026-11-20-p2r1-kemal.md: Super+Right switches workspace instead of moving focus (0 days)
  2026-11-20-p2r1-kemal.md: Super+q does nothing visible; the terminal closed only with Ctrl+d (0 days)
  2026-11-20-p2r1-kemal.md: Launcher lists "Chat with a helper" first and the helper panel covers the right tile (0 days)
  2026-11-20-p2r1-ricer.md: The key list offers a reload key but there is no settings file to edit (0 days)
  2026-11-20-p2r1-ricer.md: Yellow square at the top-left corner after switching workspace (0 days)
== xp / contract T1 (2)
  2026-11-20-p2r1-erika.md: Notepad from Start never opens a window (0 days)
  2026-11-20-p2r1-murat.md: Notepad never opens (second hit) (0 days)
== xp / contract T2 (1)
  2026-11-20-p2r1-murat.md: Help key list fails the key-sheet words check (0 days)
== xp / contract T3 (1)
  2026-11-20-p2r1-erika.md: Helper chat opens but "hello" gets no answer; state stays no_helper (0 days)
== xp / contract T4:clock (1)
  2026-11-20-p2r1-erika.md: Clock on the taskbar is tiny, and the checker finds no clock node (0 days)
== xp / contract T4:lock (1)
  2026-11-20-p2r1-pixel.md: Lock does nothing: Start > Lock and Super+Shift+Escape (0 days)
== xp / contract T4:volume (1)
  2026-11-20-p2r1-murat.md: Clicking the speaker icon shows no volume control (0 days)
== xp / contract T6 (1)
  2026-11-20-p2r1-murat.md: Run box says Nothing found for an unknown program (0 days)
== xp / contract X2 (1)
  2026-11-20-p2r1-pixel.md: The small window does not follow a drag (0 days)
== xp / none (12)
  2026-11-20-p2r1-erika.md: Power dialog Cancel is small and low in the dialog (0 days)
  2026-11-20-p2r1-erika.md: Starting Notepad balloon comes back after the power dialog is cancelled (0 days)
  2026-11-20-p2r1-murat.md: Help lists key "Vol-" that cannot be pressed, and screenshot not tried (0 days)
  2026-11-20-p2r1-pixel.md: Super+D hides the window and it does not come back (0 days)
  2026-11-20-p2r1-pixel.md: Super+F1 and Super+Alt+1 show nothing (0 days)
  2026-11-20-p2r1-pixel.md: Help lists shortcuts that need scripts or programs that are not there (0 days)
waiting 75, overdue 27, missing ux 0
OVERDUE
FORMAT OK
```
`triage.py --format`: FORMAT OK. The 27 overdue are the older files; every p2r1 finding was 0 days old.

### Counts
| Look | Findings | New row | of which verify | Duplicate | Rejected (harness, env) |
|---|---|---|---|---|---|
| xp | 15 | 4 (xp-31, 32, 33 entered by the PM at r1 review and already fixed; xp-34) | 1 | 4 | 7 (6, 1) |
| tiling | 8 | 3 (ti-27, 28, 29) | 2 | 3 | 2 (2, 0) |
| mc | 12 | 4 (mc-42 to 45) | 1 | 5 | 3 (3, 0) |
| construct | 13 | 3 (co-22, 23, 24) | 3 | 4 | 6 (6, 0) |
| shared | row sh-29 made as the anchor of lane launcher-click (3 hits, counted as duplicates of the in-flight fault) | | | | |
Total 48 = 14 new + 16 duplicate + 18 rejected.

### xp (Erika, Murat, pixel-watcher; round on 5755b578)
New BLOCK: none.
New MAJOR (check named):
| Row | Finding | Check | State |
|---|---|---|---|
| xp-32 | Murat: speaker click shows no volume control | contract T4:volume (red at r1) | fixed 81a29fe8, act: click opens the popup |
| xp-33 | pixel: Super+F1 shows nothing | contract T2 (red at r1) | fixed 5da2712c |
| xp-31 | pixel: Super+D hides windows, no way back | none at r1; act now guards (second Super+d, task button) | fixed 5ac45053 |
Raised: xp-11 MINOR to MAJOR (Murat T2: the Help key list lacks the shared words; contract T2 red on a shared-bar task, workaround Super+F1 sheet); this is a duplicate row, not new.
Duplicates:
| Finding | Row |
|---|---|
| Erika: Cancel small and low in the Turn Off dialog | xp-9 (+1) |
| Erika: Starting Notepad balloon comes back | xp-6 (+1; the balloon waits 30 s) |
| Murat: Help key list fails the key-sheet words | xp-11 (+1, raised) |
| Murat: Run box "Nothing found" | xp-2 (+1, confirms the fix) |
Verify: xp-34 MINOR (tiny tray clock; the red T4:clock contract is harness: it searches scene nodes named *clock*, the xp tray clock is none).
Rejected:
| Finding | Reason |
|---|---|
| Erika: Notepad never opens; Murat: second hit | harness: the persona session has no GUI editor stub (backlog H-persona-env, PM review r1; persona-env 370495ad is not in 5755b578); xp-25 stays the product row |
| Erika: helper chat gets no answer, state no_helper | harness: the fake voice agent answers from the seeded file only (H-persona-env) |
| Murat: key "Vol-" unknown | harness: act.py has no alias for a Help key cap |
| pixel: small window does not follow a drag | harness: the chora-probe window has no title bar (H-persona-env, X2 needs an xp window) |
| pixel: Lock does nothing | harness: shared scripts not installed in the persona environment (choices.rs:248), no loginctl stand-in call |
| pixel: Help lists shortcuts that need scripts or programs | env: no file manager, shared scripts missing |
Stop rule: new MAJORs backed by a check exist (xp-32, xp-33, and xp-11 raised): ANOTHER ROUND. Next: R2 on the fixed sha after H-persona-env (the PM plan).

### tiling (Kemal, beginner, ricer; round on b97369d1)
New BLOCK: none. New MAJOR: none.
Duplicates:
| Finding | Row |
|---|---|
| beginner: a click on Foot in Apps opens the helper chat | ti-1 (+1, BLOCK stays open: fuzzel >= 1.11 pin) and sh-29 (lane launcher-click) |
| beginner: unknown program, "Nothing found" | ti-2 (+1, confirms the fix; contract T6 expects a notice, harness mismatch to fix in check.py) |
| Kemal: Super+Right switches workspace | ti-12 (+1) |
Verify: ti-27 (key list stays open after the power menu is cancelled; MINOR), ti-29 (helper first in the launcher, panel hides the right tile; MINOR). Open MINOR: ti-28 (Super+q closes nothing: tiling.yaml:92 binds close to Super+Shift+c).
Rejected:
| Finding | Reason |
|---|---|
| ricer: no settings file to find from the key list | harness: the persona session has an empty XDG_CONFIG_HOME, so L2 could not start |
| ricer: yellow square after the workspace switch | harness: the 12 px yellow pointer marker of the persona session (co-14, fb-swarm4-c4); also in cook shot 011 and nurse shot 004 |
Not measured in this round (harness): L1 (vacuous), L2 (vacuous), T5, T7; the verdict below rests on T1, T2, T3, T4:power, T6, L3 only.
Stop rule: no new BLOCK or MAJOR: STOP by the rule (ti-1 and sh-29 stay open through their own lanes; L2, T5, T7 stay unmeasured, the PM may fold them into R2).

### mc (cook, sofa, nurse; round on b97369d1)
New BLOCK (check named):
| Row | Finding | Check | Hits |
|---|---|---|---|
| mc-42 | A window page (Foot) swallows arrows, Escape and Super+F1; a remote without Super cannot leave it; the column doors are not on screen | contract T4:volume red (sofa) and T2 red; cell to build `mc-paths terminal-back` | 3 (cook M1, cook ux page 5, sofa T4:volume) |
New MAJOR (check named):
| Row | Finding | Check | Hits |
|---|---|---|---|
| mc-43 | Super+F1 on a Foot page counts as open and draws nothing | contract T2 red; cell to build: gate-doors keys-key on a window page | 2 (cook, sofa) |
| mc-44 | A program that does not exist starts silently | contract T6 red; cell to build `mc launcher-none` | 1 (nurse); p2-plan 2 reads a shared task as BLOCK, taken as MAJOR (launcher lists programs, no window lost): PM may raise |
Duplicates:
| Finding | Row |
|---|---|
| cook: ux page 5 root shell, no way back | mc-42 (+1) |
| sofa: no way to change the volume | mc-42 (+1; the Volume door exists in the column, gate-doors volume-door green, sh-3) |
| sofa: Super+F1 shows nothing | mc-43 (+1) |
| sofa: a click on the launcher row Foot opens the chat | sh-29 (lane launcher-click; fuzzel, ti-1) |
| sofa: Escape closes the chat before it turns a page | sh-15 (+1; the old exchange in the chat is the seeded transcript, harness) |
Verify: mc-45 MAJOR? (focus None after the chat closes; the end captures show the chat open, read nurse 011 first).
Rejected (harness): cook and nurse "Agenda is empty" (no agenda item seeded, M2 untestable); nurse "helper answers only Hello" (FAKE_VOICE_ANSWER=Hello).
Not run: nurse T7 (xp -> mc): the xp+mc session never quiet for check.py (NotQuiet mc:write:mc-s-agenda-panels-today): harness or a real mc write loop, to read; M2 untested in all three.
Stop rule: new BLOCK and MAJORs backed by checks (mc-42, mc-43, mc-44): ANOTHER ROUND, after a FIX of mc-42, 43, 44 (red cells first) and seeded agenda item.

### construct (explorer, Max, demo; round on b97369d1)
New BLOCK: none. New MAJOR: none.
Duplicates:
| Finding | Row |
|---|---|
| explorer: Programs > Foot opens the helper chat | sh-29 (lane launcher-click) |
| explorer: chat panel covers the key list | sh-15 (+1) |
| demo: Esc closes the key list and steps out of the pane | sh-15 (+1) |
| Max: Super+d, q, Tab reach foot as text | sh-24 (+1; lanes stray-grab; T5 unmeasured, probe died) |
Verify (all MINOR): co-22 (step in by mouse needs a click and Return), co-23 (typing in a pane goes to the walk keys), co-24 (helper only from the top bar, key not found).
Rejected (harness):
| Finding | Reason |
|---|---|
| explorer and demo: I7 on every output (136 and 100 lines) | harness: chora-xp did not connect in any session, so the faces are empty (lane p2-harness item 1; r1b shows whether I7 is gone) |
| explorer: cube row leads to "Starting XP" then a failure; demo: every face "did not start" | harness: the same xp start timeout |
| demo: no pull gesture found (C2) | harness: C2 compares with the start snapshot ("new s1"), the pane sat on a failed xp face |
| demo: Lock click and chord show no lock | harness: the swaylock stand-in draws nothing; the contract passed; the 02:03 to 02:05 clock is two real minutes |
Stop rule: no new BLOCK or MAJOR: STOP by the rule, but the round measured little (no face ran): the p2-harness r1b data decides whether C1, C2, I7 and T5 need a round.

### Stop-rule verdict
| Look | New BLOCK/MAJOR with check | Verdict |
|---|---|---|
| xp | xp-32, xp-33 (fixed), xp-11 raised | another round (R2 after H-persona-env) |
| tiling | none | stop |
| mc | mc-42 BLOCK, mc-43, mc-44 | another round (after the fix) |
| construct | none | stop (r1b pending) |

### Ledger edits
xp.md: xp-2, 6, 9, 11 (MAJOR), 31, 32, 33 rewritten, xp-34 added. tiling.md: ti-1, 2, 12 rewritten, ti-27 to 29 added. mc.md: mc-42 to 45 added. construct.md: co-22 to 24 added. shared.md: sh-15, sh-24 rewritten, sh-29 added. shared-doors.md: untouched.
Open BLOCK after T1: co-18 closed, mc-32 closed, ti-1, mc-42, sh-29.
