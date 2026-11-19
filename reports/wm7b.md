# wm7b r1 (F3): the xp cube hosted on construct — handed over at 250 k, not done

Lane: opus high, cloud, branch `claude/wm7b` from f4e4f4d9. Head 1ae3f243 (pushed). No gate run yet.
Commits: 5322c586 `test(acts)` red (modifier drag, kill without restart, restart, pager without construct);
e450d672 `fix(core)` (+test, red first: a button grab of one client ends another client's lone-modifier tap,
so Super+drag on a face no longer opens xp's Start menu at the release; found by the new act, I4 xp-menu);
8ca85eab `fix(xp)`; 1ae3f243 `fix(construct)`.

## Items (cell: tests/acts/run.py construct on 1ae3f243's build, before the last act fix)
1. (a) PASS act "a drag with the modifier on the front face turns the cube" (construct cube/hand.rs; xp grabs no
   Super+BTN_LEFT while hosted or flat). `turn_to` uses `chora_kit::cube::quarters_to` (copy gone).
2. (b) a2 PASS act "the pager turns the cube" (xp pager on faces calls `cube.turn {to}`; row wm7a-a2 removed).
   rw-5 was in r2 already. a1 OPEN: I3 still breaks in "the key of the step back stands back": the core
   focuses construct-look, xp says it holds the probe window (xp ignores focus.changed {surface: null}).
   Row wm7a-a1 stays, but its match no longer fits: acts BLOCK on I3. xp no longer claims the focus without a
   window on virtual-only outputs.
3. (c) m2 PASS act "construct killed mid-turn leaves the windows with a flat xp within 3 s" (xp has no
   PDEATHSIG, stderr to a followed file; on the host's disconnect xp takes the screens, face i -> desk i).
   PASS act "construct started again reopens the cube with the windows on its faces" (construct reopens around
   the running xp; xp goes back on the role event); m1 covered there (start card not beside panes).
   Last run's I1 FAIL ("construct answers no state.get") is fixed in 1ae3f243 (s.looks), not re-run.
4. m3 (rw-8) done: `last_word` + `tell_end`, unit test in role.rs. m4 OPEN (MANAGER still the constant "xp").
5. Flat fallback: xp on virtual outputs without a host switches desks flat, key list "Switch to the desktop on
   the left/right" (help.rs unit test). OPEN: act "without construct the pager of xp switches the desk" got
   desk 4 for a click on button 2 (not diagnosed). xp's own cube on a screen stays (deviation: removing it
   is a visible change to the xp flavor and large; proposal below).

## Not run / open for the next round
- `tests/gate-construct-cube.sh` section 6 still expects "the xp of the cube ends with a construct that dies"
  and "a restarted construct removed the outputs": must change to the new behaviour (it will FAIL now).
- No `tests/merge-gate.sh --only ...` run, no kit coverage run, docs/api.md (input.key: tap ended by a button
  grab; xp hosted/flat) and configuration.md (cube_manager outlives the construct; xp pager on faces) and the
  decision record line: not written.
- `cargo test -p chora-construct -p chora-xp -p chora-kit --features chora-kit/testing`: all ok on 1ae3f243
  minus the last two clippy edits (clippy --workspace clean after them). Size: cube.rs shrank (launch,
  manager_runs, remove_left_outputs moved to cube/start.rs).

## Refactor proposals
- xp/cube.rs + wm/desktops.rs turn rendering (~500 lines): drop xp's own 3D cube now that the cube lives in
  construct; flat desk switch on screens (owner call: visible change to the xp flavor) (L).
- xp focus: one `focus_away` that also counts a focused node of another client (wm/state.rs:41) (S).
- construct `Host.child: Option<Child>` plus `end_manager`: a small `Manager` type with child, tail, command (S).
- tests/acts: `click_box` (construct.py) belongs in harness.py next to `click` (S).
Cut: core touched (input.rs, one rule); no plugin-to-plugin name added (xp finds the host by the role).

## r2
Lane: opus high, cloud, branch `claude/wm7b` from 1ae3f243 + train 62 (merge a636b9d2; conflicts in xp
startup.rs and state.rs resolved on kit::wm::state types). Head 31f04c06 (pushed). Train unmoved at the gate.

Items (red -> green):
1. I3 a1: 14dad42 -> 60a2334. construct's step back sends `focus {surface: null}` before `focus_node` look,
   xp drops its window on focus.changed null; row wm7a-a1 out of known-red. Act "the key of the step back
   stands back" pass. Follow-on found by the acts: the restart act broke I3 (fresh construct claimed while an xp
   window had the focus): red = that act on 0acdad8, fix 6e79fb7 (reopen reads the focused surface).
