# kit-sec r1 (F5): the six KIT1 security MINORs

Branch claude/kit-sec, head cb2976c1 (merge of origin/claude/autonomous-dev-session-3w-i33pg7 on top of 65928c9c; base a9e7084d). Pushed.

## r1
| # | Red commit | Fix commit | Test that proves it |
|---|---|---|---|
| 1 runtime dir checked (euid owner, no 022 bits; logged refusal with path and mode) | 37a0cc2b | 61e20d87 | scene `a_runtime_dir_that_others_can_write_is_refused`, `a_runtime_dir_of_another_owner_is_refused` (check_runtime_meta with a foreign uid), `no_subdir_is_made_in_a_runtime_dir_that_others_write`; core `listen_refuses_a_runtime_dir_that_others_can_write`; kit `a_runtime_file_is_refused_in_a_runtime_dir_that_others_write` (red 2bf4e4c5, fixed in 5be5b264) |
| 2 mode check | e22b12a9 | 7625b22f | scene `a_dir_without_group_or_other_bits_is_accepted` (0700, 02700, 0500, 01700); `a_dir_with_group_or_other_bits_is_refused` stays |
| 3 shader, icon and cache dirs through the helper | 2bf4e4c5 | 5be5b264 | kit `the_directory_of_a_runtime_file_is_private`, `an_old_directory_of_others_is_refused_for_a_runtime_file`; icons `no_copy_is_made_in_a_runtime_dir_that_others_write`, `a_copy_is_private_below_a_runtime_dir_of_the_user`; draw `the_cache_is_refused_in_a_runtime_dir_that_others_write`, `the_cache_and_its_instance_are_private_dirs` |
| 4 voice data dir | c0f7c063 | bcbad2a7 | voice `no_data_home_is_no_relative_path` |
| 5 CHANGELOG | - | 65928c9c | new "## Unreleased / ### Fix" block, four lines (there was no Unreleased section; commitizen writes the releases) |
| 6 tests | in the red commits above | | env wrappers: scene `the_homes_read_the_environment` and `the_wrapper_of_the_runtime_dir_refuses_a_world_writable_one` (child process of the test binary, since unsafe is denied and a test cannot set a variable); listen: core `listen_binds_below_a_runtime_dir_of_the_user`; icons in a loose dir: above; refusal logged: scene `the_refusal_of_a_runtime_dir_is_logged_with_path_and_mode` (tracing capture, new dev-dependency tracing-subscriber of chora-scene) |

## Decisions
- Item 2: a check (no 077 bits), not fchmod after create. Creation still asks for 0700.
- Item 3: one helper, `paths::runtime_subdir_in(runtime, name)` (and `runtime_subdir(name)` from the environment): checks the runtime dir, then `private_dir`. Used by listen, runtime_file, icon copies, cache_dir (and its instance dir) and the tiling runtime dir (else a 0755 `chora-tiling` made by the theme writer would now block the shader file). Not fd-based: every caller uses the path afterwards, so an fd check gains nothing; the Refactor row stays open for that and for the copies rows at 0 (XDG_*_HOME, var_os("HOME")).
- Runtime dir check follows a symlink (stat), checks the target. Icon copies fall back to the temp dir only when the runtime dir is unset or no directory; a set but unsafe one refuses (no copy, theme icon).
- Visible: a runtime dir that group or other can write now stops chora at start (listen error) and stops plugin shaders, caches and icon copies. The refusal is a warn line with path and mode.
- Not done: xp launch.rs:167, construct cube start, voice config.rs:1306 keep their runtime_home or /tmp fallbacks (COSMETIC, safe; not in the list).

## Gate
`tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,doors,xp` on cb2976c1 (run /tmp/merge-gate.qKLHuk): all seven PASS (pre-commit 20 s, build 16, test 247, clippy 4, ci-check 19, xp 92, doors 335). The origin/claude/autonomous-dev-session-3w-i33pg7 head was already merged at the gate. Cells of flake-h, flake-k, flake-l, nits-3 and pw were not touched. The first gate run died with a container restart and was run again whole.
