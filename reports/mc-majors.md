# mc-majors

## r1

Lane: cloud, branch claude/mc-majors from a61e8eaa, head 92ecd30a (pushed). Setup and
`cargo check --workspace` passed. All five MAJORs of journey-1 (mc-33 to mc-37) are gone on the
head: the tour v4 stills are from 11-16 09:33, the fixes and their red cells landed the same
morning (10:19 to 11:46) and the ledger rows were never closed. No product code changed in this
round; I added three guard cells and closed the rows.

### Reproduction on the head (a61e8eaa, 1280x800 and 1280x720, dark and light)

| id | v4 symptom | on the head | red | fix | cell |
|----|-----------|-------------|-----|-----|------|
| mc-33 | chat door by mouse opens no chat while a helper row shows | click on the door opens `hud-chat` (probe and cell) | none: guard written green | none found; gone before dda31e6a | `door` of tests/gate-mc-sofa.sh |
| mc-34 | voice HUD text about 11 px, grey | answer line 19 px (`size: 19` in hud.yaml), ink (217,212,201) on (28,28,28), 11.5:1; light 13.8:1 | e90abaaf | 0fbb751e | `answer`, now with the ink (62e8ca12) |
| mc-35 | critical alarm is a cut footer line, column breaks | alarm is a card of chora-notify above the footer, no NOTIFICATIONS or empty NOTES head, Chat door above the KEYS rule | 3337875f | 3c19f217; since f200c497 chora-notify draws the card | `alarm`, new (62e8ca12); the old `notices` cell went with f200c497 |
| mc-36 | No helper pill while answering, overlaps the status word | Mute stands beside the state word (red 34587eeb, fix 1eb04373). The pill word: the host says Speaking when the voice agent speaks and never No helper while a helper works | 34587eeb | 1eb04373 | `status`; `pill`, new (62e8ca12) |
| mc-37 | Super+n hides the column, the page does not widen | the lead text widens by more than 100 px and narrows back | d4938d1c | 309ae998 | `wide` |

The word No helper in the v4 and v7 stills is the tour script: demo/tour/v8_mc.py answers with
`hud.say` and leaves the voice state at `listening`, so the host rightly says no helper. A real
voice agent emits `voice.state speaking` (cell `pill`). A one-line change in the tour script
(emit `voice.state speaking` with the answer) would drop it from the next tour; I did not touch
demo/.

### Gate

Merged origin/claude/autonomous-dev-session-3w-i33pg7 (30727c82), then
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,mc-paths,pixels,mc-sofa`:
all PASS (mc-sofa added because I changed it). Light and dark of the sofa gate passed alone too.

### Seen on the way, not in scope

1. NEW, MAJOR for the sofa, host plugin: a long answer (150 characters) of the voice HUD wraps
   to three lines at x 479 to 1110 and covers the date, the first rows of the column and the
   start of the page title row (dark, 1280x800, `voice.say` with 150 characters; capture in the
   lane scratchpad only). The `answer` cell uses a 27-character line, so it does not see this.
   The fix is in the hud layout (`near: state` grows left and down over the work): max width
   or a place of its own. Proposed: mc-38.
2. `tests/gate-mc-column.sh` fails here on the head, before any of my changes: `ctl wait
   owner:notify` times out because chora-notify shows no node until a notice is up (state.get
   answers, `shown` is empty). The step list for this round does not run that gate, so it is
   not in the merge gate result; the sofa gate waits by `state.get` instead.
3. The card of chora-notify in the mc slot has about 11 px text; it is a card, but the same
   sofa distance question as mc-34 applies to it.
