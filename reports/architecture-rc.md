# Architecture plan: deduplication and the plugin cut (input for sprint 1)

Head read: rv-arch `fe8ba6d0` (train 35). Architect persona (opus), 2026-11-17. Read-only: nothing was built or run. Evidence comes from grep/`wc -l`/`diff` on that head, and file:line refers to it. "Production lines" means lines above a trailing `#[cfg(test)]`. I mark where I am unsure. This plan is for the sprints after the tag. §8 lists only the bugs I found on the way, for the PM's findings ledger.

## 0. In brief

- Most duplication across the four looks is **mechanism inside each window manager's process**: the window lifecycle, system actions, the key sheet, the safety question, notices, launching, config, words, paths and the test fakes. That belongs in **chora-kit modules**, not in new processes. A plugin per mechanism would add IPC round trips, start-order races and z-order fights (D3), and it would buy no reuse that a library does not already give.
- **Plugins** are the right size only where a part has its own surface, its own lifecycle and the same behaviour in every look. That covers chora-notify and chora-assistant (they exist, but are not yet used by every look), and later the D73 blocks (stack, panel, desktop). Each look supplies a **costume folder**, and the shared plugin ships only its neutral one. kit `hud::When`/`Rule` (hud.rs:705–721) already does this for the assistant.
- The models to copy already exist in kit: `volume.rs` (used by all four), `launcher.rs` (fuzzel), `clock.rs`, `give.rs`, `effects`/`float`/`wobbly`/`glass`, `hud::theme_event`. Dedup means doing this for the rest.
- The clean cut is close at the crate level. Every plugin depends only on chora-scene/kit/draw (Cargo.toml checked). It **breaks at assets, packaging and hard-coded names** (§3).

## 1. Duplication map (what is copied where)

| # | function | xp | tiling | mc | construct | shared parts | target |
|---|---|---|---|---|---|---|---|
| 1 | surface lifecycle: created/changed/detached/destroyed/requested; the claim filter `to != NAME` (D55/D72); detached = free again; output choice | wm.rs:768–810 | main.rs:648–720 | main.rs:815–850 | plugin.rs:22–80, scene.rs:26 | – | kit `book` |
| 2 | system actions (lock, power, screenshot, volume, run, terminal, launcher, keys, chat) and `run_shared` with a "not installed" notice | wm.rs:3825–3864 (no Action enum; keys hard-coded) | via scripts theme.rs:788, `Action::Volume` | `Action` config.rs, main.rs:3327–3345 | `Action` config.rs, keys.rs:54–56 | kit shared.rs:57 | kit `actions` |
| 3 | the same action under different names: `Keys`/`ShowKeys`/`Help`; `Volume(Change)` vs `VolumeUp/Down/Mute`; `Run`/`Terminal`/`Launcher` | – | config.rs Action | config.rs Action | config.rs Action | – | kit `actions` |
| 4 | key sheet model: rows from bindings and how a key is spelled | help.rs:72 `name`, :130 `rows` | keys.rs:255 `spell`, :314 `rows` | keys.rs:80/122, :214 `human` | hints.rs | – | kit `keysheet` |
| 5 | safety question (confirm) | – | – | agent.rs:98–165, :838 | hud.rs:959/993, voice.rs:58 | kit ask.rs (assistant only) | kit `ask` only, drawn by the assistant |
| 6 | helper presentation (voice state, say line, question) | assistant | assistant | own agent HUD agent.rs (1,082 lines) | own voice HUD when `voice_hud: Own` (config.rs:312; hud.rs 1,829, voice.rs 351) | assistant (15k), voice mc.rs:59–210 `hud` path | chora-assistant + look costume/slot |
| 7 | notices and notifications | chora-notify over D-Bus through kit `Notifier` (zbus), wm.rs:2453/3023/3698/3863 | mako (external), `notify-send` main.rs:237 | its own `notify.push` server main.rs:1275, agent.rs:619 | hud.rs:1091 `notice` (in the HUD; no server seen) | chora-notify | chora-notify everywhere, kit `notice` client |
| 8 | launching and window arrival | launch.rs (618), session.rs:383 | 10× `Command::new` main.rs | fonts.rs 5× | cube.rs:166, wm.rs 3× | kit lib.rs:55/67/91 | kit `launch` |
| 9 | config top sections (`test_accept_injected_keys`, programs, terminal, clock, bindings) | config.rs:102–132 | config.rs:358–379 | config.rs | config.rs:232–246 | kit settings.rs | kit `settings::Common` |
| 10 | session language | – | theme.rs:707 (buggy) | – | modes.rs:104 | assistant main.rs:517, chat main.rs:73, kit words.rs:30 | kit `words` |
| 11 | runtime/config/home paths, shader files | desktop_icons.rs:905, startmenu.rs:263 | main.rs:1849, theme.rs:551/608 | main.rs:409–441 | main.rs:605–633 | kit icons.rs:76, draw theme.rs:1066, core ipc.rs:204/257 | `chora_scene::paths` |
| 12 | injected-input filter | wm.rs:424 | main.rs:425 | main.rs:815 | plugin.rs:19, input.rs:85 | notify main.rs:90, assistant main.rs:245 | kit dispatch |
| 13 | fake core in tests | wm.rs (2) | main.rs | testing.rs (best) | tests.rs | notify, assistant, kit conn.rs, voice | kit `testing` |
| 14 | HUD costume folders, kept twice | – | – | `plugins/chora-mc/hud` = `assistant/huds/mc` (`diff -rq`: identical) | `plugins/chora-construct/hud` = `assistant/huds/construct` (identical) | `assistant/huds/xp` = `flavors/xp/assets/assistant` (identical) | one copy, owned by the look |

