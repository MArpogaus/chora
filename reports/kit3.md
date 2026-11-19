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
