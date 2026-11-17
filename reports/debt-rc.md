# Debt audit before rc (read-only)

Code: /home/user/wt/rv-debt at fe8ba6d0. Records: this log tree. Nothing was built or run; "unclear" marks what I could not confirm by reading.

## 1. Summary

- Counts: workarounds 9 (2 with user-visible risk), refactors 7, design questions 8, missing decision records 9, hygiene items 14.
- Code markers are clean: 0 TODO/FIXME/HACK/XXX in Rust; non-test `unwrap()` is near zero (crates/chora 45 and voice 11 are mostly render test helpers and fake.rs; mc audit.rs 7 `lock().unwrap()`); `unwrap_used`, `expect_used`, `todo`, `dbg_macro` are `warn` workspace lints (Cargo.toml:107-121).
- Matters most before the tag: (1) `placed` is not persisted, so a pulled pane hides again after a restart (B1); (2) CHANGELOG has no rc.4 entry and R4 (changed mc default keys) is open (B3); (3) the unknown state of cargo deny (CI2) and gate-construct-hud, red on base and outside the merge gate (B4).
- The backlog "rc" section is about 40% stale: 11 items are proven done (section 4).
- Largest after-tag debts: four copies of `language()`, four `on_path()`, three confirm.answer builders, at least 13 files over 1,500 lines (xp wm.rs 4,773), and no decision record for about nine decisions taken since 11-14.

## 2. Before the tag (user-visible risk only)

| id | what | where | why it is a workaround | proper fix | size |
|----|------|-------|------------------------|------------|------|
| B1 | A pulled pane's `placed` flag is not saved. After a restart a saved pane in front of a stood-back cube is hidden again (review-construct-r2 MINOR). The pane is not lost. | plugins/chora-construct/src/frame.rs:170 (`covers_cube`), main.rs:233 (field), adopt.rs:112, scene.rs:229 (init false) | `covers_cube` hides the pane geometrically (overlap + depth test) instead of a state that says "this pane belongs to a face". `placed` is a session-only guess for that. | Persist `placed` with the pose in file.rs, or invert it: store "on a face" (default for panes of a face) and test that. | S |
| B2 | xp Notepad fallback: an editor started as `exec nano` writes no status and waits the full 30 s; one still alive at 30 s with no window gets "Notepad opened with X" (review-xp-polish-r6 MINOR 1, COSMETIC). A failed `StatusFile::create` is silent. | plugins/chora-xp/src/launch.rs:17 (`GIVE_UP`), `StatusFile` about :158, wrapper shell with `echo $status >FILE` | Window arrival is inferred from three weak signals: the xdg-activation token (good), a 30 s timer, and an exit status written to a temp file by a shell wrapper around the command. The pid is polled with `chora_kit::alive`. The right design tracks the child with `waitid` (or a pidfd) in chora-xp and matches the window by token or app id. | Own the child in chora-xp (pidfd + `waitid`), drop the wrapper and the file, log if there is no status. Rare path: user text is "slow", not wrong. Can wait if the rc only needs the happy path. | M |
| B3 | R4: release notes do not name the new mc default keys (Super+a chat, Super+Return terminal scene). CHANGELOG.md starts at rc.3 (2026-11-01). conventions.md says the rc.4 entry was hand-written by claude/docs; I do not find it on this branch (unclear: it may sit on another branch). | CHANGELOG.md:1 | Not code. Changed defaults are user-visible and have no notice. | Add the rc.4 section (cz bump per git-conventions) with the key list. | S |
| B4 | Gate gaps hide red: gate-construct-hud.sh fails "the pane ahead has no label" on base (construct-r2.md:33, review-doors-r2); T2 (stale help row count) is open. It is not in merge-gate.sh. Also gate-construct-windows (T3) is not in the gate; construct-k1, k2a, 3d, mcpane and nested are not referenced either. | tests/gate-construct-hud.sh; tests/merge-gate.sh:183 lists only cube, keys, layout, out, pane-keys, pixels, pull, walk, wm-pane | Hidden regression risk in the HUD label path, which users see. Unclear whether the label bug is the script or the product (the construct-r2 reviewer says the diff does not touch it). | Triage once: fix or delete each of the six unreferenced construct scripts; wire the HUD one into the gate when green. | M |
| B5 | mc root is moved below the other plugins' roots at each build so a window being handed over shows above the mc page. Not "index 0" any more: `under_other_plugins()` (commit 18795b9e). Residual: order-dependent cover if another WM without `--output` shares the mc output (review-mc-r6 rv6-1). No unit test for the order (rv6-2). | plugins/chora-mc/src/main.rs:1792, :3420 | Implicit z-order by node index, re-asserted on every build; two plugins that both want "on top" fight silently. Right design: the handover window carries an explicit layer/order, or the core draws the held window in a drag layer above every root. | Core-side drag layer (see D3), or keep the ordering rule and add the missing unit test. | S for the test, M for the core layer |
| B6 | F26 (chat field left on screen) is a real fix (cause found, 50 of 50 cells), not a workaround; `hide` on every `closed` is idempotent. Remaining rc-visible flakiness: F16 (tiling bar probe), F17 (words backtrace not reproduced, only the state is named now), F18, F19, F28 residual (a chat of helper B holds Enter while A asks, review-flakes-r6 rv-1). | crates/chora/src/words.rs:203, plugins/chora-assistant/src/chat.rs:1067 | Flakes are gate noise; F28 residual is user-visible only in a two-helper case. | Keep F17 open until a sighting with the new line. | S |

