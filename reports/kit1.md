# kit1 r1 (F2): chora_scene::paths and kit::words Language::of_session in every look

Branch claude/kit1 (from 5990b001). Commits: 27dcd025 (red: scene and kit cells, copies rows at 0), 2cdfc17f (paths, Language::of_env), f7e4bf68 (core, choractl), 614052c6 (kit, draw, log dir; `chora_kit::runtime_file`), edb94c25 (looks, services, docs, copies rows).

## Done
- `chora_scene::paths`: `runtime_home`, `config_home`, `state_home`, `data_home`, `home` (empty and relative values do not count), `RUNTIME_ENV`, `private_dir` (create 0700, then check) and `check_private` (owner is the euid, mode exactly 0700, no symlink; the refusal is logged with its reason and returned as the error).
- Users: core ipc.rs (socket dir, client dirs) and uploads.rs, text.rs, choractl (view.rs, main.rs), scene logs.rs, kit xdg.rs/icons.rs, chora-draw theme.rs, tiling, xp (desktop_icons, startmenu, press, launch), mc, construct, voice (coder, chat, config, main). Their own lookups are gone.
- `Language::of_env(get)` (LC_ALL, LC_MESSAGES, LANG; empty is unset); `of_session` calls it. The own `language()` of construct modes, assistant and voice chat and the tiling `LaunchError::notice` use `of_session`.
- `chora_kit::runtime_file(dir, name, text)` replaces three copies of the shader writer (mc, construct, tiling).
- tools/copies.toml: `XDG_RUNTIME_DIR` and `LC_ALL` at 0 outside chora-scene and chora-kit, `fn language` 3 to 0. cut-check prints no copies line. docs: api.md (refusal), known-issues.md (ti-26 gone).

## Cells (red first)
- Red on 27dcd025: scene `a_dir_with_group_or_other_bits_is_refused`, `a_symlink_is_refused_also_to_a_private_dir`, `a_dir_of_another_owner_is_refused`, `a_file_is_no_private_dir`, `a_new_private_dir_has_mode_0700`; kit `an_empty_lc_all_does_not_mask_lang`, `an_empty_lc_messages_does_not_mask_lang`; cut-check FAIL for both rows (23 and 4 lines).
- Green on 2cdfc17f: scene 7 of 7, kit words 13 of 13.
- core `a_client_socket_dir_with_group_bits_is_refused` (new); `a_client_socket_dir_is_private_and_fits_a_socket_path` no longer chmods an old 0777 dir. tiling `the_notice_of_a_missing_script_follows_the_language`. The "other owner" cell uses `check_meta(uid)` (no chown needed).
- Visible change: a client socket dir or runtime subdir that already has group or other bits (or is a symlink, or foreign) is now refused, not tightened. `choractl clients up` reads only `chora/clients.yaml` of the config home (no second look in `$HOME/.config` when `XDG_CONFIG_HOME` is set).

## Gate
Merge gate once on edb94c25 (/tmp/merge-gate.00cSEV): every step PASS (build, test, clippy, ci-check, 2d-guard, xp family, hud, layout, tiling-*, doors, acts, cube, pull, placed, construct-hud, out, winit, wm-pane, pane-keys, walk, keys, pixels, mc-*) except **e2e: gate-tiling-apps-click** "a click on the Zebra row did not start zebra". Cause here: fuzzel 1.9.2 (setup.sh could not build 1.11.1: 502 on download); same failure as construct-r2; fuzzel-r2 says the cell needs 1.11.1. Not in this diff. Skips: gate-0.9-session-units (desktop-file-validate), gate-0.7-6 (xkeyboard-config).

## Coverage
`cargo llvm-cov` is not installed (no cargo-llvm-cov in ~/.cargo/bin): **blocker, not installed**; the PM measures. By name: paths has 7 cells over every function (resolve, absolute, private_dir, check_private, check_meta); the `home`/`*_home` env wrappers are thin and covered through callers.

## Left open
- `XDG_DATA_DIRS`, `CHORA_XP_ASSETS` and `data_dirs` stay in kit xdg.rs; assistant folders.rs:37 reads `XDG_DATA_DIRS` itself.
- xp `user_dir` reads `XDG_*_DIR` variables (user dirs, not in this slice).
- Security review follows (PM).

## Refactor proposals
1. Move `data_dirs`/`data_file`/`xp_assets` from kit xdg.rs into `paths` and delete kit xdg.rs; assistant folders.rs:37 then uses it. S, one source for every XDG read.
2. xp desktop_icons.rs:895-915 `user_dir` and `parse_user_dir` belong in kit (mc and the launcher want the same folders). S.
3. `paths::private_dir` for every private file dir: kit `write_file` (lib.rs:112) uses `create_dir_all` with the default mode; the log dir (logs.rs open_in) has its own 0700 code that `private_dir` can replace. S.
4. Tiling main.rs tests (`bin`, `command`) and the gate scripts set the runtime variable by hand; a `chora_kit::testing` env helper would drop them. S.
5. kit icons.rs private_copy keeps copies in a per-pid dir that a crash never removes; a per-run dir below `chora/` of the runtime home, cleaned at the core start. M.
Cut check: no new breach; plugins use `chora_scene::paths` and `chora_kit::runtime_file` only (both allowed crates).

## PM coverage (11-18 11:55, edb94c25, cargo llvm-cov -p chora-scene -p chora-kit --features chora-kit/testing)
paths.rs lines 93.13 % (122/131), words.rs lines 98.52 % (333/338); scene+kit total 88.87 %. Target 80 % met. Security review (opus) Thu 00:05.

## Security review (opus high, PM-run independent agent, 11-19 00:09 UTC): MERGE-OK
0 BLOCK, 0 MAJOR, 6 MINOR, 2 COSMETIC (read-only, diff 5990b001..edb94c25). Clean: check_private uses symlink_metadata and euid; language values never reach a path; copies rows at 0 honest.
MINOR: (1) XDG_RUNTIME_DIR parent not checked (world-writable or foreign accepted; path bind TOCTOU; pre-existing); (2) mode check exactly 0700 refuses setgid 02700 or umask 0500 (fails safe, may break start); (3) shader and cache dirs skip private_dir; (4) voice coder.rs:73 relative "~/.local/share" fallback; (5) behaviour changes lack CHANGELOG lines; (6) test gaps: env wrappers, listen, icons foreign dir, refusal logged, ipc group-bits test not red first.
COSMETIC: xp launch.rs:167 /tmp fallback (safe); logs.rs, check.rs own 0700 code (safe).
Refactor: fd-based private_dir (openat O_NOFOLLOW, fstat, fchmod, OwnedFd); paths::runtime_subdir(name) used by listen, runtime_file, cache_dir, icons, logs; copies rows for XDG_CONFIG/STATE/DATA_HOME and var_os("HOME") at 0.
