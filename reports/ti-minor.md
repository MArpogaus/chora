# ti-minor

## r1

Base fee9c695, branch claude/ti-minor, no code change: all three items were already fixed on the head, so no red cell can be written (a cell that is green before the fix is not a red cell).

| item | state on fee9c695 | where | cell |
|---|---|---|---|
| ti-5 | chora-notify reads the open chat box at start (`notify.ready`, the host answers `assistant.chat`) and the card steps aside; the finding came from the mako era | 03e06230 (red bdb5af3a), unit `the_card_steps_aside_of_the_chat_panel` | tiling-audit (unit in test) |
| ti-6, ti-19 | `"tooltip": false` on Apps, helper, Keys and power | c1adc900 | tiling-look (cell tooltip of gate-tiling-look-r2) |
| ti-7 | `hide_bar_for_fullscreen` sends SIGUSR1 to waybar while a window is fullscreen | 6092aee3 | tiling-audit (cell fullscreen) |

Gate on fee9c695: `tests/merge-gate.sh --jobs 2 --only build,tiling-audit,tiling-look,tiling-empty` PASS (build 132 s, tiling-audit 253 s, tiling-empty 21 s, tiling-look 533 s). Not run: pre-commit, ci-check, test, clippy, gate-lib, layout, chat-keys, doors (nothing changed).

Open: the findings come from older captures; a fresh capture on the head would confirm the eye sees it too. No cell checks a toast started while the chat is open end to end (only the unit does).
