# P2 plan: the simulated user test (p2-plan r1, 2026-11-19, opus, read only)

Input: reports/plan-after-foundation.md (Re-plan 11-17 14:24), design/foundation.md, design/rc.md,
vision-card.md, reports/persona-3.md, reports/swarm-2/, feedback/README.md, findings/README.md,
reports/proposal-glitch-gates.md, triage.py (at the root of claude/session-log, not tools/), and on
a9e7084d: tests/acts/, tests/persona/, tests/doors/lib.sh, tests/merge-gate.sh. `cargo check -p chora` green.

## 0. What exists today (the base the plan stands on)
- Act contracts: tests/acts/harness.py (`Session.act(..., expect=Expect(...))`), acts per look in
  tests/acts/looks/*.py, gate step `acts` (tests/gate-acts.sh). tests/acts/known-red.toml has no row: every act passes.
- Invariants I1-I10, I12-I14 (no I11) in tests/acts/invariants.py, run after every scripted act;
  planted.py plants a breach of each and must see it fail.
- Shared bar as cells: tests/doors/lib.sh `doors_cells` (keys, f1-free, launcher, helper, lock,
  power, screenshot, volume, clock, key and door each) in gate step `doors`; handover in
  tests/gate-handover-matrix.sh (12 pairs x paths, cell pass = owned, attached, visible, focused).
- Persona driver: tests/persona/session.sh (one flavor; grants `persona` only screen.capture and
  input.inject, session.sh:231), tests/persona/act.py (look, click, move, drag, type, key, scroll),
  the haiku brief in tests/persona/README.md.
- Findings path: feedback/DATE-RUN-PERSONA.md (feedback/README.md format), triage.py counts `new`;
  ledgers findings/<look>.md with BLOCK/MAJOR by the RC bar (findings/README.md). Open today
  (grep `| BLOCK |.*| open` and MAJOR): 2 BLOCK (co-18, mc-32), 23 MAJOR.

The gap: a persona run and the invariants never meet. The persona session has no checker, and a
persona's "reached" is an opinion (persona-3, swarm-2 judged by reading captures). P2 joins them.

## 1. Personas and tasks per look
Shared tasks (vision-card.md "Shared RC bar"). A task PASSES when all three hold:
(a) its end-state contract is green (`check.py end TASK`, section 4 item 1; read from state.get,
scene.boxes and the stand-in logs, not pixels); (b) no invariant breach during the run outside
known-red.toml (breaches.jsonl of the run); (c) the persona says reached within its cap.
(a) and (b) green but (c) no = a UX finding (door hard to find). (a) red but (c) yes = triage reads
the capture. Contracts reuse harness.Expect; the cell named is the scripted twin that must stay green.

| Task | Contract at the end of the run | Scripted twin |
|---|---|---|
| T1 start a program | a new toplevel, owner = the look, focused; `open` empty (launcher shut) | doors launcher-key/-door; acts window_acts |
| T2 key list | the sheet id was in `open`, then `open` empty; keysheet words present (common.keysheet_breaches) | doors keys-key/-door; keysheet_acts |
| T3 ask the helper | helper idle -> working -> idle with an answer entry; I8; focus back on the window (I3) | doors helper-key/-door; helper_acts; gate-chat-keys |
| T4 system | volume: a wpctl stand-in call; clock: clock-door node present; screenshot: grim stand-in call; lock: `loginctl lock-session` in PERSONA_CALLS; power menu opened and cancelled: `open` empty, no systemctl call | doors lock/power/screenshot/volume/clock cells |
| T5 no stray keys | the probe window (`chora-probe window --report`) got no key of a Super chord the persona pressed | doors f1-free; gate-injected-presses |
| T6 honest failure | a program that cannot start gives a notice that I9 sees drawn; no helper -> door says No helper | doors helper-none; I9 |
| T7 no lost window | after a handover the toplevel set is the same, every window owned (I1), inside an output (I2) | gate-handover-matrix; construct.py pull_and_remove |

| Look | Persona (input) | Tasks |
|---|---|---|
| xp | Erika, 74, mouse only, double-clicks when unsure | T1 (Notepad from Start), X1 find the window again after minimize (contract: shown, focused; I3), T4 clock + power cancel, T3 |
| xp | Murat, 52, mouse only, office habits | T1, T2 by Help and Support, T4 volume + screenshot, T6 |
| xp | pixel-watcher, mouse and keys | X2 Start menu then drag a window (contract: menu shut, xp drag_acts), T2, T5, T4 lock |
| tiling | Kemal, keyboard first | T1 by Super+Space, L1 two terminals tile, move focus, close one (contract: 2 then 1 toplevel, focus moves, I2; gate-tiling-focus), T5, T3 by Super+a |
| tiling | beginner, mouse first, never used tiling | T1 by a visible door, T2 by a door, T4 power cancel, T6 (must not be stranded: every task by mouse) |
| tiling | ricer, keys and an editor | L2 change a gap in the config and reload (cell gate-tiling-reload), L3 switch workspace and back (window shown again, I3), T2, T7 tiling -> xp (second look, item 2) |
| mc | cook, remote keys only (arrows, Return, Escape) | M1 turn the page (mc state page changes; gate-mc-paths, ledger mc-32), M2 open an agenda item and back (gate-mc-sofa), T2 by Super+F1, T4 clock |
| mc | sofa, low-tech, mouse and remote at a distance | M1, T1, T3, T4 volume |
| mc | nurse, hands busy, short key presses | M2, T6, T7 xp -> mc (gate-mc-handover) |
| construct | explorer, mouse first | C1 open the cube, step in, step out (construct.py cube_acts; I13), T1, T2 |
| construct | Max, ricer, keys and mouse | C2 pull a window out of a face and back (T7 contract; gate-construct-pull; ledger co-18), C3 a window as a pane (gate-construct-windows), T5, T3 |
| construct | demo watcher, follows 3 spoken-style steps | C1, C2, T4 lock |

Voice personas (Lena, en/de) are out: act.py has no speech and fake-voice.py only replays events;
the voice path stays with the scripted gates (tests/voice-events.sh, fake-voice) until round 5.

## 2. How a haiku persona drives a session
- Tool: only tests/persona/act.py on a session from tests/persona/session.sh, with the brief of
  tests/persona/README.md plus 3-5 goals in a user's words. Not choractl (admin socket: it would bypass
  the screen; session.sh grants persona two methods only), not tests/nested.sh (a sway e2e, not a
  persona harness), not the act scripts (scripted contracts; the checker reuses their invariants).
- Driver: one sonnet lane per look (rules.md 5). It snapshots the binaries once per round (one sha,
  as swarm-2 did with CHORA_BIN), starts at most 2 sessions at once (4 CPUs, `nproc`; swarm-2
  README), seeds the helper transcript (session.sh:103 writes `[]` to voice/entries.json, so atlas
  never answers: erika6's "no answer arrived" may be this, check before ledger), spawns the haiku
  persona (Agent, model haiku), then runs `check.py end TASK...` and writes the run file.
- The persona may not: read any file but its PNGs, run another command, search, retry a goal a 4th
  time, exceed 45 actions or 20 looks (swarm-2 budget rule), be coached. It never sees breaches.
  The driver may not fix code or touch the session between actions (data only, as swarm-2).
- Findings: feedback/DATE-p2rN-PERSONA.md in feedback/README.md form plus two lines per finding:
  `task: T1..T7|X1..|none` and `check: I<n> | contract T<n> | cell <name> | none`. Each breach line and
  each red contract becomes a kind `issue` finding by the driver, with the capture of that action.
- Severity: the persona's high/medium/low is a guess. Triage maps by findings/README.md: BLOCK =
  check red on a shared-bar task or main path, or I1/T7 lost window; MAJOR = check red with a
  workaround; `check: none` enters the ledger as `verify` until a cell or one capture read confirms
  it (rules.md 7; proposal-glitch-gates.md 4: an eye-only finding becomes an invariant or cell before its fix).
- Dedupe: same (flavor, check) = one row with a hit count; else the title against open ledger rows;
  env noise (missing program, locale) is `rejected: env` (swarm-2: 9 of 31 sections were env or harness).

## 3. Lanes Sat-Mon (percent of a week; model per rules.md 9)
| # | When | Lane | Model, effort | % | Done when |
|---|---|---|---|---|---|
| 0 | Sat 04:00 (Fri evening if a slot frees: test code only) | PW: items 1, 2, 4 | sonnet high | 1.5 | smoke.sh green on 4 looks; a planted breach shows in breaches.jsonl; T1-T7 contract selftest in tests/test_acts.py; merge gate green |
| 0b | Sat 04:00, parallel | TRI: item 3 on claude/session-log | sonnet low | 0.3 | triage.py groups by (flavor, check), flags high without check |
| 1 | Sat after PW merges | R1-xp, R1-tiling, R1-mc, R1-construct (3 personas each) | sonnet medium + haiku | 4 x 0.8 = 3.2 | 4 run files, triage.py OK on format |
| 2 | Sat late | T1 triage: dedupe, ledgers rewritten, each BLOCK/MAJOR with its check or a red row in known-red.toml | sonnet high | 1.2 | ledgers rewritten; every new BLOCK/MAJOR names a check |
| 3 | Sun | FIX per look (4) + FIX-shared: BLOCK then MAJOR, red test first (rules.md 8) | sonnet high | 4 x 2 + 1.5 = 9.5 | each fix turns its check green through merge-gate.sh |
| 4 | Sun-Mon | reviews: opus for core and kit fixes, sonnet for ports (rules.md 7) | opus high / sonnet medium | 2.5 | MERGE-OK or FIX per branch |
| 5 | Mon | R2 per look: same tasks on the fixed sha + one fresh persona not in R1 | sonnet medium + haiku | 3.2 | run files |
| 6 | Mon late | T2 triage | sonnet high | 0.8 | ledgers rewritten |
| 7 | Tue (only looks that need it) | R3 + its fixes | as above | 2.0 | stop rule |
Total 24.2 % (cap 25 %). Order: 0 and 0b, then 1 (all 4 in parallel: one lane per look), 2, 3 and 4, 5, 6, 7.
Stop rule: a look's rounds stop when its triage finds no new BLOCK or MAJOR (new = no duplicate of
an open or verify row, and backed by a check); hard cap 3 rounds; the whole P2 stops at 25 %.
P2 is met (plan-after-foundation P2) when every task of section 1 passes (a)+(b) on one sha, and no
BLOCK or MAJOR is open on a path a task uses.

## 4. What must exist first (4 items)
1. tests/persona/check.py, about 180 lines + selftest (lane PW). Attaches to a persona session as a
   second client (`persona-check`, granted methods.call and screen.capture by session.sh), takes a
   snapshot at start, and after each act.py action (act.py runs it when PERSONA_CHECK=1, output only to
   $PERSONA_DIR/breaches.jsonl, never to the persona) runs the invariants of tests/acts/invariants.py.
   Needs `Session.attach(env_file)` split out of harness.Session.setup (harness.py:400). Skips I4 (it
   needs declared opens; the task contract carries `open` instead) and treats no disconnect as declared
   (I14). `check.py end TASK...` checks the T/X/L/M/C contracts of section 1 as Expect objects.
2. session.sh, about 40 lines (PW): the stand-ins of tests/doors/lib.sh doors_fakes for wpctl,
   systemctl and grim, logging to PERSONA_CALLS. Today only loginctl and notify-send are faked
   (session.sh:91-99) while xp Stand By runs `systemctl suspend` (plugins/chora-xp/src/session.rs:62)
   and tiling runs systemctl (plugins/chora-tiling/src/overlay.rs:52): on a dev machine a persona
   click suspends the host, and wpctl is absent here (`command -v wpctl`), so T4 volume is untestable.
   Plus `FLAVOR+FLAVOR` for a second look (T7 off construct; persona-3 needed a scratch harness).
3. Feedback format and triage.py, about 40 lines (TRI): `task:` and `check:` lines in feedback/README.md;
   triage.py prints `new` findings grouped by (flavor, check) and marks severity high with `check: none`.
4. tests/persona/round.sh, about 60 lines (PW): reports/swarm-2/swarm.sh moved into the repo without its
   hard-coded scratchpad paths: binary snapshot, 2 slots, PERSONA_KEEP, PERSONA_CHECK=1.

## 5. Risks (5) and out of scope
1. The tree still moves: state.md burn-down has F2 partly, F5 open, F7 next week. Findings on a sha that a
   refactor merge then changes are stale. Mitigation: one frozen sha per round (item 4); fixes rebase.
2. False breaches from the checker on animating looks (construct turns, tiling motion): I3 and I13 read
   "once quiet". Mitigation: check.py waits quiet as harness.wait_quiet does; a breach seen once and not
   on replay of the action log is `verify`, not BLOCK.
3. Harness and env noise eat triage (swarm-2: 9 of 31; persona-3 caveats: injected flag, empty
   transcript, missing HUD dir). Mitigation: tasks use only programs present here (foot, chora-probe,
   fuzzel; no LibreOffice or thunar), seeded helper transcript, env rows rejected by name.
4. The injected-press flag (--test-injected-press-is-real) can hide real-input bugs; gate-injected-presses
   covers the refusal side only. Accept for P2; the owner's daily driving is the real-input check.
5. Fix budget: 25 open BLOCK/MAJOR today plus new finds may exceed 9.5 % of fixes. Mitigation: R1 first
   confirms or drops the `verify` rows; rules.md 6 keeps MINOR out; leftovers go to the known issues.
Out of scope: new features or new looks' behaviour; voice personas in en/de, two screens, a several-hour
leak session, the battle script (rc.md round 5); the new tour and the video checker (after P2,
plan-after-foundation Re-plan); MINOR and COSMETIC fixes; xp taskbar and tray redesign (X10-X12);
opus vision on frames (only in the new tour).

## PM review r1 (2026-11-19 22:45 UTC)
Accepted as the P2 plan. Checks: 138 lines (<= 150); every task names a contract and a scripted twin; lanes 24.2 % <= 25 %; stop rule and P2-met line present; voice out of scope named. The systemctl stand-in (item 2) is a safety fix: a persona click must never suspend a host. PW starts now (slot free, test code only, budget underspent), TRI when a slot frees.

## PM ruling 11-20 06:00 UTC: M2
An agenda row is display content of panel.set with no action behind it; it does not open in this RC. M2 leaves the plan for mc (cook keeps M1, T2, T4; nurse keeps T6, T7). Backlog AGENDA-OPEN (post-RC). mc round 3 after mc-47, mc-48, mc-49 are routed (mc-fix-r1).
