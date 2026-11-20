# co-p2-fix (construct RC shared bar: no stray keys, key list by keys)

Lane: cloud, branch claude/co-p2-fix from 79c8d2b5; base branch already merged (up to date). Step 0: unshallow, cargo check green.

## r1
Head 7b43b063 (pushed). Gate: `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,keys,doors,construct-hud,cube,out,pull,matrix`: all 12 cells PASS (run dir /tmp/merge-gate.DxAlvN). A first run had ci-check red by my own doc line (cut-check: swallow.rs named chora-tiling; fixed 7b43b063, whole gate rerun green).

### co-25 (MAJOR): no unbound Super chord reaches the focused client in a face
- Red: cf27cfcb (gate-construct-keys, probe pane with `--report`; probe report `key 125 key 32 key 125 key 16 key 125 key 15 key 125 key 2 key 30`: d, q, Tab, 1 all arrived), c50beafa (gate-construct-cube, chora-probe on the front face of the cube with xp; q, Tab, 1 reached it).
- Fix: b061930f. New plugins/chora-construct/src/swallow.rs (the tiling key list; Super+KEY, Super+Shift+KEY, Super+Ctrl+KEY; nothing for modifier Alt). `regrab` grabs the bound modifier chords and then the unbound ones with `Grab::Swallow` (does nothing; input.rs arm placed before the `_ => fly_out` fallback) in a pane, and on a cube once the swallow is due. Bound chords keep their action in both places (Terminal, launcher, keys, chat, lock, power, screenshot, volume, close, and the cube key through `Grab::Own`).
- Design point: xp, the manager of the faces, grabs its own chords (Super+d, Super+e ...) as it connects, and a chord the construct held first would be lost to it (gate line "Super+d of the xp of the faces" went red when the construct swallowed at cube open). So on a cube the swallow starts 1 s after the manager connected (`Host::swallow_at`, deadline in `cube_deadline`, `check_cube` regrabs). Unit tests: swallow.rs (chord set, Alt) and tests_construct.rs (held after connect, not before).
- Tests green: gate-construct-keys "co-25: Super+d, q, Tab and 1 reach no window in a pane, a plain a does"; gate-construct-cube "co-25: on the front face Super+q, Tab, 1, Shift+x and Ctrl+z reach no window, a plain a does" and "stood back, Super+Tab reaches no window". Plain `a` still reaches the probe. Gate cells: keys, cube, doors, out, pull, matrix, construct-hud PASS.
- Docs: docs/configuration.md, construct section after the fly_out paragraph (beside the tiling paragraph, cross-referenced).

### co-26 (MAJOR, verify): Super+F1 in a face opens the key list
- Reproduced red on the cube face before the fix (build of c50beafa with the stray-key asserts relaxed): "FAIL: Super+F1 on the front face shows no key list of the construct (co-26)". Cause: not the harness. Outside a pane the construct held none of its modifier chords while a cube was open, so Super+F1 reached xp (which opens Help and Support, as the demo persona saw). In a pane the chord was grabbed already.
- Fix: same commit b061930f (bound chords are grabbed in a pane and on a cube). Green: gate-construct-keys "co-26: Super+F1 in a pane shows the key list" (the pane is left for the list, as before), gate-construct-cube "co-26: Super+F1 on the front face shows the key list of the construct, and a second one hides it".
- Behaviour to know: on a face the key list does not take the keyboard (the window keeps it), so Escape reaches the window; a second Super+F1 closes the list. In a pane the list steps out of the pane first, as before.

### For the ledger / PM
1. xp: a chord that nobody sees still opens the Start menu on the release of Super (xp's own rule). Swallowed Super+q on a face therefore opens the Start menu (pre-existing: unswallowed it did too). The gates close it with Escape. A lane for xp could treat a chord that another client grabbed as a cancel.
2. chora-tiling keeps its own copy of the swallow key list (not touched here, as ordered); swallow.rs repeats it. One shared list in chora-kit is a follow-up.
3. xp re-grabs its chords when a screen joins its outputs; with the construct holding unbound Super chords after the settle second, a chord xp lost then stays lost until the cube is rebuilt. Not seen in any cell; noted.
4. Tabs left open: co-26 text "only the Keys item of the top bar" is explained, not a harness issue.
