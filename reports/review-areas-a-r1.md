# review areas a r1 (rc 6): kit, xp, tiling, notify

Session branch head reviewed: `claude/autonomous-dev-session-3w-i33pg7` at **66588880c9106f0276a438a6a827949f6a07b5f4** (read-only, one round).
Scope: crates/chora-kit, plugins/chora-xp, plugins/chora-tiling, plugins/chora-notify.
Checklist: design/rc.md row 6 (simplicity and readability, then extraction and duplication, correctness, security per charter 11, dead code, docs).

Method and limits (stated plainly): the earlier reviews (review-plugins-r1 at 498d9de, review-core-r1) already cover these crates line by line, so this round did **not** repeat them.
I read everything that changed in the four areas since 498d9de (about 2,460 added lines: kit launcher.rs, shared.rs, runtime.rs, settings.rs, give.rs; xp apps.rs, launch.rs, session.rs, sound.rs; tiling main.rs, keys.rs, theme.rs; notify main.rs, card.rs, bus.rs).
For the older findings I re-checked the HIGH and MEDIUM ones listed under "Earlier findings that still hold" against the head. The older LOW findings were not re-checked one by one.
Nothing was run (no build, no tests). Line numbers are those of the head sha above.

## Counts

| area | HIGH | MEDIUM | LOW | security (included) |
|---|---|---|---|---|
| chora-kit | 0 | 3 | 5 | 1 MEDIUM (raa-3), 1 LOW (raa-4) |
| chora-xp | 0 | 1 | 6 | 1 LOW (raa-10) |
| chora-tiling | 0 | 2 | 4 | none |
| chora-notify | 0 | 3 | 4 | 1 MEDIUM (raa-23) |
| across plugins | 0 | 2 | 1 | none |
| total | 0 | 11 | 20 | 2 MEDIUM, 2 LOW |

No new HIGH. The three HIGH of review-plugins-r1 (XP-B-01, MC-01, NO-01) are fixed (confirmed by review-plugins-high-r1 and by the code, card.rs:26-30, 202-208).

## chora-kit

- raa-1 MEDIUM simplicity+correctness crates/chora-kit/src/runtime.rs:384, 407: `start_grabs` / grab everything / `end_grabs` is a three-step protocol that every caller writes by hand (tiling main.rs:601-610 and 1634-1636, xp wm.rs:4247-4249, mc main.rs:990, construct camera.rs:471-493). Tiling writes `grab_all()?; end_grabs()?`: when a grab fails, `end_grabs` never runs, `old_grabs` keeps the bindings that the core still holds, and `grabs` has already forgotten them (the key then reaches no action until the next reconfigure). Fix: one `Runtime::regrab(|rt| ...)` that runs `end_grabs` also on the error path, and make the five callers use it.
- raa-2 MEDIUM simplicity+duplication crates/chora-kit/src/runtime.rs:27, 48: `from_person` and `hears_pointer` are two near-twins over the same nine event variants, and every plugin adds its own spelling on top: wrappers `hears()` (tiling main.rs:421, xp wm.rs:423), `!injected || accepts_injected` inline (notify main.rs:95, mc main.rs:1144, construct input.rs 8 places, modes.rs 5 places). One missed spelling is an injected-input hole (charter 11), and the tests pin the match arms by `include_str!` of the source text (runtime.rs:974-996), which breaks on any reformat. Fix: one `Runtime::hears(&Event)` that reads its own `accepts_injected`, one event list, and a test over the `Event` enum that does not read source text.
- raa-3 MEDIUM SEC crates/chora-kit/src/launcher.rs:106, 185: text typed into the launcher that matches no row, whose first word is an installed program, runs as a shell command (`Answer::Run(text)` then `crate::run(&command)`, which is `sh -c`). That is the Run box of XP by design, but it is the launcher of every look, the text goes to a shell with no word on it, and `choractl run` gives it a unit with the activation token of the plugin. The docs of `answer` and of `launcher::run` say it, the key list and the help text of the looks do not. Fix: say in the help text of both looks that typed text runs as a command, and refuse text with a shell metacharacter (`;|&$\`<>(){}`) unless the whole line is one installed program name plus plain arguments.
- raa-4 LOW SEC crates/chora-kit/src/shared.rs:39-44, 48-51, 57: `power.sh` and `screenshot.sh` are run with `sh` from the first folder that has them: `CHORA_SHARED_DIR`, then `../shared` of `CHORA_FLAVOR_DIR`, then the installed path, then any `flavors/shared` above the binary. A binary in a user-writable place (a build in `target/`) is found by the last rule, and an environment variable first. Same user, so no privilege step, but the power script is also what Lock and Log Off run. Fix: let the installed folder win over the checkout rule unless `CHORA_SHARED_DIR` is set, and log the folder that was chosen once.
- raa-5 LOW dead/duplication crates/chora-kit/src/shared.rs:67, plugins/chora-xp/src/launch.rs:317, plugins/chora-xp/src/desktop_icons.rs:952: three copies of the single-quote function (`format!("'{}'", w.replace('\'', ...))`). Fix: make `chora_kit::shared::quote` pub (or move it to lib.rs) and delete the two copies in xp.
- raa-6 LOW simplicity crates/chora-kit/src/launcher.rs:169-174, shared.rs:97-111, plugins/chora-tiling/src/theme.rs:648-665: "which folders have `applications`" is decided three ways: `launcher::entry_dirs` takes the first shared folder, `shared::data_dirs_in` takes the first, tiling's `data_dirs` takes all (`dirs.iter().filter(..)` plus `dedup`). The launcher list and the XDG_DATA_DIRS of fuzzel can then disagree about the same entry. Fix: one function in `shared.rs` that returns the folders, used by all three.
- raa-7 LOW logging crates/chora-kit/src/launcher.rs:124-127, lib.rs:16-20: `ask` logs a failed fuzzel start with a bare `eprintln!` and drops the write error to its stdin (`let _ =`); a launcher that never opens leaves no line saying which program or which config file. Fix: one log line with the program, its first arguments and the error.
- raa-8 LOW docs crates/chora-kit/src/lib.rs:4-6: the module comment lists "the start of apps" but not launcher, shared files, give keys, notify or volume; `shared.rs` doc line 3 says the entries are "Chat with a helper, Screenshot and Screenshot of an area" (the test at shared.rs:148 pins them) but the folder is not described in any README. Fix: one sentence per module in the crate doc.

