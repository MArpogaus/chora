# Review xp-polish r7 (head 2179966)

Verdict: MERGE-OK (no open BLOCK or MAJOR; 4 MINOR, 2 COSMETIC). Read-only diff review; cargo not run (builder: 188 tests ok, merge gate green).

## Checks
1. Notepad GUI only: PASS. nano, vim, vi, VISUAL and EDITOR left the chain (plugins/chora-xp/src/apps.rs:76-134, TERMINAL_EDITORS and env removed). A user-set `editor` still leads the chain (apps.rs:121-130); unit test `editor(Some("foot -e nano"), ...)` keeps it (apps.rs:251-254).
2. No GUI editor: PASS. `load` always adds Notepad with NO_EDITOR (apps.rs:87-88); `start_program` routes it into `run_chain` (wm.rs:2982-2995); empty chain returns Failed/NoEditor, no spawn, no status file, no thread (launch.rs:293-299), text "Cannot open Notepad" + "No text editor installed (for example mousepad)." (launch.rs:117). Fallback recursion at launch.rs:360 only runs with a non-empty rest, so no double notice. Nothing waits, nothing hangs.
3. N-6: PASS with a caveat. fill_pool drops terminal programs and titles with "(" (startmenu.rs:976-990), tested (startmenu.rs:1453-1470). Only the left-column fill is affected; All Programs and user `programs` are untouched, so no app vanishes from the system, it is merely not auto-pinned.
4. Packaging: PASS. PKGBUILD optdepends (dist/arch/PKGBUILD:19), RPM recommends (crates/chora/Cargo.toml:133), docs/configuration.md:2221, docs/install.md:43-46, known-issues xp-23 (docs/known-issues.md:35). Podman/CI image lists no mousepad (dist/fedora-build-requires.txt); not needed, the gate fakes mousepad.
5. Test: gate-xp-notepad-gui-only.sh asserts for real: nano is the only editor (vim/vi stripped from PATH, EDITOR=nano, tests/gate-xp-notepad-gui-only.sh:20-25,35), a real click on the menu item, the exact notice via D-Bus monitor (:98), and an unchanged surface list after 5 s (:99-100); second cell: fake mousepad window opens (:104-112). Wired into xp-notepad step (tests/merge-gate.sh:169).

## Findings
- MINOR-1 startmenu.rs:~988 `!node.title.contains('(')` is broader than the stated target. Legit titles with parentheses (for example vendor-style "Name (Beta)") are not auto-pinned; no common GUI app (Firefox, Files, GIMP, LibreOffice, Calc) has one, so not MAJOR. Prefer matching the raw suffix `\s\(\w+=` or a trailing "(key=value)".
- MINOR-2 startmenu.rs:~989 terminal test is `c.starts_with(terminal)`, a string prefix: with terminal "foot" a GUI command like "footage" would be dropped. Compare the first word.
- MINOR-3 Behaviour change for users who relied on nano/EDITOR: Notepad now says "no text editor" until they set `editor: foot -e nano`. Documented (configuration.md:2221, xp.yaml), but no first-run hint in the notice.
- MINOR-4 Gate has no end-to-end cell for a configured terminal editor (`editor: foot -e nano` opens a window); only the unit test in apps.rs covers it.
- COSMETIC-1 tests/gate-xp-notepad-gui-only.sh:8 usage line names gate-xp-notepad-fallback.sh and the SHOTS file name; copy-paste leftover.
- COSMETIC-2 Notepad is now always listed, also with no editor (reverses "Without any editor the menu has no Notepad"); docs updated, fine.
- Security: none found; no new shell interpolation (command strings unchanged, NO_EDITOR is a constant).

## Refactor proposals
1. startmenu.rs `pinnable`: parse the first word of the command and the raw-suffix pattern once; return a small enum (Pin, Terminal, Raw) - S.
2. apps.rs: fold `editor` into `editors().chain.first()` (builder's proposal 1 agreed) - S.
3. Move the three gate-xp-notepad*.sh harness copies to gate-lib.sh - M.
