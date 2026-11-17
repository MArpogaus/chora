# Architecture refactor plan before and after rc (read-only)

Head read: rv-arch `fe8ba6d0` (train 35). Architect persona, opus, 2026-11-17. Inputs: vision-card.md, debt-rc.md, docs/decisions.md, design-principles.md, CONTRIBUTING.md, and the code. Nothing was built or run. Line counts come from `wc -l`/grep. "Production lines" means lines above a trailing `#[cfg(test)] mod tests`.
Release scope applies: before the tag only items that close a bug (with a red test first). Everything else waits until sprint 1 or later.

## 1. Check of debt-rc.md

| debt id | verdict | note |
|---|---|---|
| B1 | holds | `Pane.placed` construct main.rs:233, set :347, init false adopt.rs:112, scene.rs:229, wm.rs:115; `PaneFile` file.rs:27 saves `at`, `yaw` only |
| B2 | **wrong head** | `StatusFile` and the wrapper shell are not on fe8ba6d0. They sit on unmerged `claude/xp-polish` c7e4e59e (`merge-base --is-ancestor`: no). The audit read branch code as head code. |
| B5 | holds | the order rule exists; no unit test |
| RF1 | holds, plus a bug | tiling theme.rs:707 reads `LC_ALL` by hand. An empty `LC_ALL=""` counts as set and masks `LANG=de_DE`. kit words.rs:30 does not do this. |
| RF2 | **overstated** | 2 production copies: kit desktop_entry.rs:188 and xp session.rs:397. kit launcher.rs:222 is in `#[cfg(test)]` (:189); voice config.rs:1339 is test-only (:1190). Neither copy checks the exec bit. |
| RF3 | holds, **worse** | 3 different confirm models (§2 PT2, AR1). mc and construct have no guard and no sanitize. |
| RF4 | holds | more sites: XDG_CONFIG_HOME read by hand at tiling theme.rs:608, xp desktop_icons.rs:905, choractl view.rs:102, main.rs:1006; HOME at xp startmenu.rs:263/1008, wm.rs:2444, desktop_icons.rs:762/932 |
| RF5 | holds | kit dispatch already drops injected keys (runtime.rs:482) but not pointer events; the plugins call `hears_pointer` themselves (notify main.rs:90, assistant main.rs:245, construct plugin.rs:19, mc main.rs:815) |
| RF6 | holds, **misleading** | 24 files over 1,500 lines. Raw counts include inline tests: core ipc.rs is 4,680 lines but 1,719 production (63 % tests); input.rs 3,015/1,728; mc main.rs 4,606/3,455; tiling main.rs 3,532/~2,280. xp wm.rs 4,773/4,251 is the real outlier. |
| D6 lines | wrong lines | "Ja"/"Nein" sit at mc agent.rs:165 and construct hud.rs:870. The lines debt-rc gives (1015, 1622) are near tests. |

New items not in debt-rc: PT1–PT4 below, eight fake cores, 21 copies of `until_ok`, zbus weight in kit, a duplicated `shader_file`, and docs/configuration.md:483 being stale.

## 2. Before the tag (bug closers only; each S, red test first)

