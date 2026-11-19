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

## Review r1

Lane cloud, read-only on code, claude/cut2 head 29b2fd25 against dc4bc46d. Verdict: **MERGE-OK** (0 BLOCK, 0 MAJOR, 2 MINOR, 1 NIT, 1 pre-existing).

Checks:
1. Core exception: minimal and safe. Only the exact names `look.theme`, `look.view` pass; `data.look` is always overwritten with the emitter (a spoofed `look` in the cell is replaced); a non-object data is refused; emitters in a core namespace are still refused first; size is measured after stamping and the per-name rate applies. A confined connection stamps its own name, and `@look` needs that name to be the space or WM of the context and a folder of that name, so it cannot steer another look's costume (name squatting on the main socket is pre-existing, D33). `a_look_names_itself_with_the_shared_look_events` passes. docs/api.md matches except rw-1; docs/decisions.md D65 still says "the core refuses any other name" and needs the amendment line (PM).
2. Red first holds: at 4643e772 cut-check-selftest has 15 FAIL lines over the 8 new cells, 3 assistant tests and the core test fail; selftest is green from 32a337ab; head: assistant 238 ok, kit 223+11 ok, core look cell ok.
3. Cut gate: `tools/cut-check` on 29b2fd25 exit 0 (8 crates, 18 reported); `--base dc4bc46d` exit 0, no variant reported (the diff adds no kit-enum arm). Selftest covers packaging, includes, links, huds entries, names over cut-known.toml, `--base` arms. merge-gate.sh passes the merge base to `--base`; CI runs it without `--base` (reports only, fine).
4. Behaviour: tests/hud-folders.sh lists neutral, construct, mc, tiling, xp; packaging-files.py 165 files 0 gaps, its selftest ok; PKGBUILD and Cargo.toml install plugins/chora-tiling/hud; tiling emits `tiling.theme` then `look.theme`.

Findings:
- rw-1, MINOR, crates/chora/src/ipc.rs:1071, a client named `look` takes the own-prefix branch, so it emits any `look.X` unstamped with its own `data.look`, against docs/api.md:705 ("No other `look.` name is accepted"); no consumer acts on it today; fix: refuse an event that starts with `look.` unless it is in LOOK_EVENTS, with a line in the core cell.
- rw-2, MINOR, plugins/chora-assistant/src/main.rs:356, `look_event` trusts `data.look` for every `look.` event, not only the two stamped ones, so rw-1 becomes a spoof once a third `look.` name is handled; fix: take `data.look` only for `look.theme` and `look.view`.
- rw-3, NIT, plugins/chora-assistant/src/context.rs:316, `themes` now gains a row for every client name that emits `look.theme` (main-socket names are free) and never drops one; fix: remove the row when the client leaves.
- rw-4, PRE-EXISTING, crates/chora/src/ipc.rs:2179, `a_refused_request_logs_its_client_and_code_without_its_parameters` fails under `cargo test -p chora ipc::` at dc4bc46d and at the head, passes alone (tracing callsite cache across tests); fix: build the subscriber before any callsite registers, or rebuild interest (`tracing::callsite::rebuild_interest_cache`) in `logged`.

Not run: merge gate (F21 known), gate-hud and tiling acts (covered by the lane's gate run).
