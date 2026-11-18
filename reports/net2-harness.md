# net2 r1 harness (F3): tests/acts, I1 I3 I4 I7 I8 I14, known-red, gate step acts

Branch claude/net2-harness on f31ff880 (3 commits, pushed): 569ecc4e test (red: no harness),
ead29e7c feat harness, 4052cd82 ci gate step. Merge gate `tests/merge-gate.sh --jobs 3` on 4052cd82: every step PASS (acts 20 s, 26 pass, 6 known red, 0 blocking), GATE_EXIT=0, log /tmp/claude-0/gate/net2h.log, run dir /tmp/merge-gate.YS8701.

## Done
- `tests/acts/harness.py`: `Session(look, ...)` (headless session per look, own `acts` and
  `acts-events` connections, grants), `act(name, cues, expect, opens, closes, disconnects)`,
  quiet = every `busy` of every look empty (timeout 10 s fails the act, `NotQuiet`), cues `key`,
  `text`, `click`, `program`, `wait`; `Expect(new_window, gone, focus, open, helper)`; a breach keeps
  states before/after and a still in the run folder. `judge` + `known-red.toml`: a listed failure
  does not block, a listed pass and an unlisted failure do, a row needs finding and slice.
- `tests/acts/invariants.py`: I1, I3, I4, I7, I8, I14, pure functions of two snapshots.
- Selftests: `tests/test_acts.py` 45 cells without a session (red: ModuleNotFoundError at 569ecc4e;
  stubbing each invariant to pass turns 6/5/3/2/2/3 cells red); `tests/acts/planted.py` live in mc:
  black rect (I7), keyboard on a harness node (I3), two helper surfaces (I8), undeclared vs
  declared disconnect (I14), dropped window (I1), undeclared/declared open (I4), helper working
  after `voice.state` idle (I8), endless busy (quiet). All planted cells red, sound cells green.
- Acts on the head (`tests/acts/looks/LOOK.py`, a window opens and closes): mc all green;
  xp, tiling, construct: I1 and I3 red because they answer no `state.get` (6 known-red rows,
  slice NET2 adapters). Gate step `acts` (`tests/gate-acts.sh`, about 20 s), CONTRIBUTING.md line.

## Notes for the adapters lane and the PM
- Door acts go into `tests/acts/looks/LOOK.py` (`run(s)`); remove the 6 rows as adapters land
  (a listed pass fails the gate).
- construct: a new pane does not take the keyboard (the walk keeps `construct-look`), so its
  window act does not assert focus; confirm that this is intended.
- The core closes a connection without `hello` after 5 s: an event watch needs a name.
- I1/I4/I8a breaches are planted in the read `state.get` answer (no look misreports on purpose);
  I3, I7, I8b, I14 are broken in the session itself.
- I8 helper surfaces: id `hud-chat`, meta `helper: surface`, app id `chora-chat`; a look's own
  helper surface should carry that meta (KIT7).

## Backlog (refactor proposals)
- ruff runs only on services/voice; tests/*.py (also existing ones) fail the D rules of ruff.toml.
- `scene.boxes` (NET3) should replace the `scene.query` rects read here.
- quiet still needs a 0.4 s settle after the cues before the first read (an animation starts
  after its key event); a `busy` set synchronously by the look would remove it.

## r2 (fix round): head b5a8625b, gate GATE_EXIT=0 (log /tmp/claude-0/gate/net2h2.log, run dir /tmp/merge-gate.r5jD8n; acts 30 s)
- Commits: 44da455c test (10 cells red), 36552b92 fix, b5a8625b drop a wrong known-red row.
- 1 Session.__enter__ is exception-safe (setup in `setup()`, any failure runs `__exit__`); cell: forced failure leaves no process, no temp dir.
- 2 Missing keys are breaches "missing key K": I1 windows, I3 focus, I4 open, I8 helper (after a voice event) and rects of helper surfaces, busy (NotQuiet). No state.get stays "no state". One cell each.
- 3 known-red `match` (every breach of the cell must contain it); 4 SIGTERM all, wait 8 s, then SIGKILL; run.py removes /tmp/acts-run-* on a pass and prints the path on a failure; 5 quiet = two empty reads 0.2 s apart; 6 docstring.
- Deviation: the brief's mc I4 row is not added; mc already answers `open`, the gate flagged the listed pass, row removed. 6 known red remain (xp, tiling, construct I1 and I3).
- Note: ruff still reports D-rule errors in tests/*.py (the earlier backlog item).
