# kit3 r1 (F2): kit::keysheet, one key list for all four looks

Branch claude/kit3 from 888dfb60; merged claude/kit2c (8eabeda0) and the session branch (4548feb3, trains 60/61). Head 5b788c72.
Commits: a7916c4b (red), b8872054 (keysheet and the four ports), c64752fa (cut gate: xp cell reads the table at run time), e847eeb0 (doors cells spell keys as kit), a77a1fb1 + 5b788c72 (known-red row tiling I5 and its selftest go).

## Done
- `chora_kit::keysheet`: `rows(bindings, language, named)` gives one `Row {action, words, chords, binding}` per wording (System rows first in `System` order, then the look's); `words(&System, Language)` one wording per action, English and German; `caps`/`chord` one spelling of a key (glass caps, `drag` for BTN_LEFT, thin space before an arrow); `give_row`. `named` lets a look name its own actions and give one binding its words (tiling `labels`).
- Spec: `tests/keysheet/words.table`, read by the kit cells, the xp help cell and the acts. Cells red in a7916c4b: kit `tests/keysheet.rs` (6), act `the key list of LOOK speaks the keysheet` in mc, tiling, construct (and xp, after the session merge made `texts` of xp's `state.get` readable), xp `help::tests::the_key_list_speaks_the_keysheet`. Red failures seen: mc `Super+Shift+Escape`, tiling "All keys" missing, construct "Power menu" missing, xp "Escape"/"Left" spelled raw.
- mc keys.rs `words()` and `volume_order` gone; the System word tables of tiling (`system_words`, Close/Drag, volume row words), xp (`LINES` text for System lines) and the construct help (hand rows) gone; tiling's `spell` and xp's `name` use kit. Three rows in tools/copies.toml at 0; docs/api.md names the rows.

## Behaviour that changes (one wording, one spelling)
- Wording: construct "Show these keys" -> "All keys", "Open the power menu" -> "Power menu", "Close the pane ahead" -> "Close window", "Give it to X" -> "Give window to X"; tiling "Show this list" -> "All keys", "Screenshot" -> "Take a screenshot", "Terminal" -> "Open a terminal", "Run a program or a typed command" -> "Start a program or run a command" (default `labels` of tiling.yaml; the I5 overprint of tiling went with the long word, row removed); xp "Open this help" -> "All keys", the launcher, power, cube rows likewise, give row "Give window to a look" at `Super+Alt+1-4`.
- Spelling: letters in capitals everywhere (`Super+J`, was `Super+j` in tiling and mc), `Esc`, `Super+Return` (tiling had `Enter`), `/`. Gates updated: gate-tiling-keys, gate-tiling-keycaps, gate-construct-hud, doors cells keys-names/keys-door, gate-mc-column.
- Costume kept: tiling still joins the three volume keys in one row (a gap test at 696 px would fail with two more rows); the act cell counts each part of a " / " row.
- mc passes English to kit (its column words are English only); construct and xp too.

## Gate and coverage
Merge gate once on 5b788c72: `tests/merge-gate.sh --jobs 1`, all 41 steps PASS, log dir /tmp/merge-gate.NgkzMY (lane container only), copy /tmp/claude-0/gate-kit3-final.log. Earlier runs stopped on real findings, all fixed: cut-check asset (include_str out of the crate), doors keys-names (spelling), acts (known-red row that went green, selftest cell of it).
Kit coverage (`XDG_RUNTIME_DIR=<0700 dir> RUSTC_WRAPPER= cargo llvm-cov -p chora-kit --features chora-kit/testing --summary-only`): kit 90.76 % lines, keysheet.rs 89.31 %.

## Open
- mc I5 (net3-m1): "Start a program or run a command" wraps over "Chat with a helper" in the mc column; still known red (slice KIT3 in its row). It needs a layout fix in mc (row height for a wrapped word) or a shorter wording for all looks; not done here (no new wording decisions).
- cut-check reports `Event::FocusChanged outside kit: grew to 9, max 8` (already on the merged base, not from this slice).

## Refactor proposals, ranked
1. tiling `ORDER` ranks rows by English words, so the labels of the default file and kit must agree by hand; rank by action (and label only as a fallback) once `Row` carries an order key from kit (S).
2. Give the look a `group` in `Row` (kit order by task: programs, windows, system) so construct, tiling and xp stop keeping group tables of their own (M).
3. mc, construct, xp pass `Language::En`; take the session language through one `Language::of_session` in each costume once their own words are translated (S).
4. xp `LINES` has empty text slots for System lines; replace by `Option<&str>` (S).
5. The three volume keys as one row in tiling is a costume rule; a `Row` merge hook in kit would let any look do it (S).

## Review r1

Review of claude/kit3 head 5b788c72 (F2, one round). Verdict: **FIX** (1 MAJOR, 5 MINOR, 2 NIT; no BLOCK).

Checks:
1. Red first: a7916c4 is older than b887205 (`git log --oneline afeebaf9..5b788c72`). OK.
2. keysheet.rs builds rows from the bindings; one wording per System action (EN, DE), one chord spelling; no new feature, key or System variant. `cargo test -p chora-kit --features chora-kit/testing --test keysheet`: 7 pass (the name filter `keysheet` matches 0 tests, use `--test keysheet`). OK.
3. mc `words()` and `volume_order` gone; no look keeps a System word table. But two looks keep their own row table, see rk-1. copies.toml KIT3 rows at 0; cut-check REPORT lines 19 on afeebaf9, 20 on head (the extra line is rk-3); the 7 `variants` lines with `--base afeebaf9` are moved match arms, not new variants.
4. `cargo test -p chora-mc -p chora-tiling -p chora-xp -p chora-construct`: all green (199, 141, 156, 187). No renamed key or lost row in the shipped configs. The visible wording changes go beyond capitals/Esc/Return, but all are named in the report and follow from item 1 (one wording per action).
5. known-red.toml: tiling I5 row and its selftest gone, mc I5 (net3-m1) stays; docs/api.md names the rows. OK.

Findings:
- rk-1, MAJOR, plugins/chora-construct/src/config.rs:498-620 and plugins/chora-xp/src/help.rs:92-108 (`LINES`, `rows`): construct and xp still keep their own row tables (hand rows, `LINES`) and take only `keysheet::words`/`chord`; `keysheet::rows` has two callers, mc and tiling (`grep -rn "keysheet::rows" plugins`). Brief items 2 and 3 ("every look's own row table goes", "only rows come from kit") are not met, and the report ("construct help (hand rows) gone") overstates it. Fix: port construct and xp to `keysheet::rows` with `named` for their own actions, or have the PM accept it and record the two tables as a follow-up (refactor proposal 2).
- rk-2, MINOR, plugins/chora-xp/src/help.rs:170: the give row shows only when all four give keys are held (was: the span first..last of the held ones); with one key held by another client the row is lost. Fix: show the row when any is held, or say in docs/api.md that it needs all four.
- rk-3, MINOR, tools/cut-known.toml (xp names chora-chat): the count fell from 6 to 4, so cut-check reports "lower max from 6"; the only REPORT line that grew. Fix: set max = 4.
- rk-4, MINOR, crates/chora-kit/src/keysheet.rs:give_row: German give row "Fenster an einen Fenstermanager geben" is not in tests/keysheet/words.table and differs from `System::Give` "Fenster geben an {}". Fix: put both wordings in the table and the cell, one wording for both.
- rk-5, MINOR, plugins/chora-tiling/src/keys.rs `spell`: it strips the thin space that `keysheet::chord` puts before an arrow, so tiling spells a chord differently from the other three looks, and docs/api.md says the thin space is universal; no cell sees it (the act splits on "+"). Fix: keep the thin space in tiling, or name it as a costume rule in docs/api.md.
- rk-6, MINOR, plugins/chora-tiling/src/keys.rs `rows`: the old `binding.contains("MOD")` filter is gone, so a user binding without MOD, and an unlabeled Terminal/Lock/Power/Screenshot binding, now get a row (the shipped yaml is unaffected); not in the report. Fix: name it in the report and docs/configuration.md.
- rk-7, NIT, plugins/chora-mc/src/keys.rs `all()`: the volume rank `3 + usize::from(*change != Change::Up)` ties Down, Mute and Set and relies on the stable sort and kit's order. Fix: rank by the position in the sheet, or comment it.
- rk-8, NIT, tests/acts/looks/common.py `EVERY_LOOK`: the act checks nine English wordings; close, terminal, give and German are checked only in the kit cells. Fix: add the actions a look binds to the act, from the table.

## r2

Head 04ea5851 on claude/kit3. Gate (`tests/merge-gate.sh --jobs 3`, once, on the head): all steps passed.
`tools/cut-check` REPORT lines 19 (was 20 on 5b788c72; 19 on afeebaf9): not grown.
Red first: 7c44ceb5 (construct, xp, rk-2) and 838c3e35 (rk-4, rk-5, rk-6) fail before their fixes.

- rk-1 done, b43e555b (xp) and 19f5e497 (construct). Both tables go: xp `LINES` and `about` are
  replaced by `own_words` and `place` (the costume: group and order), construct `help_groups` takes
  every bound row from `keysheet::rows` (its own actions named by `Own::words`; the four give rows are
  fed as `Super+Alt+n` bindings of `System::Give`, so their words are kit's). Rows of the Walk group
  still join several actions (W A S D), as before. Keys that no binding holds (move modifier,
  `fly_out`, `Alt+y/n`, `Ctrl+period`, the cube keys, Escape) stay rows of the look. I dumped the rows
  of the shipped config before and after in both looks (construct: 3 binding sets, cube on and off;
  xp: all held, one key not held): identical, no lost row, no renamed key. A new side effect, named:
  an action the shipped file does not bind (`volume: {set: N}`) now has a row in both looks.
- rk-2 done, b43e555b: the xp give row shows while any give key is held.
- rk-3 done, 8e7ded04: tools/cut-known.toml xp names chora-chat max 4.
- rk-4 done, 8e7ded04: `give_row` speaks `System::Give` in both languages ("Give window to a look",
  "Fenster geben an einen Fenstermanager"); the table has a `giverow` line, the kit cell checks it.
- rk-5 done, 8e7ded04: tiling keeps the thin space (`spell` is gone); docs/api.md is true as written.
- rk-6 done, 8e7ded04: tiling shows a binding only when it names `MOD`, or when it is a media key of
  the volume, as before; no change to name.
- rk-7 done, a642f7ae: mc ranks the volume rows by their place in the sheet. rk-8 not done (the act
  checks the nine words every look binds; the other actions differ by look and need a per-look list).
- docs/api.md names the four looks (eb549825).

Merge: origin/claude/autonomous-dev-session-3w-i33pg7 merged (04ea5851). reports/integ.md does not
exist on the log branch, so claude/integ is not merged; kit5b and kit7 stay for lane integ.
Resolutions: crates/chora-kit/src/testing.rs (same change both sides, only reordered: theirs);
plugins/chora-mc/src/region.rs (their split into region/ taken, my one line `keysheet::chord` for
the chat key re-applied in region/items.rs); docs/api.md (their wording "Such a look applies");
plugins/chora-xp/src/help.rs (my `rows`, their `fallback(binding, action)` signature);
tests/acts/known-red.toml (their I3 and pager rows kept, tiling I5 net3-t1 stays gone, mc I5 stays);
tests/acts/looks/construct.py (their `cube_acts` and helpers, my `keysheet_acts` call before them);
tools/copies.toml (their move of the kit2d rows, my three KIT3 rows kept).

## Review r2

Review of the r2 fixes on claude/kit3 head 04ea5851 (F2, one round; commits 7c44ceb5 b43e555b 19f5e497 838c3e35 8e7ded04 a642f7ae eb549825). Verdict: **MERGE-OK** (0 BLOCK, 0 MAJOR, 1 MINOR, 2 NIT).

Checks:
1. Red first: 7c44ceb5 and 838c3e35 are older than their fixes (`git log --oneline 5b788c72..04ea5851`); run on their own commits the new cells fail (construct and xp row cells, xp give row, kit give wording x2, tiling MOD filter and thin space). OK.
2. rk-1 closed: `LINES`, `about` and the hand rows are gone; xp `rows` and construct `help_groups` draw `keysheet::rows`. I dumped the rows of the shipped config on 5b788c72 and on 04ea5851 (xp: all keys held and Super+e not held; construct: cube off and on) with a throwaway test: both dumps identical (39 and 57 lines), no lost or renamed row. `cargo test -p chora-xp -p chora-construct -p chora-kit --features chora-kit/testing` green; `cargo test -p chora-mc -p chora-tiling --features chora-kit/testing` green.
3. rk-2 (xp give row on any held key, cell), rk-3, rk-4 (one wording from `System::Give`, `giverow` line in the table), rk-5 (thin space kept, `spell` gone), rk-6 (MOD or media key of Up/Down/Mute, as before: same set as the old `volume_row`), rk-7 (rank by place; kit order Up, Down, Mute, Set) closed as the r2 report says. rk-8 open: the reason holds (the nine words are what every look binds; the other actions differ by look); not a finding.
4. `tools/cut-check`: 19 REPORT lines, not grown (r1 head: 20); cut-known.toml xp names chora-chat max 4 = the 4 lines counted. With `--base 5b788c72` the new `variants` lines in construct (Change::Set, System::Volume in config.rs) are the named Set row, not a new variant.
5. No new key, feature or System variant. Side effects named by the lane (xp and construct now show a row for `volume: {set: N}`, and xp for Run and Terminal, when a file binds them) are not in the shipped files.

Findings:
- rk-9, MINOR, plugins/chora-construct/src/config.rs:460 (`bound`): the row is found by `r.action == action`, but `keysheet::rows` merges bindings of one wording and keeps the action of the first binding, so with `run: fuzzel...` bound on a key that sorts before the `launcher` key the Launcher row has no keys in construct's help (the shipped file binds only `launcher`; before r2 `bound` matched every binding by its action). Fix: take the chords of the row whose `words` equal `keysheet::words(system)` (or of every row that contains a binding of the action) instead of matching the action of the first binding.
- rk-10, NIT, plugins/chora-construct/src/config.rs:609: the Set rows use `chord(&r.binding)`, the first binding only; a second key for the same `volume: {set: N}` merges into the row's `chords` and is dropped. Fix: use `r.chords` as `bound` does.
- rk-11, NIT, docs/api.md:490: the added sentence is appended to a wrapped line (144 characters; the paragraph wraps at about 88). Fix: rewrap the paragraph.
