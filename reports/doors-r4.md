# doors r4 (rc 1: one helper word rule), claude/doors at 57df3822

`tests/merge-gate.sh` as one whole run on 57df3822: every step passes (run /tmp/merge-gate.4p62KG in the worker).
The step tiling-audit passes as a skip: mako is not installed in the container.

1. dr3-1: the island says `No helper` whenever the host knows no running helper and voice rests; the cached
   answer of `agent_list` is gone, so the word follows `voice.state` and `voice.remove` of helpers. Red (cb573922):
   `check construct helper-exit fail`, gate-hud `helper-word missing`. Green (279e4b72): both pass. The check stops
   coder on one voice connection, reads `No helper`, starts it again and reads `Listening`.
2. dr3-2: the host makes no call to the agent for the word any more, so nothing repeats or blocks.
3. dr3-3 (57df3822): `shared::entries_in` is the one rule; `launcher::entry_dirs` and `data_dirs_in` use it.
4. Session branch 4e3c019e merged (01613f69). ti-1 clicked with the real fuzzel 1.9.2 in a tiling session: the
   Apps list opens with tiling's fuzzel.ini, but hover does not move the selection and a click starts nothing.
   Cause: fuzzel 1.9.2 has no mouse selection (upstream CHANGELOG: mouse support is new in 1.11.0); the same
   fuzzel under sway ignores the click too. Down+Return starts the program in chora. ti-1 stays open until the
   target has fuzzel 1.11 or newer.

Open: gate-hud cells of a late host and a reconnected voice read `Working` now (a resting word needs a helper).
A host that starts after a helper says `No helper` until that helper's next state event, as the xp door does.
