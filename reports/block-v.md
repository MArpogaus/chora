# block-v

## r1 (2026-11-19, sonnet, claude/block-v from a9e7084d, head cbdbc34f)

Verdict: co-18, co-16 and mc-32 are gone on the head. No product fix; three guards, all green, no red commit needed.

- co-18 (BLOCK): act "a drag on a face turns the cube" now also checks `stood_back_whole`, plus the new acts "a drag to the right on the stood back cube turns it back" and "a second drag to the left turns it on again" (tests/acts/looks/construct.py). Real input path (`input.inject` pointer and button events). After each drag: cube front moves 0 -> 1 -> 0 -> 1, mode stays navigate, the probe window stays on its face with xp (I1, `windows_on_faces`), the chips turn-left, step-in and turn-right keep their key caps.
- co-16 (MAJOR): the same check reads the box of the front face (`construct-cube-face-N`) and fails when it leaves the 1280x800 screen. It lies at 416,255 375x290 after every drag. A scratch probe also showed that a drag that starts outside the cube (x=80 or x=1200) does not turn it; a drag from inside does, also a short one (640 -> 600). The tour v4 shot likely started outside the cube. Not treated as a bug (a drag on the empty floor).
- mc-32 (BLOCK): act "Right of a remote turns the page in dark mc" and its four siblings (tests/acts/looks/mc.py): `settings.set theme dark`, then `key` events (pressed, released, no modifier) through `input.inject`; the session runs with `--test-injected-press-is-real`, so the press counts as a person's. Right, Right, Left turn agenda -> morning -> agents -> morning; Super+Home goes to agenda. A plain Home is no mc key (mc.yaml binds Super+Home and the file says Right, Left and Escape alone turn a page), so the Home half of the row is not a product fault. Product vs injection: the same press turns the page here, so the tour's miss came from the harness of that run.
- Ledgers: the three rows are `closed: <act name>, cbdbc34f`.

Gate (`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,mc-paths`): all seven PASS. `construct-pull` is no name of `--list` (the list has `pull`), so it was dropped and did not run. origin/claude/autonomous-dev-session-3w-i33pg7 was already merged into the base (merge: already up to date).
Not covered: an act with a person's real pointer on hardware (risk 4 of p2-plan).

## PM review r1 (23:15)

Diff a9e7084d..cbdbc34f, 2 files, tests only (tests/acts/looks/construct.py, mc.py); gate cells pre-commit,
ci-check, build, test, clippy, acts, mc-paths PASS (lane). construct-pull is not a gate cell (brief error by
the PM; the cell is `pull`, which the merge gate runs).
- co-16, co-18: the guard stood_back_whole checks the front face on screen, the chips' key caps and
  windows_on_faces after three drags both ways. Accepted: closed with the act.
- rw-1 MAJOR (mc-32 Home half): mc-32 named Right and Home. Right and Left hold. "Plain Home is no mc key"
  is the fault, not its absence: a remote has no Super, so the remote path cannot reach the first page
  but by repeated Left. Reopened as mc-38 (findings/mc.md); fix in r2.
- rw-2 MINOR tests/acts/looks/mc.py remote_acts: sets theme dark and never sets it back; helper_acts then
  run dark. Fix: restore the theme at the end of remote_acts.
Verdict computed: rw-1 MAJOR open -> FIX. r2: briefs/block-v-r2.md.
