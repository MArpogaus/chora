# xp-polish r7 (rc 1): Notepad is never a terminal editor

Branch claude/xp-polish at 2179966 (session branch aa8aa8c merged, r6 included). Sonnet, one round, 2026-11-17.

## Done
| item | result | cell |
|---|---|---|
| j1-2/N1 Notepad opens only an editor with a window | fixed | gate-xp-notepad-gui-only.sh (new, in the xp-notepad step); unit tests in apps.rs |
| no GUI editor: notice, no window | fixed: "Cannot open Notepad" / "No text editor installed (for example mousepad)." | gate-xp-notepad-gui-only.sh cell 1 |
| a fake GUI editor on PATH opens its window | ok | same gate, cell 2 |
| packaging | mousepad is an optdepend (dist/arch/PKGBUILD) and a recommend (crates/chora/Cargo.toml, the RPM); docs/configuration.md, docs/install.md, known-issues xp-23 say so | merge-gate package-deps, package-lists |
| N-6 Start menu pins no Vim or ImageMagick | fixed in the fill of the left column (no terminal program, no title with a raw "(...)" suffix) | unit test startmenu.rs; no gate cell, no capture of the menu |

Behaviour: nano, vim, vi, VISUAL and EDITOR are gone from the Notepad chain. A terminal editor serves only as the setting
`editor` in xp.yaml (for example `foot -e nano`). With no editor with a window, the Notepad entry stays and a click says so.
apps::editors lost its terminal and env parameters.

## Evidence
- Red (before the fix, uncommitted: the pre-commit clippy hook refuses a test commit that does not build or leaves a dead
  variant, so tests and fix are one commit): cargo test -p chora-xp: 4 FAILED (both editor tests, "left: Some("foot -e nano") right: None";
  an_empty_chain_...; the fill test "left: [Notepad, Calc, Vim, ImageMagick ...]"). Gate cell on the old chora-xp (aa8aa8c build):
  FAIL "no notice Cannot open Notepad / No text editor installed" (it opened nano in a foot).
- Green (2179966): cargo test -p chora-xp 188 ok; clippy -D warnings clean; both gate cells ok.
- Merge gate, one full run on 2179966 (after `git fetch --unshallow`; the first start stopped at 2d-guard, no base): all steps
  passed, incl. xp-notepad, hud, layout, tiling-audit, keys, pixels, e2e ("all steps passed; no --push").
- Capture: media/2026-11-17/xp-polish-r7/xp-2179966-2026-11-17-notepad-no-gui-editor.png (look xp, build 2179966, 2026-11-17).

## Open
- The podman/CI image (dist/fedora-build-requires.txt, ci.yaml) lists no runtime editor; no change there.
- A configured editor that is a terminal editor is not wrapped in a terminal; the user writes `foot -e nano`.
- The fill filter drops every title with "(", also a legitimate one; the raw-entry names themselves (N-2) stay.

## Refactor proposals
1. plugins/chora-xp/src/apps.rs `editor` + `editors`: one caller each is left (`editor` only feeds `load`); fold `editor` into `editors().chain.first()` - S.
2. plugins/chora-xp/src/startmenu.rs `fill_pool` + `pinnable`: decide "shown to the user" once in chora-kit desktop_entry (Terminal=, raw-suffix names, N-2) for all looks - M.
3. tests/gate-xp-notepad*.sh: three scripts repeat ~60 lines of harness (fake PATH, dbus, notice helpers); move to gate-lib - M.
