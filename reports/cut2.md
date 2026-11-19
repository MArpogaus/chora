# cut2 r1 (F6): cut gate blocking, asset moves, look.theme event

Lane cloud, branch `claude/cut2` from dc4bc46d, head 29b2fd25 (pushed). 6 commits, 33 files.

## Commits
- 4643e772 test: red cells. Cut-check selftest (link out of the crate, costume of another look in huds, asset outside the crate, extra workspace dependency, plugin name beyond a known count, new match arm with --base), core cell `a_look_names_itself_with_the_shared_look_events`, assistant cells (`only neutral in huds`, `folder of each look in its own directory`, `a fifth look gets a costume from its own directory`). Red output: selftest 8 cells FAIL; 3 assistant tests and the core test panic.
- eee4de7c refactor: tiling HUD to `plugins/chora-tiling/hud`; the assistant keeps `neutral` only; the three links (mc, construct, xp) deleted; `folders.rs` source roots are `huds`, `plugins/chora-{}/hud`, `flavors/{}/assets/assistant`; PKGBUILD, crates/chora/Cargo.toml, gate scripts (new tests/hud-folders.sh), docs.
- 7ce5f312 feat(core): any client may emit `look.theme` and `look.view`; the core sets `data.look` to the emitter (no spoofing). Deviation, see below.
- 773031b4 feat: kit `look_theme_event`/`look_view_event`; the assistant subscribes to `look.*` and keeps the old names; tiling emits `look.theme` beside `tiling.theme`.
- 32a337ab ci: cut-check blocks packaging, assets (include path, link, extra huds entry) and names beyond `tools/cut-known.toml` (counts only fall, "lower max" reported); `--base REF` reports new match arms on a pub kit enum; merge-gate passes the merge base.
- 29b2fd25 fix: the first version drew the look folder as a fallback and broke hud cells that force a folder (`mc-chat-drawer`, `construct-host-hud`, `construct-chat-panel`, `construct-chat-long-task`: MISMATCH). It is now the rule `{folder: '@look'}`, last in the shipped assistant.yaml; a user setting without it stays strict.

## Evidence
- Green: selftest, assistant 238 tests, core look test, kit hud tests, packaging-files and its selftest, `tools/cut-check` (8 crates, 18 reported, exit 0).
- Merge gate on 29b2fd25 (`--jobs 3`): red-first to cube PASS (including hud, matrix, layout, tiling-audit, acts, cube); `pull` FAILED once under load ("185 px wide for 200"); `--only pull` alone PASS; the 13 later steps (placed to e2e) PASS with `--only ... --jobs 2`. This is not one clean full run. Logs: /tmp/claude-0/gate/merge-gate.KrWl8B (full), .fEaXho (pull), .2RAEe0 (rest); the first full run (before 29b2fd25) failed at `hud`, fixed by 29b2fd25.
- Not run: lychee and vale (skipped as in CI), tiling `name_theme` has no unit test (no FakeCore for tiling); the hud and tiling-look gates cover the emit end to end.

## Deviations and open points
- `events.emit` allowed only `NAME.x` for client NAME, so `look.theme` was impossible without a core change. I added the two names as an exception (LOOK_EVENTS, ipc.rs). It amends D65; docs/api.md says so. The PM may want a decision line.
- Only tiling emits `look.theme` now. construct, mc and xp keep their old names (the assistant reads both), per the brief (no change in construct or xp). One line each in their next port; then the old names can go.
- The cut is not yet "no breach": `tools/cut-known.toml` counts 5 asset includes (chora-chat 1, chora-voice 3, chora-xp 1) and 16 name pairs. They block only when they grow. docs/decisions.md D103 still says "reported"; not edited (PM).
- Variants are found by single-line `Enum::Variant ... =>`; an arm spanning lines is not seen.

## Refactor proposals (ranked)
1. Names (16 known rows): construct config.rs:317-324 and mc config.rs:323 list every look by name; a kit look registry from `look.*` removes most rows. M.
2. Assets (5 rows): voice chat.rs:`include_str!` of README and docs, chat opener.rs desktop file, xp launch.rs fuzzel.ini; move each into its crate (build copy). S each.
3. construct, mc, xp emit `look.theme`/`look.view`; then delete `theme_event`/`view_event`, the five old names in assistant EVENTS and `look_event`'s legacy branch. S.
4. Shipped `huds` rules for construct, mc, xp, tiling become redundant once all emit `look.theme` (only the construct-pane rule stays). S.
5. `folders.rs` roots as strings with `{}`: a typed `Root` enum. S.
6. cut-check: merge `copies.toml` and `cut-known.toml`; a Rust-aware arm scan for multi-line patterns. S.
7. tiling `name_theme` test once tiling has a FakeCore (WM1). S.
