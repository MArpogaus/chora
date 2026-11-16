# xp polish r2 (rc 1): editor, volume level, question card

Branch claude/xp-polish, head 5a17aab (session branch merged, clean). Sonnet, one round.

- xp-25 editor: red e754091 (Writer, mousepad, gedit before nano), fix 20fc059. `lowriter`
  now leads the editor list, then mousepad, gedit, the rest; nano only as fallback. Docs and xp.yaml in step.
- xp-26 volume: red 851c9b1 (ink right of the cone must grow with the level, 1x pixel assert),
  fix 64432a5: tray speaker draws a level bar (1..12 px, 0..100 %). The tick on the current level
  in Control Panel > Sound was already right (unit test sound_shows_the_volume_and_mutes); the
  missing thing was a visible level in the tray.
- xp-27 question card: red cf78bef, e73198e; fix 03afb97 (one commit, mixed subject). Key words
  (Alt+Y/N, "spoken yes") now grey under the Yes/No buttons; "Typing does not answer" only in
  the field hint, once; the balloon grows (down over the figure when the screen is low) so the
  card never covers header or transcript, and the field follows (found at 2x: the card sat over the
  whole balloon). New cell tests/gate-xp-question.sh (1x and 2x), step xp-question in merge-gate.
- Evidence, media/2026-11-16/xp-polish-r2/ (build 03afb97): Run box, no-helper idle paperclip,
  question card 1x and 2x. Fallback art only, no Microsoft file.
- Gate: red-first, pre-commit, gate-lib, build, test, clippy, ci-check PASS; every later step
  PASS (xp, matrix, virtual, xp-faces, xp-paperclip, xp-fields, xp-question, hud, layout,
  tiling-audit, chat-keys, last-words, cube, pull, winit, wm-pane, walk, keys, pixels, mc-paths, e2e)
  except two. NOT RUN: 2d-guard (needs /home/user/wt/base2d, absent in this cloud container).
  FAIL: doors, mc look, cells screenshot-door ("no grim after the click") and volume-door ("no
  set-mute after the click"); xp, tiling, construct cells pass. mc code untouched here: not
  reproduced on a base, needs a look on the PM side. Steps ran in three invocations (--only),
  not one full run, because 2d-guard and doors stop the chain.
- Open: tray bar for 0 % is 1 px; at 2x the balloon covers the paperclip's head (tail points in).