Rows 1–4 and 7 are the largest wins I can see. Each look spends its first 60–100 lines of event handling on the same lifecycle, and each look re-invents what Lock/Power/Screenshot/Keys mean.

## 2. The cut: kit modules and plugins

Rule (from D77 and the owner, 11-17): a plugin uses only chora-scene, chora-kit, chora-draw and the live API, ships only its own assets, and can be packaged and moved alone. The pieces proposed below follow the same rule.

**Kit modules** (a library in the look's process; one look's policy stays in the look):
- `kit::book` (new, M). It consumes `Surface*`, `Output*` and `FocusChanged`, applies the claim filter and the detach-is-free rule, and keeps `Window { id, app_id, title, size, token, pid, output, decoration }` plus the primary output (non-virtual, the next one on removal). API: `Book::apply(&Event) -> Option<Change>` with `Change::{Adopt, Forget, Resized, Retitled, Requested, Outputs}`; the look matches on `Change`. Replaces the four handlers of row 1. Placement, stacking and layout stay in each look.
- `kit::actions` (new, S–M). `enum System { Run(String), Terminal, Launcher, Keys, Chat, Lock, Power, Screenshot, Volume(Change), Close, Give, Drag }` with one serde spelling, embedded through `#[serde(untagged)] enum Action { System(System), Look(LookAction) }`. `actions::perform(&mut Runtime, &System, &dyn Notice) -> Handled` covers shared scripts, the launcher, chat and volume. Replaces rows 2–3 and gives xp a `bindings` section (D7).
- `kit::keysheet` (new, S). `Sheet { groups: Vec<Group { title, rows: Vec<Row { keys, words, binding }> }> }` built from bindings plus `System` words plus look words. `keysheet::spell(binding, Language)` gives one spelling. Every look still draws its own sheet (XP dialog, tiling glass list, mc page, construct capsule).
- `kit::ask` (exists; becomes the only model): `Question` with sanitized text, `GUARD`, `buttons(lang)`, `send_answer`. Row 5.
- `kit::launch` (new, M): `Pending { token, pidfd, unit, since }`, `poll() -> {Shown, Exited, Waiting, Slow}`. Row 8; D1.
- `kit::notice` (new, S): `notice::push(conn, Notice { title, body, urgency, from })` calls `notify.push` on the live API. This replaces kit `notify.rs` `Notifier`, and **zbus leaves kit** (88 of kit's 125 unique deps; only xp uses it, wm.rs:22).
- `kit::settings::Common` (C10), `kit::words` (C12), `kit::log` (one stderr format), `kit::testing` (feature, dev-dependency only: `FakeCore` with a scene model taken from mc testing.rs), and `chora_scene::paths` (scene, because the core may not depend on kit).

**Plugins** (own process, own surface, costume per look):
- `chora-notify`: the **only** notification server in every look. It serves D-Bus `org.freedesktop.Notifications` for third-party programs and `notify.push`/`notify.clear` on the live API for plugins, emits `notify.shown/closed`, and picks placement and costume from `huds`-style rules keyed on the look (as `assistant::Config::folder`, config.rs:61). It replaces mc's own server (main.rs:1275) and construct's `hud.notice`. For tiling, mako becomes optional. Unsure: mc may need notices inside its content flow. Then mc gets a slot (`mc.notify.slot`, as `mc.chat.slot` exists) and chora-notify draws into it. Design pass M.
- `chora-assistant`: the **only** helper presenter, the safety question included. Looks provide a costume folder (they already do for mc and construct) or a slot. It replaces mc agent.rs (helper HUD and confirm) and construct `voice_hud: Own` (hud.rs Confirm/Ask, voice.rs). voice mc.rs then reports to one HUD host. **L, design pass**: mc is voice-first, and its agent view is part of its page. I am unsure that a slot carries all of it.
- D73 blocks `chora-stack`, `chora-panel`, `chora-desktop`: xp is the first user. The panel's second candidate is tiling (waybar today); stack can run inside construct panes (D67). Do them only after the kit modules exist, because the blocks are built from them. Risk: a lowest-common-denominator look. Each block ships a neutral costume, and the look ships its own.
- Not plugins: geometries stay kit modules or construct modules (D76 allows both); key sheet, actions, launch and book are libraries (reason above).

## 3. What blocks a clean cut today (breaches) and the gate

Production code and packaging (they block packaging or moving a plugin alone):
1. The assistant ships look costumes. `plugins/chora-assistant/huds/{tiling,mc,construct,xp}`. Three are identical copies of folders the looks also hold (row 14); tiling exists only there. PKGBUILD:105 installs `huds/tiling` from the assistant. folders.rs:14 `SOURCE` searches only `plugins/chora-assistant/huds`. Fix: tiling gets `plugins/chora-tiling/hud`, and the assistant keeps `neutral` only. `SOURCE` becomes `plugins/*/hud` plus `flavors/*/assets/assistant`.
2. The assistant names every look: main.rs:90–94 subscribes to `construct.theme`, `mc.theme`, `tiling.theme`, `xp.theme`, `construct.view`. A fifth look needs an assistant change. Fix: kit `hud::theme_event` (hud.rs:144) emits one `look.theme {look, theme}` (and `look.view`), and the assistant subscribes once. Keep the old names for one release.
3. construct hard-codes other plugins' binaries as defaults: config.rs:317 `cube_manager = "chora-xp"`, :321–324 `xp→chora-xp`, `tiling→chora-tiling`, `mc→chora-mc --manage-windows`. D67 wants them, but the binaries should be found at run time (installed `*.desktop`-like manager entries under `/usr/share/chora/managers/`), with a clear "not installed" state, so the construct package does not hard-depend on three looks.
4. Spawns of `chora-chat` (package voice/chat): xp launch.rs:21, control.rs:265; construct keys.rs:54; assistant chat/host.rs:167–169. That is acceptable as an optional dependency only if each look degrades honestly when the binary is missing (unverified for construct keys.rs:54).
5. services/voice chat.rs:48–53 `include_str!("../../../README.md")`, `docs/glossary.md` and `docs/api.md` from the repo root. The voice crate cannot build outside this tree, and D77 moves it to `chora-voice`. Fix: chora-scene exports the API doc text, or the voice service reads `/usr/share/doc/chora/` at run time (unsure which is better; the first keeps it compile-checked).
6. PKGBUILD is one package. :53 installs all ten binaries. The xp notify costume links into the xp theme fonts (`ln -s ../../theme/fonts`, :115), which is fine inside one package, but it must stay inside the xp package.
7. Flavor folders live outside their plugin (`flavors/xp`, `flavors/tiling`, `flavors/construct`), and `flavors/shared` (power.sh, screenshot.sh, chat entry) is read by every look through kit shared.rs:7 `INSTALLED`. That needs an owner per folder: `flavors/<look>` belongs to the look's package, and `flavors/shared` to the base package.

Test-only (they break once the repos split, not packaging): notify main.rs:356 reads `flavors/xp/assets/notify`; assistant main.rs:751/766/780/838 read `tests/hud` and `flavors/xp/assets/assistant`; chat opener.rs:111 includes `flavors/shared/applications/chora-chat.desktop`. Fix: test fixtures inside the crate.

Gate (`tools/cut-check`, in `ci-check` and the merge gate; S–M):
- Deps: `cargo metadata`. Every crate under `plugins/` and `services/` may depend on workspace members {chora-scene, chora-kit, chora-draw} only. No path dependency leaves that set.
- Assets: no `include_str!`/`include_bytes!` and no `CARGO_MANIFEST_DIR` join that leaves the crate directory (production and tests), except `flavors/<own name>` until the flavor folder moves in.
- Names: no string literal naming another plugin's binary or method prefix in production code, unless it is listed in `[package.metadata.chora] optional = [...]`. That list becomes `optdepends`/`Suggests`.
- Packaging: every file a split package installs lies under that package's crate or its `[package.metadata.chora] assets` list. The gate fails on a file claimed twice or by nobody (this catches row 14).
- Weekly, slow: build each plugin from a sparse copy (its crate plus scene/kit/draw). Unsure how hard this is with the workspace lock file.

## 4. Packages and repositories (after the tag)

- One PKGBUILD (`pkgbase=chora`, `pkgname=(...)`) and one spec with subpackages:
  - `chora` (core, choractl, session, portal, shaders, `flavors/shared`);
  - `chora-notify`, `chora-assistant` (neutral HUD only), `chora-chat`, `chora-voice` (service, quadlet, pipewire);
  - `chora-xp`, `chora-tiling`, `chora-mc`, `chora-construct`, each with its flavor folder and its costume folders for notify and the assistant.
- Meta packages:
  - `chora-flavor-xp` = xp, notify, assistant; chat optional;
  - `chora-flavor-tiling` = tiling, assistant, waybar, mako, swaylock, swaybg, kanshi, fuzzel, foot;
  - `chora-flavor-mc` = mc, voice, chat;
  - `chora-flavor-construct` = construct, assistant; xp and tiling optional, per point 3;
  - `chora-desktop` = all four.
- Repos: one repo until 1.0. Then the owner's plan (a plugin repo, later per view or user scenario) differs from D77 (four repos, xp blocks with mc and construct in `chora-plugins`). Record it as an amendment to D77 when it is decided. The cut gate makes either split mechanical.

## 5. Migration order (one merge per line; each with its test)

1. `kit::testing` FakeCore (from mc testing.rs) with its own tests. Everything after this needs in-process proof that behaviour did not change.
2. Cut gate in report mode, listing today's breaches. It turns blocking once they are fixed (steps 3–4).
3. Assets: move `assistant/huds/tiling` to tiling, delete the three identical copies, update folders.rs `SOURCE` and PKGBUILD. Test: the HUD folder lookup test per look, and the packaging smoke.
4. `look.theme`/`look.view` events with the old names kept. Test: the assistant reacts to a fifth fake look.
5. `chora_scene::paths`, `kit::words::Language::of_session`, `kit::log`: foundations, each S.
6. `kit::actions` together with `settings::Common` (= C10). One look per merge: mc, construct, tiling, then xp (xp gains `bindings`). Test: the same YAML binding gives the same `System` action in all four; old configs still parse.
7. `kit::keysheet` (needs 6). Test: one binding has one spelling in every look.
8. `kit::book`. One look per merge: mc (best tests) first, then construct, tiling, and xp last after the step-0 split of wm.rs (AR7). Test: a FakeCore script of create/detach/destroy/output-removed gives the same `Change`s.
9. `kit::ask` as the only question model, and `kit::launch` (after D1).
10. `kit::notice` plus chora-notify as the only server. zbus leaves kit. Then mc's `notify.push` server goes (design pass).
11. chora-assistant as the only helper presenter for mc and construct (L design pass).
12. Split packages and meta packages; the cut gate blocks.
13. D73 blocks, built from steps 6–10.

## 6. Other refactor items (summary; size, when)

- **AR7 xp wm.rs (L; the Fable candidate).** It has 4,251 production lines. One `impl Wm` spans 473–3868 with about 150 methods, and `struct Wm` (324–423) has about 55 fields. The clusters: panel/tray/volume 969–1050, 1479–1583, 3347–3648; keys 1849–2190; pointer/drag/handover 2175–2970; dialogs 3064–3330, 3721–3851. Step 0 moves code only (child modules with `impl Wm`, reviewed with `--color-moved`). Step 1 adds sub-structs (`Panel`, `Pointer`, `Dialogs`). Step 2 is D73. This is the best Fable item, because the D73 design doc is the deliverable.
- **AR8 mc main.rs, tiling main.rs (M).** mc: protocol structs at 93–253, `impl Mc` 493–788, `impl Plugin` from 806. Move them into `protocol.rs`, `plugin.rs` and `tests.rs`. Step-0 style.
- **AR9 tests (L, opus design pass).**
  - 8 fake cores (row 13).
  - 157 entries in tests/, about 23k lines of shell, 775 `sleep` lines.
  - `until_ok` copied into 21 scripts; gate-lib.sh `session_start` ends with an unconditional `sleep 2`.
  - Slices: shared `until_ok`/`wait_socket` in gate-lib (S); wait for the socket rather than sleeping; FakeCore (step 1 above); convert gates whose assertions are scene facts (red first). Pixel, e2e and cross-process handover gates stay in shell.
- **AR10 size ratchet on production lines (S, first in sprint 1).** Raw counts mislead: core ipc.rs is 4,680 lines but only 1,719 production (63 % tests). Move test modules over 500 lines into `x/tests.rs`.
- **AR5 injected input in kit dispatch (M).** runtime.rs:482 drops injected keys only. Pointer filtering is left to each plugin (row 12), and xp forgot it until 78d79db7. Plan: `Plugin::inject_ok()` defaults to false, and dispatch filters the event. Unsure: an agent's hover motion may be wanted; audit the assistant chat view first.
- **AR11 logging (M).** The core uses `tracing` (34 files). The rest uses `eprintln!` with ad hoc prefixes: kit 15, construct 38, mc 34, xp 27, assistant 13, tiling 13, chat 7. The plan is a kit `log` fn, then clippy `disallowed-macros`, plus a `reason =` on the 238 allows that lack one. Keep `Result<_, String>` (KISS).
- **AR12 dependency weight (S).** zbus leaves kit with step 10; a kit feature `notify` is the stop-gap. Duplicate fontdb 0.23/0.24 and harfrust 0.5/0.12 (cosmic-text against resvg): wait for upstream, and give each a deny.toml `skip` with a reason.
- **AR14 loose bools to enums (M).** construct `Pane { placed, taken, attached }` becomes `Origin` + `Place`; the mc chat focus flags get the same treatment. Each comes with a transition-table test.
- **AR15 core drag layer (L, design pass).** See D3.
- **AR4 one `on_path` (S).** debt-rc RF2 is overstated. There are only 2 production copies: kit desktop_entry.rs:188 and xp session.rs:397. The other two are test code. Add an exec-bit check.

## 7. Check of debt-rc.md

These hold: B1, B5, RF1, RF3 (worse than reported), RF4 (more sites, row 11), RF5 and RF6. Corrections:
- **B2 read an unmerged branch.** `StatusFile` is on `claude/xp-polish` c7e4e59e, not on head.
- **RF2 is overstated** (AR4).
- **RF6 counts tests** (AR10).
- **The D6 line numbers are off.** "Ja"/"Nein" is at mc agent.rs:165 and construct hud.rs:870.
- **debt-rc missed rows 1–4, 6, 7, 14 and the cut breaches of §3.**
- **Workflow change:** an audit names the commit it read.

## 8. Bugs found on the way (for the findings ledger; release scope decides)

- **German buttons in every locale.** mc agent.rs:165 and construct hud.rs:870 hard-code "Ja"/"Nein" whatever the session language. kit `Card::buttons(lang)` exists at ask.rs:229. The docs repeat it at configuration.md:479/:1268. S.
- **Possible security/POLS bug (not reproduced, unsure how reachable).** mc and construct show confirm `prompt`/`detail` unsanitized; voice mc.rs:188 passes them through, and they have no 400 ms `GUARD`. A bidi or hidden character could show a different command from the one that runs. Fix: `chat::sanitize` plus `Question::press`. S. Needs an opus review.
- **B1 `placed` is not saved.** Add `#[serde(default)] placed` to `PaneFile` (file.rs:27). S.
- **An empty `LC_ALL=""` masks `LANG=de`** in tiling theme.rs:707. S.
- **Stale docs.** configuration.md:483 says mc cannot tell an injected click; agent.rs `answer(node, injected)` does.
- **Still open from secfix-a r2.** kit icons.rs:76 falls back to the temp dir.

## 9. D1–D8

The owner took the PM's recommended options as proposals (11-17 04:38/04:41). I have not seen which options those were. Where they differ from what follows, weigh the reasons given here.
- **D1: A'. Keep the token, add a pidfd on the MainPID (choractl main.rs:684), drop the status file and the wrapper.** B needs `waitid`, which D42 rules out because `systemd-run` is the parent. C adds core surface for a hint. Units use `ExitType=cgroup` (:653), so a forking launcher's pid exit must be checked against the unit state. If an answer chose B, I recommend reverting it.
- **D2: B now, plus a saved `placed` field (one serde default, no format break); enums with AR14.** Stored placement as a separate model (A) adds a second truth beside geometry.
- **D3: the rule plus a unit test now; later a core drag layer.** A held window above everything is mechanism, not look policy. Numeric `order` (A) only moves the fight into numbers. This needs a design pass.
- **D4: trains stay in the PM container for the rc.** Then setup.sh writes a tool manifest, and a drifted cell reports SKIP-DRIFT instead of red. This is the owner's environment decision.
- **D5: run every unreferenced gate once.** A pass gets wired into the gate or a profile. A fail that another gate covers gets deleted. A unique fail goes to the backlog with a red test.
- **D6: kit word tables with exhaustive enums, not Fluent.** Two languages are checked at compile time. Revisit at a third language. The shared words live in `actions`/`keysheet`/`ask`, which removes most per-look tables.
- **D7: one kit `settings::Common`, flattened into each look's file; docs generated from it.** Adapting an answer of "a docs table only": with `kit::actions` the shared schema costs little, and a table drifts.
- **D8: keep 4000 and 30 s.** Measure F23 on 2560x1600@2 and scale by output area only if the load scales with pixels.

## 10. Top 10, sprints, workflow

Top 10 (gain × safety ÷ cost):
1. `kit::book` (row 1)
2. `kit::actions` with `Common` (rows 2, 3, 9)
3. chora-notify as the only server, plus `kit::notice` (row 7; zbus out of kit)
4. the cut gate plus the asset moves (§3)
5. `kit::testing` FakeCore (AR9)
6. one question model and the sanitize fix (row 5)
7. `kit::keysheet` (row 4)
8. chora-assistant as the only helper presenter (row 6, L)
9. AR7 wm.rs split
10. `chora_scene::paths` (row 11)

Sprints:
- **Sprint 1** (C10, C11, C12 already planned): migration steps 1–7 of §5; AR10 first; AR7 step 0 and AR8 (moves only) for C11; AR4 and AR12 as fillers.
- **Sprint 2:** steps 8–10, AR5, AR11, AR14; design passes for steps 10–11.
- **Sprint 3:** step 11 (assistant), step 12 (split packages), AR7 steps 1–2 with the D73 doc (Fable), AR15 doc.

Each sprint ends with lower counts:
- copies in §1;
- cut-gate breaches;
- files over 1,500 production lines;
- `eprintln!`;
- fake cores;
- `until_ok` copies.

Workflow:
- Refactors land as step-0 merges (moves only) before any logic change.
- Every new shared part names the look code it deletes, in the same merge or the next one.
- Fable goes to one L item (the AR7/D73 design); AR9 and the assistant pass are opus design passes.
- No refactor goes in before the tag; only the §8 bugs, with red tests first.
