# Review xp-polish r6 (round-6 commits e23952b7, 6399fd47, c7e4e59e)

Verdict: MERGE-OK

Base (merge-base with origin/claude/autonomous-dev-session-3w-i33pg7): 4105f8d1. Head: c7e4e59e. Read-only review, no build or test run.

## Findings

BLOCK: none. MAJOR: none.

MINOR 1, launch.rs:448-453 (check, GIVE_UP branch): a fallback start whose launcher pid is still alive at 30 s with no window gets the success notice "Notepad opened with X" (served after Done). A hung editor that never shows a window would be named as having opened. Brief allows "still running at 30 s" as the defined fallback; the wording stays optimistic. Scenario: fake fallback `sleep 600` without window -> notice at 30 s. Rare; not a normal flow.

MINOR 2, launch.rs StatusFile::create (about line 158-170): if create_new fails (stale file of a crashed chora-xp with the same pid and counter, unwritable dir) status is None and the start silently loses early fall-through, keeping the 30 s limit; nothing is logged. Safe failure (no doubled editor), only slower.

MINOR 3, process: the unit tests of launch.rs came in the fix commit c7e4e59e, not in a red commit before it. The gate cells (the user-visible behaviour) did get red commits e23952b7/6399fd47 with red evidence in the builder report (old binary fails both cells).

COSMETIC: A start that replaces the shell with `exec` (e.g. editor command `exec nano`) writes no status and so has only the 30 s limit; builder already lists this as open. Documented behaviour is consistent with it (docs say an editor that ends without an error waits 30 s).

## MAJOR 1 (notice only after the window shows): verified fixed
- fail() (launch.rs ~335-355) no longer emits Change::Fallback; it keeps `fell = (from, why)` in the Next of the following start and pushes Done to close the "Starting" message.
- Fallback is emitted only in served(), called from window() (token matches, entry removed), from add() when the token's window already opened (race of early window vs start answer), and from the 30 s alive path.
- A failing chain ends in Change::Failed ("Cannot open Notepad") with no earlier success notice; unit test a_fallback_that_ends_without_a_window_leaves_no_notice_of_success and the gate last cell (lowriter exit 1, mousepad exit 3) assert it. The gate negation `! notice ... || fail` is correct (fails when the notice exists), and wait_notice "Cannot open Notepad" runs first, so the check is not racing the notice. Not vacuous: red evidence on old binary.
- Missing-binary chain (missing() path in begin) goes through fail() too, so only "Cannot open Notepad" results; unit test updated.

## MAJOR 2 (no early fall-through on exit 0): verified fixed
- The old `age >= SHOW_AFTER && pid dead` test is gone; early fall-through is now only `status file non-empty` (non-zero exit) while silent. Status 0 writes nothing, so a hand-off editor (forks, exits 0, window after 4 s) keeps its entry until its window (token) or 30 s. Missing binary: begin() fails at once via missing(); a shell 127 writes a non-zero status.
- Shell command: `{ CMD\n}; status=$?; [ $status -eq 0 ] || ! [ -e FILE ] || echo $status >FILE; exit $status`. Group with newline before `}` handles a trailing comment or `&` in CMD. Path single-quoted through quoted() (embedded quote escaped); path is chora-xp's own (XDG_RUNTIME_DIR, pid, counter), no user text, so no injection. Status is preserved as exit code.
- Races: file created with create_new before the start (file exists but empty = "not written", failed() requires non-empty text; a truncate-then-write read just retries on the next poll). Stale files cannot be mistaken: name includes pid and a process counter, create_new refuses an existing file (fails safe, see MINOR 2). Concurrent launches each get their own file. The `-e` guard stops the shell from recreating a file that Drop already removed, so no leak after the entry is gone (small -e/echo window is negligible). Drop removes the file with the last Arc holder; unit test the_status_file_goes_with_the_start covers it.
- Gate hand-off cell: fake kate forks (window after 4 s with app id notepad-handoff), exits 0; asserts the window opens, waits 8 s, asserts no notepad-second (gedit) surface and no "gedit ... kate" notice. On the old binary the gedit fallback started at 2 s, so the cell fails there (red evidence given). Not vacuous.

## Other checks
- No stray keys or lost window: no input code touched; fallback start happens only after fail; the `served` ordering (opened-token race) is covered in add().
- KISS: acceptable; one small struct, one shell wrapper, Next carries `fell` and `status`. Early flag only for run_chain (Notepad); single-command launches unchanged (wrap only when template.early).
- docs/configuration.md editor row and plugins/chora-xp/xp.yaml comment agree with the behaviour (xp.yaml text still accurate, "when one does not open" is general; configuration.md adds the 30 s hand-off note).
- Gate: builder reports all cells and merge-gate green except e2e gate-tiling-apps-click (fuzzel 1.9.2, also on base); not re-run here.