## chora-xp

- raa-9 MEDIUM simplicity plugins/chora-xp/src/wm.rs (4773 lines, one `Wm`): wm.rs grew by 556 lines since r1 (XP-A-07 still holds and is worse). The injected-input comments (see raa-12) and the grabs (raa-1) show the cost: six concerns edit one impl. Fix: split by the seams that already exist in `wm/` (settings dialogs, taskbars, keys, switcher, session, volume) before more is added; known issue with a reason if not done in rc.
- raa-10 LOW SEC plugins/chora-xp/src/session.rs:383-395: Turn Off, Restart, Stand By and Log Off run the command of `xp.yaml` with plain `sh -c` in a thread, outside `choractl run` (no unit, no activation token, no confirm). The defaults are now the shared power script, so a user who edits `turn_off` gets a shell with the full environment of the plugin. Same user and file, so LOW; the other launches go through `chora_kit::run`. Fix: use `chora_kit::run` for these commands, or document that the four commands are plain shell.
- raa-11 LOW simplicity plugins/chora-xp/src/sound.rs:45-48: `sh -c "{player} \"$1\" ... &"` only to detach the player; the player is one of three constants. Fix: `Command::new(program).args(rest).arg(&file).spawn()` and reap the child, or call `chora_kit::run`.
- raa-12 LOW simplicity plugins/chora-xp/src/wm.rs:812-815, 840-844, 1817-1844, main.rs:179-181: 17 trailing `// injected: ... hears drops ...` comments on match arms, the same sentence copied (project rule: no inline comments in function bodies; r1 XP-A-21 and XP-B-18 are the same finding). Tiling has 7 (main.rs:681-703, 888, 893) and notify 1. Fix: one doc line on `hears` and delete the arm comments; `node: _, code: _, modifiers: _, text: _, injected: _` in tiling main.rs (NodeKey arm) says nothing and can be `..`.
- raa-13 LOW correctness plugins/chora-xp/src/apps.rs:97-122: the editor of `VISUAL` or `EDITOR` is run as a shell command; `installed(command)` tests only the first word, so `EDITOR='vim; rm -x'` passes and the whole text goes to `sh -c` (same user, own environment). More likely: `EDITOR=code --wait` is opened in a terminal as if it had no window (`windowed` is a list of eleven names). Fix: treat the env value as a program and arguments, run it without a shell, and ask `desktop_entry` whether the program is a graphic one.
- raa-14 LOW duplication plugins/chora-xp/src/session.rs:397-400, crates/chora-kit/src/desktop_entry.rs:188: `session::on_path` is a copy of `desktop_entry::on_path`. Fix: delete the copy.
- raa-15 LOW docs plugins/chora-xp/xp.yaml, src/config.rs docs of `turn_off`, `restart`, `log_off`: the docs now say "runs the shared power script" while the tests (config.rs) pin `None`; the Run box (`launch.rs` `RUN_BOX`, `run_box`) and `editor` have no entry in `xp.yaml`. Fix: add both to xp.yaml with their defaults.

