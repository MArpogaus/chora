# ux r3 (rc 1): the last two open findings of ledger r2

Lane: opus, cloud, new branch claude/ux-r3 from claude/autonomous-dev-session-3w-i33pg7 (be205f89 or
newer). One round, fixed. Read dev-rules.md and conventions.md first, then reports/ledger-r2.md and
the lines ti-1 and sh-10 of findings/. Stop at 250 k context and hand over. logging r2 works in the
log code and dist/chora-report, flakes r5 in tests/ flake files, harness r3 in tests/merge-gate.sh,
tests/red-first.sh and hooks: keep away from those.
Items, red first (a gate cell or unit test commit before each fix):
1. sh-10 (MAJOR): the no-helper words must be whole in every look at 1280x800 and 1024x768, 1x and
   2x. Today tiling's chat header cuts `...who set up this c...`; construct's panel clips
   `...this compu` at its edge with no ellipsis. Fix by wrapping (the words are the content; KISS,
   no smaller type). The helper-none cells of the doors gate must read the whole second sentence,
   so a cut turns them red. Also check the waiting word of a question with no helper in each look.
2. ti-1 (BLOCK, verify): build fuzzel 1.11 (or newer) from its release tarball into a prefix under
   $HOME (no system install; meson and its deps are there or the item is a blocker: say so at once),
   put it first in PATH for the tiling launcher cell, and click a program in the launcher at 1x and
   2x. If clicks start the program: closed by the dependency (5dd34fcd); add a gate cell that skips
   with a clear reason when fuzzel < 1.11, and fails in CI (CI=true). If not: fix it.
Update findings/ (state column) for ti-1 and sh-10 with the cell that proves each.
Output: the full tests/merge-gate.sh once, green; reports/ux-r3.md on claude/session-log (item, red
commit, fix commit, cell, captures at 1x). Report to the PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most
15 lines).
Rules: push only claude/ux-r3 and claude/session-log (report, findings); author Marcel Arpogaus
<38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines; never --no-verify; never end a
turn to wait for a long background job.