Assessed as right design, not workaround: F26 hide on close; Notepad fallback chain by activation token (only the timer and status file are weak, B2); construct `covers_cube` as a rule (only the missing persistence is the defect, B1).

## 3. After the tag

### 3a. Refactors (duplication evidence)

| id | what | evidence | size |
|----|------|----------|------|
| RF1 | Locale handling copied: `language()` reads `LC_ALL`, `LC_MESSAGES`, `LANG` | plugins/chora-assistant/src/main.rs:517, plugins/chora-construct/src/modes.rs:104, services/voice/chat/src/main.rs:73, and the shared one crates/chora-kit/src/words.rs:31; tiling theme.rs:707 reads LC_ALL/LANG by hand. Plugins should call kit. Folds into C12 (Fluent). | S now, L with C12 |
| RF2 | `on_path()` four times | plugins/chora-xp/src/session.rs:397, crates/chora-kit/src/launcher.rs:222, crates/chora-kit/src/desktop_entry.rs:188, services/voice/src/config.rs:1339 | S |
| RF3 | confirm.answer wire: params builder and call in each HUD | assistant card.rs:844 + voice.rs:375/843; construct hud.rs:1353 + voice.rs:63; mc agent.rs:142 + agent.rs:843; kit ask.rs:152 already has "the answer for confirm.answer". Same `eprintln!("...: confirm.answer: {err}")` text three times. construct hud.rs:959 has its own `struct Confirm`. This is the "bridge x9" (rab-47) of C10. | M |
| RF4 | Runtime/private dir logic repeated, some with different safety checks | `XDG_RUNTIME_DIR` read at construct main.rs:632, mc main.rs:441, tiling main.rs:1850, tiling theme.rs:551, draw theme.rs:1066, kit icons.rs:76, core ipc.rs:205, voice config.rs:1307; `private_dir` at assistant check.rs:116 and core ipc.rs:257. Secfix-a r2 found icons.rs:75-86 accepting an existing dir unchecked: that is this class. One kit helper "private runtime dir (owner, mode, no symlink)". | M |
| RF5 | Injected-input checks per plugin (raa-2, raa-30) | `injected` handled in notify main.rs, assistant voice.rs/chat/view.rs/chat/host.rs/card.rs/main.rs, construct input.rs/voice.rs, xp wm.rs. Move the refusal into kit's event type so a plugin cannot forget it (xp forgot it until 78d79db7). | M |
| RF6 | Files over 1,500 lines | wm.rs (xp) 4,773; core ipc.rs 4,680; mc main.rs 4,606; tiling main.rs 3,532; voice chat.rs 3,062; core input.rs 3,015; voice core.rs 2,768; mc region.rs 2,521; assistant chat.rs 2,520; construct cube.rs 2,310; voice agents.rs 2,188; scene.rs 2,033; core render.rs 1,954. C11 says 24 files; I list the 13 largest, all over 1,500; the full count was not taken. | L, ratchet first (S) |
| RF7 | Loose Option/bool fields beside a Mode enum (R5, R7): five review rounds on mc chat focus, two on the helper word | construct `Pane.placed` is another loose bool (main.rs:233); review-ux-r1 M1 `held`. | M |
| Lints | 378 `#[allow(...)]`, 238 without `reason =` (many carry a trailing comment instead); `cast_possible_truncation` 124, `cast_precision_loss` 58, `too_many_lines` 24, `unsafe_code` 16. 161 non-test `eprintln!` in plugins/core/voice despite `print_stderr = warn`: they bypass the tracing target scheme of L1 (unclear how many are inside allows). | grep over crates, plugins, services | M |