Earlier xp findings that still hold at the head (re-read): XP-B-02 `taskbar.rs:272-284` (`owns` ends_with `-{output}`, DP-1 owns eDP-1 nodes) MEDIUM; XP-A-14 `wm/effects.rs:194-195` (`slow()` only bounds from below, inf panics in `mul_f64`) LOW; XP-B-11 `datetime.rs:88-98` (zone falls back to UTC silently); XP-B-08 `startmenu.rs:576` (`count - 1` on an empty cascade). XP-B-12 (quote in `themed`) is **fixed** by `quoted` (launch.rs:317), though see raa-5. XP-B-05 `launch.rs:21` (`DETACHED` hard-codes fuzzel and chora-chat) still holds. The other XP-A and XP-B findings were not re-checked.

## chora-tiling

- raa-16 MEDIUM correctness plugins/chora-tiling/src/main.rs:631-640, 946-962: synchronous `Command::status()` on the event loop got more of them since r1 (TIL-03 still holds: `makoctl mode` on every assistant chat event, `systemctl --user`, `makoctl reload`, `kill`); `--launch` now also calls `notify-send` with `.status()` (main.rs:236) in a process that can wait on D-Bus. A hung `makoctl` or `systemctl` freezes every window of the session. Fix: spawn them on a thread (the theme thread exists) or `spawn()` and reap.
- raa-17 MEDIUM simplicity plugins/chora-tiling/src/main.rs (3529 lines, 102 `unwrap`/`expect` sites, mostly tests): TIL-07 still holds; the 330-line shell-script test module (TIL-36) is still inside main.rs. Fix: move `mod panel` to `tests/`, and the key list and theme handling out of `Tiling`.
- raa-18 LOW simplicity plugins/chora-tiling/src/keys.rs:273-310: `volume_row` builds a `Vec<(usize, String, String, String)>`, sorts it, and joins with `" / "`; the group is found by `words(&Action::Volume(Change::Up))` and the rank by looking the English text up in `ORDER`, which now holds `"Volume up / Volume down / Mute"` (a text that is built from three other texts). `give_row` appends with rank `ORDER.len()`. Fix: a struct with named fields, and an enum rank rather than a text lookup.
- raa-19 LOW duplication plugins/chora-tiling/src/main.rs:248-263, theme.rs:648-665: `--launch fuzzel` sets `XDG_DATA_DIRS` itself with tiling's `data_dirs`, while `chora_kit::shared::launcher` builds the same environment as a shell prefix for xp (raa-6). Fix: use the kit function in both.
- raa-20 LOW correctness plugins/chora-tiling/src/main.rs:601-606: `start_grabs()` runs before `apply_theme(true)`, which runs `systemctl` and `makoctl` (raa-16); between them `grabs` has lost every non-kept binding, so a key pressed in that time finds no action and is dropped without a line. Fix: call `start_grabs()` right before `grab_all()`.
- raa-21 LOW docs plugins/chora-tiling/src/keys.rs:666-676 (TIL-17, partly fixed): the header now says `Super+F1` instead of `Super+/`, but it is still a literal; `modifier: Alt` or a rebinding of `show_keys` still shows a wrong key. Fix: take the key from the binding of `show_keys`.

Earlier tiling findings that still hold (re-read): TIL-15 `config.rs:316` (MAX_GAP applies to look gaps, not to `gap` of `set_layout`) and TIL-16 `theme.rs:189` (`from_str_radix` takes a sign) LOW. TIL-01 to TIL-06 and the rest were not re-checked.

## chora-notify

