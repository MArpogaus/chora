# Review of tour v7 (independent, read only), 2026-11-17

Build 7e06669 (session branch). Inputs: reports/media-r5.md, media/2026-11-17/tour-v7/timeline.md and the 11 stills present in stills/
(the timeline and report cite 13: 01-02-start-menu and 01-02b-no-editor are NOT in the repo; MP4s not available). I viewed
01-02c, 02-25, 02-26, 02-27, 03-04c, 03-07, 03-12, 04-12, 04-13, 05-10. Rows without a still below rest on the recorder's timeline
and are marked "timeline only". Code read from origin/claude/autonomous-dev-session-3w-i33pg7 (S), claude/construct (C), claude/media (M).

VERDICT: MERGE-OK (computed, rule 7): no open BLOCK, no open MAJOR. Both recorder disputes resolve to the harness, not the product.
Caveat: the honest "no editor" notice has still not been seen in a tour capture; its evidence is the gate cell only (see F-1). Re-record that act.

## Ruling F-1 (recorder: new MAJOR): harness artefact, not a product bug
- The notice exists and is sent: `plugins/chora-xp/src/launch.rs:117` and `:840` text "No text editor installed (for example mousepad)." for
  `Why::NoEditor`; produced only when the Notepad chain is EMPTY (`launch.rs:begin`, `if chain.is_empty()` returns `Change::Failed{NoEditor}`
  at once); sent by `wm.rs` `tell()` (~line 3025-3055) through `self.notifier.show("chora-xp", ...)` on org.freedesktop.Notifications
  (same path as "Starting Notepad", which the tour DID show, so the notify path and chora-notify work).
- The tour showed "Starting Notepad" (Change::Starting, `launch.rs:~472`, only emitted for a started entry). So the chain was NOT empty:
  some editor was still found on PATH and started. That editor is lowriter. Cause: `demo/tour/launch.py` `path_without_writer()` (M) builds a
  link farm of /usr/bin without lowriter and swaps only the PATH entry spelled "/usr/bin". In this container `/bin -> usr/bin` and PATH ends
  `...:/usr/sbin:/usr/bin:/sbin:/bin` (checked: `/bin/lowriter` and `/bin/soffice` exist). `/bin` is left in PATH, so `apps.rs editors()`
  (`on_path`, `plugins/chora-xp/src/session.rs:397`) still finds `lowriter` (first of EDITORS, `apps.rs:~60`). The editor runs, no window
  opens, and the chain waits.