- **PT1 Ja/Nein in every locale (UX bug).** mc agent.rs:165 `[(true, "Ja"), (false, "Nein")]` and construct hud.rs:870 hard-code German. An English session sees German buttons. Fix: `Card::buttons(language)` (kit ask.rs:229, from `Word::Yes/No`). Test: an En button test next to agent.rs:1055 and hud.rs:1645. Docs: configuration.md:479 and :1268 say "Ja and Nein" and need a change. Risk: low (gate texts that grep "Ja" need the session locale; check gate-mc-*/gate-construct-hud).
- **PT2 Confirm text: sanitize and guard in mc and construct (possible security/POLS bug, not reproduced).** voice mc.rs:188 `confirm()` passes `prompt` and `detail` on unchanged. mc agent.rs:98–165 and construct hud.rs:959/993 draw them raw. Only the assistant uses kit `chat::sanitize` (chat.rs:98: ESC/CSI, bidi and invisible characters) and the 400 ms `ask::GUARD` (ask.rs:17). So a command detail that contains a bidi override could show a different command from the one that runs on "yes", and a key held from typing can answer at once. Minimal fix: call `chat::sanitize` on prompt and detail where the event is read, and use `Question::press` with the guard. Test: a detail with U+202E shows without it; a press 100 ms after show returns `TooSoon`. I am unsure how reachable this is (detail comes from the agent and tool arguments). Give it an opus review, as rule 9 asks for security.
- **PT3 B1 persist `placed` (D2).** Add `#[serde(default)] placed: bool` to `PaneFile` (file.rs:27). Old files load as false, which is today's behaviour. Test: a save/load round trip keeps `placed`; frame.rs `covers_cube` is false after reload.
- **PT4 tiling empty LC_ALL.** Replace the theme.rs:707 reader with `Language::of_session()`. Test: `LC_ALL="" LANG=de_DE` gives De. Only if the C12 owner wants it pre-tag; otherwise it moves into AR2.
- Docs (wrong docs are in scope): configuration.md:483 says "mc cannot tell an injected click", but agent.rs `answer(node, injected)` now checks it.
- Already tracked by secfix-a r2 and still present at this head: kit icons.rs:76 falls back to `temp_dir()/chora-icons-{pid}` and accepts `AlreadyExists` without a check. Keep it in secfix-a. AR3 removes the class of bug.
- Test-only, allowed: a unit test for mc `under_other_plugins()` (B5, rv6-2).

## 3. Refactors (sprint 1+ unless marked)

Format: problem/evidence · target · slices (each one merge with its test) · risk · size · gain · after.

**AR1 One question widget in kit (RF3 + PT1/PT2 at the root).** L candidate.
- Problem: three models of one safety question. kit ask.rs `Question::from_event`+`Press::{Answer,TooSoon,Injected}` (used only by the assistant: card.rs, voice.rs:375/843); mc agent.rs `Confirm` :98, `Confirming` :129, `answer` :144, `press_answer` :838; construct hud.rs `Confirm` :959, `Ask` :993, voice.rs:58. The text `eprintln!("…: confirm.answer: {err}")` is repeated three times. Principle 5 (a physical answer; third-party text never hides a question) is enforced in one of three.
- Target: kit `ask::Question` is the only state. Add `ask::send_answer(conn, &Question, yes) -> Result<(), String>`, which logs once. Every look draws its own costume from `Question::{prompt, detail, buttons(lang)}`, which are always sanitized. The look keeps only layout.
- Slices: (1) kit `send_answer` and a sanitize-at-construction test in ask.rs; (2) mc on kit `Question` (mc agent tests plus a guard test); (3) construct ditto (hud tests); (4) delete the local structs, and add a grep gate: no `"confirm.answer"` outside kit.
- Risk: medium. Focus and handover with an open question were MAJOR areas in the mc rounds. Size M. Gain: one audited path for the riskiest UI. After: PT1, PT2.

**AR2 Locale in one place (RF1), then C12 words (D6).**
- Problem: copies at assistant main.rs:517, construct modes.rs:104, voice/chat main.rs:73, tiling theme.rs:707 (buggy). `Language::De` match tables sit in about 20 plugin files (construct names.rs 8, faces.rs 8, tiling keys.rs 5, …).
- Target: `chora_kit::words::Language::of_session()` is the only reader. voice/chat depends on kit already? If not, move `Language` to chora-scene (unsure; check the dependency first). Per-plugin words become `enum Word` with an exhaustive `fn text(self, Language)`, so a missing translation does not compile.
- Slices: (1) replace the 4 readers, with a test each (env table test in kit); (2) per plugin, move the inline De/En matches into its `words.rs`, one plugin per merge, with a test that iterates `Word::ALL` × both languages for non-empty text.
- Risk: low. Size S, then M per plugin. Gain: principle 7. After: D6.