2. Flat pager act: diagnosis = face order, not index base: host_gone handed each face's windows to the next
   bar in turn, so all landed on the last desk (desk 4). 6078eed -> dc208f8 (faces go straight to the screen).
   Second cause: construct's resume nodes (sky, look) stay 10 s after a kill and cover the screen, so the click
   hit them; the cue now waits for the kept nodes (GRACE 15 s). Act pass.
3. m4: f7819fb -> ae3b49d. `MANAGER` gone; face manager = client of cube_manager's program without `chora-`
   (Host::client, Construct::face_manager); unit tests in role.rs and names.rs. Reading of "found by its role":
   the core gives no pid/role for the manager, so the setting names it (backlog wording, KISS).
4. Gate section 6: ec29903 (dying construct -> flat xp with the window on HEADLESS-1 within 3 s, same xp pid;
   restart -> cube.list 4 faces, window back on a face; cube.close ends that xp). The section-5 fake manager is
   now a program `chora-xp` (labels say XP), as the client name follows the program.
   Found by it: the xp of a reopened cube outlived cube.close: 3579bc2 -> 31f04c0 (host kept while its faces go).
5. I1 restart act ("construct answers state.get"): pass in gate acts (cell list in acts.log).
6. Docs 0acdad8: api.md input.key (button grab ends another client's lone-modifier tap), xp hosted/flat;
   configuration.md cube_manager (client name, outlives the construct), cube.open/close, pager on faces; D106.
Also: xp-faces no-cube-on-faces was broken by r1's flat desk switch on pane-host outputs: b68e9e7 -> e973e15
(no pager, no turn keys, turn_to inert on virtual outputs without a cube host; hosted pager unchanged).

Gate: `tests/merge-gate.sh --only build,acts,cube,pull,construct-hud,xp,xp-faces,keys` on 31f04c0 (run
/tmp/merge-gate.v4lOkT): build PASS 21s, xp PASS 83s, xp-faces PASS 15s, acts PASS 95s (89 pass, 7 known red,
0 blocking), cube PASS 104s, pull PASS 63s, construct-hud PASS 229s, keys PASS 17s.
`cargo test -p chora-construct -p chora-xp -p chora-kit --features chora-kit/testing` ok (204, 185, 255, ...);
clippy --workspace --all-targets clean.

Open / findings:
- NEW MAJOR candidate (vision card): after a construct crash its kept resume nodes (sky, look, face) cover the
  screen for the 10 s grace time: windows are with a flat xp at once but unseen and unclickable until then.
  Fix needs core (hide kept nodes of a disconnected client during grace, or no grace for overlay nodes). Not built.
- Backlog proposal (owner call): drop xp's own screen cube (xp/cube.rs + desktops turn, ~500 lines).
- help.rs `flat_desks` text is now unreachable (Flat holds no turn key): remove in a refactor (S).
- Refactor proposals of r1 stand (focus_away counting other clients' nodes, Manager type, click_box to harness).

## Review r2
Reviewer: opus high, cloud, read-only on claude/wm7b 31f04c07 (17 own commits over daf83391 + wm7a f4e4f4d9).
Verdict: FIX (0 BLOCK, 1 MAJOR, 5 MINOR).

Checks:
1. Items: a1 closed (60a2334, known-red has no wm7a id); flat pager (dc208f8, face order; act pass per the lane's
   gate log, not re-run); m4 sound for chora-* commands, brittle otherwise (rw-4); gate section 6 (ec29903) states
   the new behaviour, 3 s bound = 12 x 0.25 s, same xp pid, cube.close ends the reopened xp; I1 act pass per gate
   log (not re-run); docs done, one stale line (rw-5).
2. Red first: 14dad42->60a2334, 6078eed->dc208f8, f7819fb->ae3b49d, b68e9e7->e973e15, 3579bc2->31f04c0, ec29903
   test only; 6e79fb7's red cell is the restart act of 5322c586 (ok); e450d672 (core) has no red commit (rw-6).
3. Extras are real fixes: 6e79fb7 reads the core focus on reopen; 31f04c0 keeps the host while faces go (xp ends
   with the last face, unit test); e973e15 restores the pre-wm7b pane-host behaviour (no weakened check).
4. Worker's MAJOR candidate confirmed, rated MINOR (rw-2).
5. clippy --workspace --all-targets -D warnings clean; cut-check FAIL (rw-1, clean on base+wm7a); ci-check skipped.
6. cargo test -p chora-construct -p chora-xp -p chora-kit --features chora-kit/testing: 699 pass, 0 fail;
   size-ratchet ok (cube.rs 1617 counted, listed 1641: shrank; no new file over 1,500).

Findings:
- rw-1 MAJOR plugins/chora-construct/src/cube/start.rs:34 (+ cube/role.rs:128,134, names.rs:91,158,161): cut-check
  FAILs new in wm7b: XDG_RUNTIME_DIR outside kit (KIT1 allows 0), construct names chora-xp 9/5 and chora-mc 19/17;
  REPORT Command::Focus outside kit::wm 18->19 (modes.rs:749); the train's ci-check goes red. Fix: log dir via
  chora_scene::paths::runtime_home(), neutral program names in tests and doc comments, and the focus via a kit::wm
  helper or a reasoned row raise.
- rw-2 MINOR crates/chora/src/ipc.rs:1191 (GRACE ipc.rs:71; kit conn.rs:43 resume; acts construct.py:282-291):
  confirmed: a crashed construct's kept sky/look/face nodes cover the screen for up to 10 s while the windows are
  already with a flat xp, unseen and unclickable; crash path only, transient, no window lost (RC bar holds), core
  rule older than wm7b. Fix (backlog, core): hide a disconnected client's kept nodes during its grace, show on resume.
- rw-3 MINOR plugins/chora-xp/src/wm/keys.rs:225: I3 covers only construct's look (modes.rs:749 sends focus null);
  a focus_node of another client (chat, assistant) emits no focus.changed (crates/chora/src/state.rs:1035), so xp
  still draws its window active. Fix: the core emits focus.changed {surface: null} when a node takes the keyboard.
- rw-4 MINOR plugins/chora-construct/src/cube/start.rs:133 (names.rs:93): the face manager's client is guessed from
  the command's program, not found by role; `env X=1 chora-xp` or `sh -c ...` gives "env"/"sh", so the runs-already
  check and float.rs `{client}-win-` miss xp's windows silently. Fix: take the client that claims the face outputs
  once it connects, the setting guess only before.
- rw-5 MINOR docs/api.md:1619: "On virtual outputs without a host, a desk switch is flat" contradicts e973e15 (no
  pager, no turn keys there); help.rs:139 flat_desks and panels.rs:113 are dead. Fix: say "no desk switch" and drop
  the Flat key-sheet text.
- rw-6 MINOR crates/chora/src/input.rs (e450d672): core fix and its test in one commit, no red commit first
  (rules.md 8); 1ae3f243 also edits acts in a fix commit. Fix: none in code (no history rewrite); record it.