Sleeps as synchronisation: Rust production code has only two small ones (xp wm.rs:2124 `ESCAPE_WAIT` 15 ms x 4 tries blocking the loop thread when the Escape grab is contested; construct cube.rs:757 50 ms before reading a dead child's tail). Tests: chora-assistant field.rs:373 sleeps 1 s, tiling main.rs:3503/3514 sleep 1.1 s for mtime granularity (legitimate), tiling theme.rs:263. Shell: 779 `sleep` lines under tests/ (top: gate-construct-cube 37, -windows 35, gate-mc-paths 29, -hud 23, handover-matrix 22). F21 (gate-construct-pull.sh:150 reads width once after `sleep 1`) and F27 (`sleep 3`, y 533 hard-coded in gate-tiling-apps-click) are the known cases; an `until_ok` sweep of the top five files is the fix. Size M.

### 3b. Design decisions to take (as questions)

- D1. Window arrival tracking for started programs: keep token + 30 s timer + exit-status file (A), or own the child with pidfd/`waitid` and match the window by token/app id (B), or ask the core for a `client.exited` event for activation tokens (C)?
- D2. Pane-on-cube rule: should "a pane belongs to a face or floats" be stored state (persisted, A) or derived from geometry each frame as now (B)? A needs a file format change; B needs `placed` saved at least.
- D3. Z-order of plugin roots: implicit by node index and rebuilt order (now), explicit `layer`/`order` on groups (A), or a core drag layer for held windows (B)? Who owns "above everything"?
- D4. Where do the PM trains run: PM container (now), or the lane image rebuilt by setup.sh (A)? F27: lane containers drift (apt fuzzel 1.9.2 vs 1.11.1; xp-polish r2 had no grim or set-mute; base2d missing). Option B: setup.sh version-checks and the gate refuses to run a drifted cell instead of failing red. Owner decides (rules.md 3, PK2).
- D5. Gate scope: which of the 30+ unreferenced scripts are kept (wire into gate or profile), which are deleted? Opt-in gates must pass once (A9 rule); six are not in the gate and their state is unknown.
- D6. Localization (C12): Fluent per plugin with gate and pseudo-locale (A), or kit-level word tables like `words.rs` (En/De, 81 words) extended (B)? The HUDs of mc and construct hard-code "Ja"/"Nein" button text (mc agent.rs:1015, construct hud.rs:1622).
- D7. Config parity (C10): same top sections in every look (style, effects, bindings, programs, clock); xp has no bindings section. One schema in kit or per-look files with a docs table only?
- D8. Hang and abort policy: after 3 s the watchdog logs, after 30 s it aborts (D64); D57 owner budget 4000 is a fixed number. Does the budget scale with output size (2560x1600@2 lost effects under load, F23)?
- Also open from known-issues.md: ti-12 (no focus-left/right key: "list it or decide").

### 3c. Missing decision records

docs/decisions.md was last changed 2026-11-14 (1a3f4b57, D23 amendment) and has 207 lines; 49 first-parent merges since 11-14. Merged work with design weight and, by my grep, no record (greps for `injected`, `rotat`, `secret`, `PNG`, `chora-report`, `locali`: 0-1 hits each):
1. Injected input is flagged and refused or ignored by plugins, with a per-plugin opt-out (78d79db7, 622d3724, 4d917c6a, 0a3d0040); only one passing mention in the D55 text.
2. API keys out of the agent environment; secrets split across chunks (secfix-b, 13f86740).
3. Plugin icon dir and FIFO checks (secfix-a r2, c949cc84) and "decode only small PNG files for a path a sender names" (d047cfbd).
4. Log files: location under `$XDG_STATE_HOME/chora/logs`, rotation, what a log may hold, `chora-report` scrub (19b84eb5, 47f2ce7a, docs/logs.md exists but no D-entry).
5. Watchdog counts the wait of all threads as host load, not cost (f67ad247), a refinement of D57/D64.
6. mc root ordering below other plugins (18795b9e) and handover shows above the mc page.
7. Construct: a placed pane stays in view when the cube steps back (d33bbc8e; written in docs/cube-modes.md only).
8. xp Notepad: fallback chain over EDITORS with 30 s limit and exit-status file (docs/configuration.md only).
9. Tiling: wobbly on by default (d220b3e6, 53751d06): decisions.md mentions wobbly only in D39.

## 4. Hygiene

### 4a. Backlog "rc" items already done (proof)

| item | proof |
|------|-------|
| F2 EGL PoisonError | 3092e205 (log poisoned display lock), 9e0842e0 (one display, own dir per test) |
| F3 shared temp dir in a client test | 9e0842e0; per-test dir helper crates/chora/src/testing.rs:26 (F24 still lists ~20 name+pid helper dirs, so the class is only partly closed) |
| C2 cube xp child parent-death | plugins/chora-construct/src/cube.rs:167 uses `wm::child_command` (wm.rs:38-50, `PR_SET_PDEATHSIG`) |
| A9 gate-construct-keys fails | e558ae9d, cb203c68; step `keys` at tests/merge-gate.sh:183 |
| L1 logging audit + chora-report | dist/chora-report (47f2ce7a), 19b84eb5, docs/logs.md; Merge 61d885f0 |
| PK1/PK2/PK3 packaging deps and files | dist/arch/PKGBUILD:14 depends fontconfig, fuzzel>=1.11; optdepends tex-gyre-fonts, grim, slurp, libnotify, mako, swaybg; :83-90 install tiling .ini/.conf/.png/*.sh; merge 5dd34fcd. Fedora side: unclear (dist/build-rpm.sh only; no spec read) |
| CI3 CI never ran | CI e2e runs are cited from 0e303d60 on (F25 entry, PR #106); done |
| CI4 CI-only e2e steps not in the gate | merge-gate.sh:10,24 step `e2e` runs ci.yaml in order via tests/e2e-sequence.sh, skips need tests/e2e-skips.txt |
| CI6 gate-0.8-plugin-runtime red | d039a306 routes output list/subscribe through chora-kit; remaining `"output.list"` hits are in test fakes (notify main.rs:406, xp wm.rs:4424); gate is in merge-gate.sh:158. Not run here |
| A8 watchdog node limit in api.md | docs/api.md:755 now says "no limit" and explains D57; the 14,600 figure is gone (the wording replaced it) |
| conventions "no `--locked`" | merge-gate.sh:155-157 all three cargo steps use `--locked` (stale entry in conventions.md) |

Probably done, not proven: A17 (claude/focus-flake merged; no reproduction since), F10/F11 (4c27ea0f touches words; F17 still open), A7 (rustix and rmcp bumps 76e48aa1, 651e9e00; list of PRs not checked), R16, F31 (cd98d2c8).
Still real: F16, F17, F18, F19, F20, F21, F24, F27, F28 residual, F29, F30, CI2 (deny.toml ignores only RUSTSEC-2026-0192 for ttf-parser; the xattr/tar advisory of the entry is not ignored and xattr is still in Cargo.lock:5643; cannot run cargo deny, unclear), CI5, CI7, CI8, A13, A2, G1, R4 (no entry), MC-HEAD, UX-D1, T2, T3, PK2 env (owner), and the `rc-minor` lines.

### 4b. Broken or unused tools and tests

- review_due.py: default repo path /home/user/chora (exists) and it reads a "split review" line from state.md (review_due.py:83); owner chat 11-17 04:22 (answers.md:370) says the state rewrite broke it. I did not run it. Not in routines.md as automatic. S to fix.
- triage.py and tokens.py: not exercised by any test in log/tests (only check_log, hooks, lanes, start_lane are). triage.py cites archive/charter.md sections; fine, but no selftest.
- 31 scripts in tests/ are not named in merge-gate.sh or any workflow (gate-tiling-bar, chip, doors, focus, follow, gaps, keycaps, keys, motion, notice, programs, reload, spacing, swallow, system, theme, gate-xp-notepad, gate-xp-run, gate-watchdog-starved, gate-0.7-5, -0.7-7, -0.8-11, -0.9-daily, -0.9-hardware, gate-chat, gate-handover, gate-demo, gate-mc-column, -host, -scale, voice-acp, voice-events, xp-assets, ...). Some may run via other scripts or by hand (gate-tiling-wobbly etc. are named; the unreferenced ones may be superseded). Unclear which. Also tests/battle/w1..w15 (15 scripts) and doors/mc.sh by path only.
- Voice scripts failing on branch (V12: voice-acp.sh, voice-events.sh, agent.sh step 2) are not in the gate; W16 cannot pass headless (Alt+y injected is refused by design).
- docs/known-issues.md: 34 open, 25 "verify" lines. Nobody has re-checked the "verify" lines; one lane could close most.
- Tour harness: the v5 handover script and the tour start (waybar started directly, not through waybar.sh) live in claude/media, not in this tree; I could not read them. flakes-r5 and harness-rc-r1 describe the cause (waybar.sh signals, c5306456 fix); no tour-side check that waybar.sh is used. Unclear.

### 4c. Convention gaps

- conventions.md "Known gaps": item 1 (`--locked`) is fixed (above); item 2 (rc.4 CHANGELOG by hand): no rc.4 section on the branch; item 3 (README lacks the exact "Rules for agents" wording): README.md has no such section (grep for LLM/agents finds only product text); still open.
- Red-first not followed in places: launch.rs unit tests came in the fix commit (review-xp-polish-r6 MINOR 3); harness R14: guards hook cannot take a test-only red commit for a guard.
- Gate margins: gate-mc-handover `within_ms` 1000 against 408-462 ms measured (CI-load flake risk, rv6-1b); gate-construct-pull step-back cell never asserts the mode changed (review-construct-r2 MINOR 1, green for the wrong reason possible).
- Merge gate and environment: F30 (PM container disk, no df check in merge-gate.sh), R18 (2d-guard base 8742dee fails on a shallow clone), lane cache drift (F27). Trains run in the PM container for this reason (friction.md:363); nothing in the tree records which container version a train passed on.
- Decisions: decisions.md untouched since 11-14 (section 3c); there is no rule in rules.md that a merge touching an IPC or security behaviour adds a D-entry (unclear: I did not read rules.md in full).
- Unit-test gaps listed in backlog: grab_action wiring, reload wiring (secfix-b LOW), WD2/WD3 watchdog /proc read cost and missing-thread credit (after rc), SA9, SA10 docstring.
