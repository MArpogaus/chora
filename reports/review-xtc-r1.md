# Review: xtc r1 (rc 1, rc 6), claude/xtc-fix at 91fb7d96

Reviewer lane: opus, read-only. Diff: `git diff claude/autonomous-dev-session-3w-i33pg7...91fb7d96`
(7 commits). Brief: briefs/xtc-r1.md. Findings: co-1, co-2 (construct.md), xp-2, xp-4 (xp.md), ti-2, ti-3, ti-4 (tiling.md).

Gates (worktree /home/user/wt/xtc, binaries of the branch):
- `tests/merge-gate.sh --worker ''`: build, test, clippy PASS (79 s, 61 s, 36 s).
- `tests/merge-gate.sh --red-first-check`: passes. Red shown by hand: the new scripts run on the session-branch
  binaries (498d9dea) fail: gate-tiling-apps "the launcher got no rows", gate-xp-run "the Run box got no rows",
  gate-xp-notepad "the settings of chora-xp have no editor". On the branch all three PASS.
- gate-tiling-bar-keys PASS; gate-tiling-notice SKIP (no mako in this container; not re-verified here).
- The two tiling gates the worker saw failing fail on the session branch too, with the same lines:
  gate-tiling-programs "FAIL: the launcher is not opaque or not in the palette", gate-tiling-power
  "FAIL: Super+Shift+e opened no power menu within 2 s". Cause: fuzzel 1.9.2 here rejects `selection-radius`
  of the theme; docs and packages name fuzzel 1.11 as the least version (first ok line of gate-tiling-programs).
  Not a regression of this branch.

Per item: co-1 verified by the new matrix cell "mc construct panes" (in the merge gate); co-2 verified
(gate-construct-keys, in the merge gate); ti-2 fixed except the typing state (xr-2); ti-3 and ti-4 verified by
new scripts; xp-2 fixed; xp-4 fixed (Notepad from `editor`, else the first installed editor; All Programs lists
installed programs only, which also covers xp-5).

Findings (id | severity | file:line | text | evidence):
- xr-1 | MINOR | tests/merge-gate.sh:150 | The five new scripts (gate-tiling-apps, gate-tiling-bar-keys, gate-tiling-notice, gate-xp-run, gate-xp-notepad) are not merge-gate steps, so ti-2, ti-3, ti-4, xp-2 and xp-4 are guarded only by unit tests and by hand runs | worker report "not steps of the merge gate"; grep merge-gate.sh
- xr-2 | MINOR | crates/chora-kit/src/launcher.rs:1 | ti-2 asks that an empty search says so; the word appears only after Enter. While typing, a search with no match still shows an empty panel (fuzzel has no such text). Not silent any more, but the finding is only partly met | worker report; launcher.rs `choose`
- xr-3 | MINOR | plugins/chora-tiling/src/main.rs:243 | Merge with claude/doors: `launcher::run` gets only the first dir that has applications/ and ignores `theme::data_dirs`; after doors moves Screenshot and Chat with a helper to flavors/shared/applications they drop out of the Apps list. Pass every applications dir (flavor and shared) at the merge | doors diff of theme.rs data_dirs and the same launch() block
- xr-4 | MINOR | tests/gate-construct-keys.sh:1 | The co-2 cell asserts that bare F1 shows the construct help; doors goal 1 says F1 is never grabbed and Super+F1 is the key list. Spec gap for the PM: either the construct walk mode keeps F1 (no client has the focus) or the cell changes with doors | briefs/doors-r1.md goal 1; commit 585c0e12
- xr-5 | MINOR | crates/chora-kit/src/desktop_entry.rs:64 | `on_path` takes any file on PATH as installed, without the execute bit; a non-executable leftover lists a program that cannot start | `dir.join(program).is_file()`
- xr-6 | COSMETIC | tests/xp.sh:1 | 605bccbd breaks the xp.sh white-column probe and 91fb7d96 fixes it one commit later, so 605bccbd alone is not green (bisect) | commit order
- xr-7 | MINOR | (new, outside the branch) | Worker observation: while the camera stands in a pane, a new construct window gets no pane, only the "No room in view" notice. Not a handover, so not co-1, but a window the person cannot see; PM to triage as a construct finding | worker report, "Hidden new windows in the construct"

Note for the PM, not counted in the verdict: the theme writes `selection-radius=8`, which fuzzel before 1.11
rejects, so on Debian or Ubuntu (fuzzel 1.9.x) the tiling launcher and power menu do not open at all. The
packages name 1.11; a fallback or a startup check would make the failure visible to the person.

Verdict: MERGE-OK (no open BLOCK or MAJOR). xr-3 must be handled at whichever of doors and xtc merges second.
