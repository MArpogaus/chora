# Review xp-polish r5 (f40b2ddc)

Verdict: FIX (2 MAJOR, 2 MINOR)

## Findings

### MAJOR 1: "Notepad opened with X" is said before X opened
plugins/chora-xp/src/launch.rs:96-106 (text), :219-229 (run_chain), :280-300 (fail).
The Fallback notice is built at the moment the next editor is *started*, with the title "Notepad opened with gedit".
If gedit then cannot be started (Refused), ends without a window, or is slow, the user already has a notice that claims success,
and a second "Notepad opened with ..." or "Cannot open Notepad" follows (builder's "Open" list admits the stacking).
The brief asked for no claim of success when spawn fails later. Fix: show the fallback notice only once the window of the
fallback shows (Done path), or word it as "Starting Notepad with X" until then and say "opened" only on Done.
Evidence: Change::Fallback is pushed in fail() before begin() of `rest`; nothing delays it.

### MAJOR 2: the 2 s early-failure rule can start a second editor over a slow one
launch.rs:382 `early && silent && age >= SHOW_AFTER && pid.is_some() && !alive` => Why::Exited => next editor starts.
Editors that hand their work to another process by design (kate forks to the background; gedit/gnome-text-editor/lowriter
handing over to a running instance) leave a dead launcher pid while their window is still coming up. A cold start slower than
2 s (loaded machine, KDE cold start) shows a false "ended without a window" notice and opens a second editor, so two editors appear.
The old 30 s rule existed for exactly this case (doc comment of check() mentions hand-off). Only fakes were tested
(lowriter `exit 1`), no hand-off fake. Fix: for the early rule use a longer limit (e.g. 5-10 s) or require that no
window shows by GIVE_UP when the pid died with status success; at minimum add a cell/unit test for a forking editor.
Not verified by running (no kate/gedit in the container); by reasoning from the code.

### MINOR 1: the skipped-editor notice repeats on every click of Notepad
wm.rs:2982-2995 (self.notepad.skipped is kept). Builder lists it as open. Not a regression of the brief but noisy for a
permanently misconfigured setting; POLS would say once per session or only when it matters. Not blocking.

### MINOR 2: the Notepad match is by command string
wm.rs:2982-2987 compares `command` with chain[0].command. Any other menu entry whose command equals it (e.g. a user
program "gedit") also gets the fallback chain and the name "Notepad". Harmless but implicit.

## Checks asked

1. Brief: setting, then graphical list, then terminal editor (foot -e nano) in a window, one notice naming which and why, or
   "Cannot open Notepad": implemented (apps.rs editors(), launch.rs run_chain/fail). Default list in docs matches EDITORS
   (docs/configuration.md line 2220, xp.yaml comment): same 12 + VISUAL, EDITOR, nano, vim, vi. Honest in the Missing path
   (checked before spawn) and in the all-fail path; NOT honest when a fallback fails later (MAJOR 1). "One notice" is true only
   for one fallback step; a longer failing chain gives several.
2. Red test: 23a8c476 is a real red. Without fd11858d the base apps::editor picks the first installed editor (fake gedit),
   a window opens, and no Notify with "Notepad opened with gedit" is ever sent, so wait_notice at
   tests/gate-xp-notepad-fallback.sh fails after 30 s (builder saw the same). The cell's fakes are isolated (PATH without
   real editors, own HOME/XDG). I did not run the cell: it needs the full chora/choractl build; skipped for disk and the
   one-build limit. Cells do not cover a hand-off editor (MAJOR 2) nor a failing fallback after a successful spawn (MAJOR 1).
3. Safety: no new shell construction; commands go through the same chora_kit::launch -> `choractl run` path as before, with
   the user's own `editor`/VISUAL/EDITOR (same trust as before); the starts run in std::thread, the compositor thread never
   blocks; no unwrap/expect added; zombies handled (alive() treats Z as dead; reaping is the compositor's existing path).
   The activation token is reused for each fallback start (xp token from one request): acceptable.
4. KISS/docs: docs and xp.yaml match the code. Scope: Fallback/Exited types, Next struct, early flag: more state than
   needed but contained. `editor()` kept as a wrapper. No unrelated scope creep.
5. Diff files: docs/configuration.md, plugins/chora-xp/{xp.yaml,src/apps.rs,src/launch.rs,src/wm.rs}, tests/gate-xp-notepad-fallback.sh,
   tests/merge-gate.sh (one step line extended). No chora core/kit/tiling/fuzzel change, so gate-tiling-apps-click cannot be
   affected by the diff (it uses fuzzel, not Notepad). Consistent with the builder's claim of a pre-existing failure with fuzzel 1.9.2.
   Note: merge-gate.sh edit makes xp-notepad run both scripts with `&&`; fine.

## What I ran
- df -h /: 20 G avail (>= 15 G), builds allowed.
- CARGO_TARGET_DIR=/home/user/wt/rv-xp5/target cargo test -p chora-xp --locked: 183 passed, 0 failed.
- cargo clippy -p chora-xp --locked -- -D warnings: clean.
- Not run: gate-xp-notepad-fallback.sh (full build needed), merge-gate.sh (forbidden).
