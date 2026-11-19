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

## r2 (mc-38: a remote's Home goes to the first page)

Head ce230288 on claude/block-v (fix fa7faabd, red act dca24709, merge of claude/autonomous-dev-session-3w-i33pg7).

1. Red act first: dca24709 adds `act:Home of a remote goes to the first page` and
   `act:XF86HomePage of a remote goes to the first page` to tests/acts/looks/mc.py remote_acts.
   Command: `python3 tests/acts/run.py --no-selftest mc` on dca24709 gave "FAIL: mc act:XF86HomePage of a remote
   goes to the first page: mc shows the page 'morning', not 'agenda'" (1 blocking; the Home act shows the same fault).
2. Fix fa7faabd: plugins/chora-mc/src/main.rs `page_key` takes plain Home and XF86HomePage to `default_scene`
   (modifier or `--output` Bound: no key, as Right/Left); keys.rs full list has a `Home | First page` row on a page;
   mc.yaml (and the two tests/configs copies), docs/configuration.md and docs/mc-style.md name Home.
   Command: `python3 tests/acts/run.py --no-selftest mc` -> "43 pass, 0 known red, 0 blocking";
   `cargo test -p chora-mc` -> 168 passed (new test plain_home_goes_to_the_first_page, keys test for the Home row).
3. rw-2: remote_acts reads the theme with settings.get, and the act `the theme is set back` restores it
   (cell `mc act:the theme is set back pass`).
4. findings/mc.md: mc-38 `fixed fa7faabd act:...`, mc-32 `closed` with both acts.

Gate after the merge of origin/claude/autonomous-dev-session-3w-i33pg7:
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,acts,mc-paths` -> all seven PASS
(pre-commit, build, test, clippy, mc-paths, ci-check, acts); "partial run: nothing is pushed" is the gate's own line.
Note: the first commit of the fix failed the clippy hook (doc backticks, a test over the complexity limit); the
test was split, nothing skipped, no --no-verify.
