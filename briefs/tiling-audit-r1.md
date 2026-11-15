# tiling audit r1 (rc 1, rc 3): the audit that mako woke up

Lane: opus, cloud, new branch claude/tiling-audit from the session branch (8bcbdc73). One round. Stop
at 250 k context and hand over. Read dev-rules.md and conventions.md (with the skills it names)
first. Setup as briefs/ci-r1.md (setup.sh now installs mako, so the tiling-audit step no longer
skips). Never end a turn to wait for a long job.
Failure (train 11 gate, 01:4x UTC, and every gate from now on): tests/merge-gate.sh step tiling-audit:
  desktop paper-2560x1600@2 and question paper-2560x1600@2: "tokens tile border: [(90, 99, 224),
  (49, 140, 175), (83, 97, 224), (40, 136, 173)] is not `line` (213, 216, 225)"
The audit skipped before because mako was missing, so this flaw is old, not from train 11.
Decide with evidence (captures at 1x and 2x, design/looks/tiling.md, the tiling v2 gradient focus
border): is a tile border drawn in the focus gradient where the look wants the plain `line` colour
(a product bug, e.g. the wrong tile counted as focused, or the paper theme at scale 2), or does the
audit read the focused tile and expect the unfocused colour (a test bug)? Fix the cause, red first for
a product bug; never loosen the audit beyond what the look spec says. Also run every other audit
case of the step (all themes and sizes).
Done when: tests/merge-gate.sh passes with the tiling-audit step running (not skipped). Report to the
PM session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 10 lines): cause with evidence, commits.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines;
never --no-verify; stage by name; no force push; push only claude/tiling-audit.