**AR3 Paths and private dirs in chora-scene (RF4).**
- Problem: 8 `XDG_RUNTIME_DIR` readers with different checks. Byte-identical `shader_file` at construct main.rs:605–630 and mc main.rs:409–426; variants in tiling main.rs:1849 and theme.rs:551, and draw theme.rs:1066. Hand-read XDG_CONFIG_HOME and HOME as in §1. Core ipc.rs:204 `listen` does not require an absolute runtime dir. Core `private_dir` at ipc.rs:257 (0700, `symlink_metadata`, uid check) is the good model; assistant check.rs:116 has its own.
- Target: `chora_scene::paths` (the core may depend only on chora-scene; logs.rs already reads XDG_STATE_HOME there) with `runtime_dir() -> io::Result<PathBuf>` (absolute or error, never temp), `config_home()`, `home()`, `private_dir(&Path) -> io::Result<()>` (moved from ipc.rs:257) and `runtime_file(sub, name, bytes)`. kit `xdg` re-exports.
- Slices: (1) move `private_dir` and add `runtime_dir`, moving the core tests along; (2) kit icons.rs uses `runtime_dir`+`private_dir` and loses the temp fallback (test: a pre-made dir owned by another uid or with mode 0755 is refused); (3) `runtime_file` replaces 4 shader writers; (4) config/HOME readers in tiling, xp, choractl.
- Risk: low to medium (the session must still start without XDG_RUNTIME_DIR in tests; gate-lib sets it). Size M. Gain: one security model for files. After: none; slice 2 may land pre-tag via secfix-a.

**AR4 One `on_path` (RF2, corrected).** xp session.rs:397 calls kit desktop_entry.rs:188, and `on_path` checks `is_file() && mode & 0o111 != 0`. Test: a non-executable file on PATH is not found. Risk: low. Size S. Gain: small; do it alongside AR6.

**AR5 Injected input refused in kit dispatch (RF5).**
- Problem: dispatch drops injected keys (runtime.rs:448/482), but pointer events reach every handler and four plugins filter them by hand. xp forgot this until 78d79db7. `from_person` is called at construct input.rs:85, tiling main.rs:425 and xp wm.rs:424.
- Target: dispatch applies `hears_pointer`/`from_person` before calling the plugin, unless `Plugin::inject_ok()` (default false; gates use `CHORA_TEST_ACCEPT_INJECTED_KEYS`). Plugins never see an injected press.
- Slices: (1) kit dispatch test, where an injected pointer press does not reach a test plugin; (2) remove the per-plugin calls one plugin per merge, keeping each plugin's injected test.
- Risk: medium. Some handlers may want injected *motion* (hover) from an agent; audit the assistant chat view first (unsure). Size M. Gain: an invariant a plugin cannot forget. After: AR9 slice 3 makes the tests cheap.

