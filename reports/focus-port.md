# focus-port: the four looks hand focus through kit::wm::focus (WM2b; F2, F3, F7)

## r1
Lane r1 (opus, high) on claude/focus-port from claude/integ 3619ccb3; integ merged before the gate (3b418008).
Setup: setup.sh was refused by the permission classifier of this session; its repo steps were done by hand
(git identity, pre-commit hooks); every tool it checks was present. `cargo check -p chora` green.

Commits: 39b27a4b (red), bda9d4ae (port), d0d7ec09 (kit7-a1 fix, row removed), 7a56a5b9 (kit7-a2 finding text), 3b418008 (merge integ).

- Red first: 39b27a4b sets tools/copies.toml "Command::Focus outside kit::wm" and "Event::FocusChanged outside kit"
  to max 0 (also the mc and tiling WM2 rows), allowing chora-scene (defines and applies the command), core
  ipc (checks who sends it), kit testing (the FakeCore records it), kit wm; `tools/cut-check` FAIL: 10, 2, 1, 9 left.
- Port bda9d4ae: kit wm::focus gains `changed(&Event)`, `give(surface)`, `Focus::hear` returns the surface,
  `Focus::core()`; kit testing `focus_changed(surface)` for the cells. mc (main.rs, state.rs), construct
  (plugin.rs, input.rs, scene.rs, camera.rs, adopt.rs), tiling (main.rs, windows.rs), xp (wm/windows.rs,
  wm/dispatch.rs), assistant (main.rs) name neither type. Both rows at 0; cut-check exit 0 after the integ
  merge, no new REPORT line (the FocusChanged "grew to 9" line went). Tests of the six packages pass, except
  the two known integ reds of xp (fixed by the integ merge); clippy of the six, all targets, clean.
- kit7-a1 (mc I3): **regression of WM2** (3d2d778f, 11-19). Steps (tests/acts helper_acts, replayed by a
  scratch script on the head harness with BIN=v8): mc session, voice with fake-acp and chora-assistant,
  agent_start coder, agent_say, the question, then `methods.call assistant chat.show`.
  v8 733967aa: core focus before and after chat.show `[mc-s-agenda-page (group, mc)]`, mc has no state.get
  (no claim, so I3 cannot break). head: same core focus, mc state focus `{"holder":"mc"}` before,
  `{"holder":null}` after. Cause: mc called `focus.lose()` on every `assistant.chat` open, but a chat.show
  without an activation token does not give the field the keyboard. Fix d0d7ec09: mc loses the focus on
  `assistant.chat.keyboard {on:true}`; cell `the_chat_takes_the_focus_of_mc_only_with_the_keyboard`;
  `tests/acts/run.py --no-selftest mc` then: "mc I3 passes but is listed", row removed.
- kit7-a2 (construct Tab act, I4): **old**. Same steps on both builds (construct session, chora-probe window,
  press and small drag on its pane, a second window, Tab). head: core focus `s1` after the press and after
  the second window, construct state open `[]`. v8 (no scene.boxes: the press at the pane centre 640,400 that
  head reports): core focus `s1` after the press and the window, 4 construct-list nodes, no rows (closed).
  Control without the press: the list opens on both (42 nodes with rows; head open `[construct-list]`).
  Root cause: a press on an own pane window gives that window the keyboard (input.rs own-window press);
  a new window does not take it and nothing gives the keys back to the walk, so Tab goes to the window.
  Not a port regression; whether a new window should return the keys to the walk is a design question for
  WM2 (D31). Rows stay listed; their finding text names v8 (7a56a5b9).
- Seen on the way: on 3619ccb3 one acts run (beside a parallel v8 build) also failed construct I14
  (`ring.frag` of chora-assistant missing) and "back at 1:1 no construct surface" (8754 px); both passed in
  the gate's acts step, so load, not the port.
- Gate at 3b418008, `tests/merge-gate.sh --jobs 3`, once: **red on a flake**. 25 steps PASS (red-first,
  pre-commit, gate-lib, build, test, clippy, ci-check, 2d-guard, package-deps, package-lists, virtual, xp-faces,
  xp-paperclip, xp-fields, xp, xp-question, xp-notepad, hud, matrix, layout, chat-keys, tiling-audit, acts,
  last-words, doors); pull FAIL in 7 s: "the pane of the pulled window is not in front of the face at 1:1 or
  nearer: 194 px wide for 200" (the open pull width flake of flakes-g, audit-f F5). The gate stops at the
  first red, so the steps after pull did not run. `tests/merge-gate.sh --only pull` then: PASS 67 s, PASS 68 s.
  The port does not touch the pull path (construct camera/adopt/input only swap the command builder).
- Open: the steps after pull want a gate run in the train; kit7-a2 stays known red (design question above);
  setup.sh is refused by the classifier in lane sessions (it clones agent_skills and installs tools).