- raa-22 MEDIUM correctness plugins/chora-notify/src/card.rs:106-112, bus.rs:190: NO-02 still holds. The waiting queue is a `VecDeque` with no cap, fed by an unbounded `mpsc::channel()`; the cap of NO-01 limits one card, not the number of cards. An app that calls Notify in a loop grows memory without end. Fix: cap `waiting` (drop the oldest of the same app, or refuse above 64) and use `sync_channel` in `bus.rs`.
- raa-23 MEDIUM SEC plugins/chora-notify/src/bus.rs:69, crates/chora-kit/src/icons.rs:70-74: NO-03 still holds. `app_icon` or the `image-path` hint of any D-Bus caller that is an absolute path is read (`path.is_file()`) and decoded by the card with no limit on dimensions and no `check_tree` for SVG, outside the confined asset rules (charter 11: the notification server is reachable by every process of the session). Fix: for a sender path, decode only PNG under a size and pixel cap and never SVG, or accept only names of the icon theme.
- raa-24 MEDIUM correctness plugins/chora-notify/src/card.rs:101-109, bus.rs:111-117: NO-04 still holds, and a second issue: `replaces_id` from the caller is taken as the id as it is, and the counter `last` is not moved past it, so `Notify(replaces_id: 7)` for an unknown 7 and a later fresh id 7 are one notification (the second replaces the first, a Close for one closes the other). Fix: when `replaces_id` names no known notification, give a fresh id; and let one app keep at most one card in `shown`.
- raa-25 LOW simplicity plugins/chora-notify/src/main.rs:90-99: three injected checks in 10 lines (`hears_pointer`, `!injected || accepts_injected` on the press, and a trailing comment that claims `Plugin::event` drops the entry, which is true only for entries). Fix: `from_person` once at the top (raa-2).
- raa-26 LOW correctness plugins/chora-notify/src/main.rs:136-150: the plugin takes the bus name before it connects to the core; when `Runtime::start` fails, the process exits having owned the name for a moment, and notifications sent in between are acknowledged on the bus and lost. Fix: connect to the core first, take the name second.
- raa-27 LOW logging plugins/chora-notify/src/bus.rs:111, 121-128: `let _ = app_name;` (unused parameter kept for the signature, say so with `_app_name`) and `let _ = self.requests.send(..)`: when the main loop is gone the caller still gets an id. Fix: return a D-Bus error when `send` fails.
- raa-28 LOW docs plugins/chora-notify/src/main.rs:6-9: the module doc names `docs/xp-split.md` and D73; the asset folder rule is repeated in the doc of `Args::assets` (58-62) and in `serve`. Fix: keep it in one place.

## Across the plugins (extraction)

- raa-29 MEDIUM duplication plugins/chora-xp/src/{startmenu.rs:1263, taskbar.rs:805, context.rs:1005, dialog.rs:1053, cascade.rs:314, session.rs:374, decoration.rs:463}: the translate helper `at(x, y) -> PropsUpdate` exists seven times now (XP-B-19 said five). Likewise chora-construct (glass.rs:29, hud.rs:323). Fix: `chora_kit::geometry::at` (or on `PropsUpdate` in chora-scene) and delete the copies.
- raa-30 MEDIUM duplication, same event handling in xp wm.rs:423-440, tiling main.rs:421-440, notify main.rs:89-100: each plugin writes the same prologue (drop injected, then match). Part of raa-2; listed once more because the fix is a default method `Plugin::heard(&Event) -> bool` that `dispatch` calls before `event`, so no plugin can forget it.
- raa-31 LOW logging crates/chora-kit/src/lib.rs:8-14 and the three plugins: 57 `eprintln!` in non-test code of the four areas under a crate-wide `allow(clippy::print_stderr)` (TIL-49, XP-A-08). One decision for all plugins; nothing new, listed as a reminder.

## Earlier core findings of the kit that still hold

K10 `settings.rs:236` / `runtime.rs` (reload clears `broken` before the plugin applies) MEDIUM, not re-checked in this round beyond reading `settings.rs:119-135` (the refusal text is still there). K22 to K31 not re-checked. K03 (value quoted in parse errors) is fixed on the read path (`settings.rs:262-266`, `without_value`); `settings::parse` (289-294) still returns the raw error, but only plugins' own defaults go through it.

## HIGH and security MEDIUM list (the ones rc 6 must fix)

- HIGH: none.
- Security MEDIUM: raa-3 (launcher runs typed text as shell, kit launcher.rs:106), raa-23 (notify decodes any path a D-Bus sender names, bus.rs:69 and icons.rs:70).
- MEDIUM, not security: raa-1, raa-2, raa-9, raa-16, raa-17, raa-22, raa-24, raa-29, raa-30; of the older ones XP-B-02 and K10.
