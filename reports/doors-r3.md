# doors r3 (rc 1: finish the shared doors), claude/doors at cb203c68

Worker: Fable lane, cloud, no sub-builders. Session branch de55f346 (with claude/xtc-fix) is merged in (77a00321).

## Result
`tests/merge-gate.sh` as one whole run on cb203c68: every step passes, mc-paths included (run
/tmp/merge-gate.DRtcKt, one log per step there; doors.log has the table). gate-doors: 80 of 80 cells, plus the
checks `xp log-off`, `construct pane-corner` and `construct layout-1280x800` / `layout-1024x768`.
Earlier whole runs: 62c95b81 before the merge passed (/tmp/merge-gate.8OyMTq); 38e09511 failed only at `keys`
(/tmp/merge-gate.UVdGgd): a check of the session branch pressed F1 for the construct help; it presses Super+F1 now.

## Items
1. dr2-1: the HUD host asks `agent_list` once per voice connection; with no helper and no project, and voice
   asleep, listening or idle, the figure and the island say `No helper` (busy states keep their word, so gate-hud
   still reads Listening). The glass chat names its header in the meta of `hud-chat`. A plain view in a slot (mc) has
   no header: an empty list says why in the place of the hint, so mc dropped its own line.
2. dr2-2: the construct strip hides in a pane (flight in included) and returns after the step out. Check
   `pane-corner` clicks where Lock stood: the window gets the press, nothing locks. Red before: it locked.
3. dr2-5: `chora_kit::glass::cap` names Vol+, Vol-, Mute; words Volume up, Volume down, Mute in xp, mc, construct.
4. dr2-4, dr2-10: `chora_kit::shared::launcher`, one quoted word, for fuzzel; `shared::script` and `data_dirs` private.
5. dr2-3 (one notice, red: 2 lines), dr2-7 (island says Listening, the list names `coder · idle`), dr2-8 (the
   audit is a counted check and runs at 1024x768 too; its result was discarded before).
6. xr-3/dr-6: after the merge `launcher-chat` was red in tiling and in xp (the Run box builds its rows too).
   `chora_kit::launcher::run` reads the shared folder after the flavor; the cells read the rows of fuzzel now.

## Commits (after dd8669a4)
03e24c1b, 2a858099 feat(assistant); b6354fbd fix(mc); 240ec1e1 test, 19a5beaf fix(construct); 3b7d7f4c test,
efac1ed2 fix(keys); afc0fec8 test (xp Start menu click flaked 1 of 3, 5 of 5 after); 7beef6ed fix(kit);
357f9d7d fix(mc); 62c95b81 test; 77a00321 merge; 28a36285 test, 38e09511 fix(kit); cb203c68 test(construct).

## Open, deviations
- ti-1 NOT verified with fuzzel 1.11: the container has fuzzel 1.9.2 (the only apt version; lanes install nothing).
  1.9.2 rejects flavors/tiling/fuzzel.ini (`selection-radius: not a valid option`), so the real Apps list does not
  open. A plain real fuzzel 1.9.2 in a tiling session: hover and click leave the selection on the first row and
  choose nothing, so ti-1 reproduces with 1.9.2. Needs fuzzel 1.11 in setup.sh, then one real click.
- Tiling shows the volume keys in one row (`Vol+ / Vol- / Mute`, `Volume up / Volume down / Mute`): three rows
  make the list 700 px at pitch 22, 4 px more than the room at 1024x768, and pitch 20 leaves 2 px between caps.
- 62c95b81 holds dr2-7 and dr2-8 together; its message names only dr2-8 (a rejected subject, no rewrite of pushed history).
- Seen in the mc capture, not this lane's item: in the header the helper name, the word Mute (session branch), the
  date and the island of the host overlap at 1280x800 with voice and the host running.
- The island says `No helper` only when voice names no project either; xp, tiling and mc say it when no helper runs.
- Not done by brief: dr2-6, dr2-9 (backlog).

## Captures at 1x, 1280x800 (in the worker, not committed)
/home/user/wt/captures/doors-r3-{xp,tiling,mc,construct}.png and doors-r3-{xp,tiling,mc,construct}-keys.png
