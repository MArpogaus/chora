# doors r3 (rc 1): finish the shared doors

Lane: Fable, cloud, branch claude/doors (at dd8669a4). One round. Stop at 250 k context and hand
over. Read dev-rules.md and conventions.md (with the skills it names) first. Setup: clone
MArpogaus/chora to /home/user/chora, fetch all branches, worktree /home/user/wt/log on
claude/session-log, `bash /home/user/wt/log/setup.sh`. Read reports/doors-r2.md and
reports/review-doors-r2.md.
Items, red first, in this order:
1. dr2-1 BLOCK: construct helper-none: chora-assistant gives the HUD host Word::NoHelperDoor when no
   helper exists, and the chat header text in the node meta; the construct island shows it. Then
   mc uses the same word instead of drawing its own NoHelper line. gate-doors.sh 80/80.
2. dr2-2 MAJOR: the construct capsule neither draws over nor takes clicks from a flown-into pane;
   a cell clicks a window's title bar through that corner in a pane.
3. dr2-5 + PM decision 11-14 19:37: one keycap rule in chora_kit::glass (Vol+, Vol-, Mute) and one
   action text in every key list ("Volume up", "Volume down", "Mute"); fix the construct keys-names
   cell.
4. dr2-4: the launcher env wrap lives once in chora_kit::shared (or env on the Command), quoted.
5. dr2-3 (duplicate notice), dr2-7 (construct helper cell checks name and state), dr2-10
   (shared::script private), dr2-8 (layout audit at 1024x768 too).
6. When the session branch has claude/xtc-fix (it merges soon): merge it in and fix xr-3/dr-6 (the
   tiling Apps list reads flavors/shared/applications). Verify ti-1 with a real fuzzel click.
Not this round: dr2-6 (mc volume polling) and dr2-9 go to the backlog.
Done when: tests/merge-gate.sh passes as one whole run (say which run and its log); captures at 1x
per look (list the paths). Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 20 lines)
and as reports/doors-r3.md on claude/session-log.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution
lines; never --no-verify; stage by name; no force push; push only claude/doors and that report;
never commit Microsoft files or captures with them; never check out another branch in an existing
worktree. Sub-builders are allowed only if you review their diffs line by line before you merge them.