- Why silent for 14 s (and 48 s in the recorder's probe): `launch.rs` `GIVE_UP` = 30 s (line 18); before that nothing is said. At 30 s,
  `check()` treats a start whose pid is still `alive` as served (`if alive { Change::Done ... served }`), so soffice staying alive in
  headless mode yields NO failure notice at all. That is F-2 below, a real (small) product gap, but it is not F-1.
- Gate difference: `tests/gate-xp-notepad-gui-only.sh:20-27` removes lowriter/mousepad/etc. from a /usr/bin link farm and sets PATH to
  `$bin:$work/fakes:$work/path` only (no /bin), and sets `editor:` empty; it listens with `busctl monitor` (line 56). So the gate has no
  lowriter path and the chain is empty. Not a "no chora-notify" difference.
- Fix is in the harness: filter every PATH dir that resolves to /usr/bin (or build PATH as farm only, as the gate does), then re-record
  01-02b. Evidence rule: claim "notice shown" needs capture; the gate's `SHOTS` capture `xp-notepad-no-gui-editor.png` can serve meanwhile.

## Ruling N-1 drag-back: script fault in v7 again; product plain-drag is untested, not shown broken
- Evidence from stills 04-12/04-13: the pane is the large skewed "Sunset" quad. Its top edge runs from about (600,172) to (1255,127). The v7
  script (M `demo/tour/v7_construct.py`, back-prep act) presses at `(pr[0]+pr[2]/2, pr[1]+10)` of the pane's BOUNDING rect: x = ~928,
  y = ~137. The quad's top edge at x=928 is y = 172 - 45*(328/655) = ~149. The press point is ~12 px above the pane, on empty space.
- Code: `modes.rs:655 press_on_a_pane` takes a press only when `pane_at(position)` (node LOOK) or `pane_before_cube` hits; otherwise the press falls
  through, and with a cube open `input.rs:396 press()` starts no drag (`else if self.cube.is_none()`). Nothing moves, nothing drops, 04-13 equals
  04-12 (identical but for the pointer glyph over the xp face). This is the exact failure construct r3 diagnosed in v6 (drag from empty
  space), reproduced by a different aim error. Not a product bug on the evidence.
- Caveat for the PM: no gate cell covers a PLAIN drag-back. `tests/gate-construct-pull.sh:221-239` ("behind" case) drops with Super held
  (lines 227-232), as does the fixed `demo/tour/ch6.py` (C: `mods=("Super_L",)`). The doc comment of `press_on_a_pane` says a plain drag
  works ("a plain drag on a floating plane pulls a window out: a drop over a face gives the window back"; `release()` ->
  `drop_on_a_face` at `input.rs:478`). So construct r3's claim "product is right" is proven for Super only; plain is by reading only. The
  recorder's "v6 held Super: a script slip" is also wrong in the other direction: Super is what the gate uses.
- Action: fix the aim (use the quad centre: bounding-rect centre of the pane, not top+10, or read its mesh), re-record; add a plain-drag
  variant of the gate cell. Until a capture shows a plain drag landing on the face, N-1 stays MINOR (a Super-drag works, gate PASS).

## v6 MAJOR rows
| id | v7 | evidence |
|---|---|---|
| j1-5 Print notice / bar Screenshot | FIXED | 02-25 notice "Screenshot saved /root/Pictures/Screenshots/2026-11-17_09-28-16.png" after Super+Print; 02-26 power menu Lock, Log out, Restart, Shut down, Screenshot; 02-27 notice with a different stamp (09-28-34), so the mouse path made its own file |
| j1-2/N1 Notepad | FIXED in product, honest notice NOT SHOWN in the tour | 01-02c: a real "Notepad" window (stand-in editor) with taskbar entry, no nano, no terminal. Notice path: gate cell `gate-xp-notepad-gui-only` (passes per recorder, not re-run here) plus ruling F-1. Tour must re-record 01-02b |

## v6 MINOR rows and N-1..N-7
| id | v7 | evidence |
|---|---|---|
| xp chat "Working" + Stop after answer | NOT SHOWN | not re-checked (r5) |
| mc dark "No helper" with helper row | NOT SHOWN (v6 FIXED) | no still; timeline says still fixed |
| mc Super+a row overprint | OPEN | 03-04c: "Chat with a helper" printed over the Super+a description in the KEYS list |
| mc dark launcher raw entries (j1-6) / N-2 | OPEN | 03-12: "Foot Client", "Foot Server", "ImageMagick (color depth=...", "UXTerm", "XTerm", five LibreOffice entries, "Screenshot of an area" |
| handover bands / tiling black no bar (N-5) | OPEN | timeline only (05-01, 05-08 not in repo; 05-08-drag-mc-out-L present but not judged) |
| handover Start menu stays open | OPEN | 05-10: Start menu over the Notepad window while the letter is typed |
| mc wide page clock vs pill | OPEN | 03-07: "09:29" is cut off by the "No helper" pill (clock reads "09:2" then pill) |
| N-1 drag-back | OPEN, MINOR, script (ruling above) | 04-12 vs 04-13 |
| N-3 mc light "No helper" while speaking | OPEN | 03-04c: "speaking" plus pill "No helper" and CHAT "No helper" |
| N-4 mc dark "3 AGENTS" empty | NOT SHOWN | r5 did not re-check |
| N-6 xp Start menu Vim/ImageMagick next to Notepad | OPEN, wider | 05-10 start menu now lists LibreOffice and LibreOffice Calc beside Notepad (no Vim in this one); 01-02-start-menu missing from repo |
| N-7 construct use mode no HUD | UNCHANGED (note) | timeline 3:01 |
| rv5-1 power by mouse | FIXED | 02-26 power glyph opens the menu |
| rv5-2 mc leg of the handover | FIXED (timeline only) | 3:58-4:04 |
| rv5-3 helper in construct HUD | FIXED | 04-12 HUD: "Programs Helper Keys Lock Power Screenshot Volume 40%" |
| N2 step back after pull | FIXED | 04-12 stepped back shows cube, xp face and the pane |
| j1-4 separate pills | FIXED | 02-26/02-27: "No helper", "Volume 40%", "Connected" separated |

## New findings
| id | sev | finding |
|---|---|---|
| F-1 | not a finding (harness) | See ruling. Retract the MAJOR |
| F-2 | MINOR (product) | A Notepad editor that stays alive with no window (lowriter headless, or a launcher that hands off) is declared served after `GIVE_UP` 30 s (`launch.rs check()`, `if alive`): no notice, no fallback to mousepad. Person with LibreOffice but a window failure sees only a vanished "Starting Notepad". Candidate for xp-polish; needs a red test |
| F-3 | MINOR (harness/evidence) | Report cites 13 stills; 01-02-start-menu and 01-02b-no-editor (the stills that carry j1-2 and N-6) are absent from the repo. Rule 7: those claims have no capture here |
| F-4 | MINOR (harness) | The tour's "Notepad" stand-in is a foot window running `cat`; the 01-02c/05-10 windows show a text caret in a foot terminal with an xp frame. Fine as a stand-in, but the handover mc leg ("7 NOTEPAD" bare page) is unreviewed |
| F-5 | MINOR (product, seen in 02-25) | The Print notice shows the full path in large type, wrapped over three lines; works, a short "Saved to Pictures/Screenshots" is friendlier. Note not a defect |

## Why MERGE-OK
Open MAJOR count 0: j1-5 FIXED with three stills; j1-2/N1 fixed in product (window opens, no nano) and the missing notice is the harness (PATH leak of /bin), with a passing gate cell for the notice itself. The recorder's own "lean MAJOR" for N-1 is not supported: the aim error explains stills 04-12/04-13 and a Super drag is gate-green. Open MINORs (overprint, launcher entries, clock vs pill, handover tiling bar, Start menu left open, N-3, N-6) are queued behind RC rules (rule 6: no MINOR work on a look with an open MAJOR; none is open).

## Refactor proposals (harness)
1. `demo/tour/launch.py path_without_writer`: build PATH from the farm only (as `gate-xp-notepad-gui-only.sh` does) or drop every PATH dir whose realpath is /usr/bin; add an assertion `command -v lowriter` fails before the xp chapter starts, so a leak is a harness error and not a tour finding (S).
2. Share one `no_gui_editor()` / stand-in `mousepad` helper between launch.py, multi.py and the gates (gate-lib.sh farm code is duplicated in three gates) (S).
3. `demo/tour/v7_construct.py`: aim at the mesh centre of the pane (`S.rect` gives a bounding box of a skewed quad); log "PANE hit ok" by querying `pane_at` or by checking the pane moved after the press, and fail the chapter if the owner did not change (S).
4. Add a plain-drag variant to `tests/gate-construct-pull.sh` so the tour and the gate test the same gesture (S).
5. Per-act automatic proof: `t.act(..., expect=callable)` so an act whose expected state (notice on bus, owner changed) is false is flagged in timeline.md instead of by a reviewer (M).
6. Commit the cited stills with the timeline (the 500 KB hook allows small PNGs); a check that every still named in timeline.md exists (S).