**AR6 Launch tracking in kit (D1).**
- Problem: kit lib.rs:55 `run`, :67 `launch` (token and pid), :91 `alive` (polls /proc, open to pid reuse). xp session.rs:383 has its own `run`; xp launch.rs has its own thread, channel, SHOW_AFTER 2 s and GIVE_UP 30 s, with English-only strings. 10 `Command::new` in tiling main.rs, 5 in mc fonts.rs, 3 in construct wm.rs, 3 in assistant field.rs; construct cube.rs:166 has its own child plus tail.
- Target: `kit::launch::Pending { token, pidfd, since }` with `poll() -> {Shown, Exited, Waiting, Slow}`; window matching by token stays in the look. pidfd_open on the MainPID that choractl run returns. No status file and no wrapper.
- Slices: (1) kit `Pending` with a unit test against a real short-lived child; (2) xp launch.rs on it (the existing launch tests); (3) the words of launch.rs into AR2.
- Risk: medium. A launcher that forks (flatpak, a browser's remote mode) exits early while its unit lives (choractl main.rs:636 `ExitType=cgroup`), so "exited" must be confirmed by the unit state, not the pid alone. Size M. After: D1, and B2 should not merge its status file.

**AR7 xp wm.rs split (D73).** L candidate; **the Fable item** if one is allowed.
- Problem: 4,251 production lines. One `impl Wm` covers 473–3868, about 150 methods; `struct Wm` 324–423 has about 55 fields. Clusters: lifecycle/outputs 476–760 and 1691–1786; windows/stacking 1072–1417; panel/taskbar/tray/volume 969–1050, 1479–1583 and 3347–3648; keys/switcher 1849–2190; press/drag/handover 2175–2970; dialogs/fade 3064–3330 and 3721–3851; icons 2314–2460 and 3558–3620.
- Target: step 0 is mechanical. Child modules `wm/{panel,keys,pointer,dialogs,outputs}.rs` each hold an `impl Wm` (private fields stay reachable). Step 1 adds state sub-structs: `Panel { bars, taskbar, reserved, hide_at, last_clock, tip, volume, mixer, volume_read }`, `Pointer { press, drag, handover }` and `Dialogs`. Methods move to them and take `&mut Panel` plus a narrow context. Step 2 covers D73's neutral blocks (stacking WM, panel, desktop) and chora-draw, after the design doc D73 requires.
- Slices: step 0 one module per merge, with no logic change (tests unchanged; the diff is moves only, checked with `git diff --color-moved`); step 1 one sub-struct per merge, with its unit tests moved with it.
- Risk: step 0 low, step 1 medium (borrow conflicts between panel and windows), step 2 high. Size L. Gain: review cost of the showcase look, and D73. After: AR10.

**AR8 mc and tiling mains.** mc main.rs has the protocol param structs at 93–253, `struct Mc` at 276, `impl Mc` 493–788, `impl Plugin` from 806, and production up to ~3,455. tiling main.rs production is ~2,280. Target: `protocol.rs` (param structs), `plugin.rs` (dispatch) and the tests in `tests.rs`. Step 0 style as in AR7. Risk: low. Size M. Gain: C11. After: AR10.

**AR9 Test architecture: kit fake core, fewer sleep gates.** L candidate (opus design pass).
- Problem: 8 fake cores, each with its own `UnixListener::bind` (notify main.rs, assistant main.rs, construct tests.rs, mc testing.rs, tiling main.rs, xp wm.rs ×2, kit conn.rs, voice config.rs). mc testing.rs (178 lines, with a scene model) is the best one. Shell: 157 entries in tests/, about 23k lines, 775 `sleep` lines; `until_ok` is defined the same way in 21 scripts; gate-lib.sh (118 lines) has no shared wait, and `session_start` ends with an unconditional `sleep 2`.
- Target: a kit `testing` feature (a dev-dependency only), `FakeCore` with a scene model (node tree, commands log, `send_event`, `expect(|scene| …)`), built from mc testing.rs. Behaviour that only checks scene commands moves into in-process tests. Shell gates keep pixels, e2e, handover across processes, and packaging.
- Slices: (1) `until_ok` and `wait_socket` in gate-lib, delete the 21 copies (S; every gate passes 3× in testwatch); (2) `session_start` waits for the IPC socket instead of `sleep 2`; (3) kit `FakeCore` with its own tests; (4) one plugin per merge onto `FakeCore`; (5) per gate, convert where the assertion is a scene fact, and delete the script only after the Rust test has failed once on the old bug (red first).
- Risk: medium (coverage can quietly drop; keep a gate-to-test map in tests/README). Size L. Gain: faster trains, fewer flakes (F16–F28 are mostly timing). After: AR5.

**AR10 Size ratchet on production lines.** tools: count lines above the trailing test module and fail when a file over 1,500 grows (debt counts must not grow, routines.md). Move inline test modules over 500 lines to `x/tests.rs` (core ipc.rs has about 2,960 test lines). Test: the tool on fixtures. Size S. Gain: honest C11 numbers. First in sprint 1.

**AR11 Logging and errors.** The core logs through `tracing` (34 files). Plugins, kit, draw, scene and choractl use `eprintln!` (kit 15, construct 38, mc 34, xp 27, assistant 13, tiling 13, chat 7; counts include some in tests) with ad hoc prefixes, against the L1 target scheme. Errors are mostly `Result<_, String>` (voice 67, assistant 18); that is fine for KISS, so keep it. Target: kit `log::{warn,info}(area, msg)` writing `chora-<look>: <area>: <msg>` to stderr (the journal keeps it). I do not recommend adding tracing-subscriber to every plugin; I am unsure it is worth its weight. Slices: kit fn with a test, then one plugin per merge, then clippy `disallowed-macros = eprintln` in plugins/clippy.toml. Same pass: give the 238 `#[allow]`s without a reason a `reason =`. Risk: low. Size M. Gain: one grep-able log format.

**AR12 Dependency weight.** kit pulls zbus (88 of its 125 unique deps) for `notify.rs` `Notifier`, and only xp uses it (wm.rs:22). Target: kit feature `notify`, off by default; xp enables it. Test: `cargo tree -p chora-tiling | grep -c zbus` = 0 in ci-check. Duplicate fontdb 0.23/0.24 and harfrust 0.5/0.12 come from cosmic-text against resvg: wait for upstream, and list them in deny.toml `skip` with a reason. Cargo.lock has 580 packages. Size S. Gain: build time for 5 plugins.

**AR13 Config parity (C10, D7).** `test_accept_injected_keys` is repeated at xp config.rs:106, tiling :364 and construct :238; `programs`/`terminal`/`clock_format` exist only in xp (:102–128); xp has no `bindings`. All looks already load through kit `settings::File`. Target: kit `settings::Common { programs, terminal, clock, bindings, effects, test_accept_injected_keys }`, `#[serde(flatten)]` into each look's `Config`; a docs table generated from one schema. Slices: kit `Common` with a parse test; one look per merge with old-file compatibility tests. Risk: medium (unknown keys, defaults change). Size M. Sprint 1 (already planned).

**AR14 Loose bools to enums (RF7).** construct `Pane { placed, taken, attached }` becomes `origin: Origin { Own, Taken, Attached }` and `place: Place { Face, Free }`; the mc chat focus flags (five review rounds) get the same treatment. Each change has a transition-table unit test. Risk: medium. Size M. After: PT3, D2.

**AR15 Core drag layer (D3).** L candidate (design pass, sprint 3+). See D3. Size L. Gain: no plugin fights over "on top" during a handover.

## 4. Design questions D1–D8 (recommendation, reason)

- **D1 window arrival: A' (keep the token, add a pidfd, no status file).** B needs `waitid` on a child, but D42 makes `systemd-run --user` the parent, so B contradicts a standing record. C adds a lifecycle event to the core for a hint, and the core does not know the units. A' works with what exists: the token still matches the window, and `choractl run` already returns the unit's MainPID (main.rs:684). pidfd_open gives a race-free "it ended", and a unit-state check covers launchers that fork. The exit status is not needed to say "X did not open". Unsure: flatpak and remote-mode browsers; measure before C.
- **D2 pane on cube: B now, plus `placed` persisted (PT3); A as an enum after the tag (AR14).** The geometric rule is sound (debt-rc agrees). The defect is only the lost flag. One `serde(default)` field closes it without a format break. Stored placement enums come with AR14, when the bools merge anyway.
- **D3 z-order: now the ordering rule plus the missing unit test; after the tag a core drag layer (B).** Numeric `order` on roots (A) turns a silent fight into a fight over numbers. A held window above everything is mechanism, not look policy, and the core already holds the window during a handover, so the core is the natural owner. It needs a design pass (input routing, how a plugin draws drop hints). Unsure how big it is in the render path.
- **D4 where trains run: keep them in the PM container for the rc; B after it (setup.sh writes a tool manifest, and a gate cell with drift reports SKIP-DRIFT instead of red).** A rebuilt lane image still drifts silently; a version check makes drift visible. This is the owner's environment decision (rules.md 3); default as in Needs owner F27.
- **D5 unreferenced gates: one run each on head. A pass gets wired into merge-gate or a named profile (testwatch). A fail that another gate covers gets deleted. A unique fail becomes a red backlog item.** No script stays in tests/ with unknown state. Before the tag only B4 (gate-construct-hud), because it hides red. The rest goes into AR9 slice 5.
- **D6 localization: B (kit word tables, exhaustive enums) rather than Fluent.** Principle 7 asks for one table per language; two languages compile-checked by `match` beat runtime lookup misses, a new file format and a dependency. Cost: translators must touch Rust. Revisit at a third language or community translations.
- **D7 config: one kit schema for the shared top sections (`settings::Common`, flattened), and one file per look.** Same names and same meaning everywhere is least surprise. Separate files keep a look switch independent. Generate the docs table from the schema so the docs cannot drift.
- **D8 hang budget: keep 4000 and 30 s for the rc; after the tag measure F23 on 2560x1600@2 and scale the budget by output area only if the measured load scales with pixels.** A fixed number that drops effects on HiDPI is a bug, but scaling without data could hide a real slow client. D64 abort stays.

## 5. Ranked top 10 (gain × safety ÷ cost)

1. AR1 one question widget (pre-tag PT1/PT2 first): safety-critical UI, three copies.
2. AR3 paths and private dirs: security class, 8+ readers.
3. AR5 injected input in dispatch: an invariant plugins cannot forget.
4. AR9 kit fake core and gate waits: trains and flakes; slices 1–2 are cheap.
5. AR13 config parity (C10): user-visible POLS.
6. AR2 locale and words (C12, D6).
7. AR10 production-line ratchet: cheap, keeps the debt counts honest.
8. AR7 xp wm.rs split (D73): biggest file, the showcase.
9. AR12 kit `notify` feature: S, build weight.
10. AR6 launch tracking (D1).
Then AR8, AR11, AR14, AR4, AR15.

## 6. Sprint sequence (after the tag; one sprint = about a week; unsure about capacity)

- **Pre-tag (this week):** PT1, PT2 (opus review), PT3, the docs line; PT4 only if cheap; B5 unit test. Nothing else.
- **Sprint 1** (already C10, C11, C12): AR10 first (it measures C11); AR13 = C10; AR2 = C12 with D6=B; AR7 step 0 and AR8 = C11 (moves only); S fillers AR12, AR4, AR9 slices 1–2.
- **Sprint 2:** AR1, AR3, AR5 (one security-review lane, opus); AR6 after D1; AR11; AR14.
- **Sprint 3:** AR9 slices 3–5 (design pass first); AR7 steps 1–2 with the D73 design doc (the one Fable item); AR15 design doc only.
- Each sprint ends with the debt counts (files over 1,500 production lines, `eprintln!`, fake cores, `until_ok` copies, allows without a reason) lower than at its start.

## 7. Workflow notes and pushback

- An audit must name the commit it read. debt-rc mixed branch code (B2) into head findings. Add `head:` to the audit template.
- Raw line counts mislead; the ratchet and C11 use production lines (AR10).
- Fable for one L item: AR7 (the D73 design doc) fits best, because a written design is the deliverable. AR9 and AR15 are opus design passes.
- Refactors run as mechanical "step 0" merges (moves only, `--color-moved` review) before any logic change. Reviews stay cheap, and a mistake is easy to bisect.
- Do not pull "S" refactors before the tag because they are small. Only PT items close bugs.
