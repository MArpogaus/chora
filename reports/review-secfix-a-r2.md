# review secfix-a r2 (bf5904f4) - VERDICT: MERGE-OK (no open BLOCK/MAJOR)

Diff base caf9c80d; r2 change over b58d3a0e is only crates/chora-kit/src/icons.rs + docs/known-issues.md.

| Sev | Where | Finding |
|---|---|---|
| MINOR | icons.rs:75-86 private_copy | AlreadyExists on the dir is accepted without lstat/owner/mode check. In $XDG_RUNTIME_DIR (0700, own) this is safe. In the fallback std::env::temp_dir() (/tmp, XDG unset) another user can pre-create /tmp/chora-icons-<pid> (pid predictable) as their own dir or symlink; the copy then lands in an attacker-controlled dir and can be swapped after write, reopening the TOCTOU. Fix: after create, symlink_metadata must be a dir, uid==geteuid, mode&077==0, else None; or refuse the fallback. |
| LOW | icons.rs:80-93 | Crash/SIGKILL leaves dir and up to 16 files (Drop does not run). With pid reuse the serial restarts at 0, create_new hits the stale file, returns None and falls back to the theme icon (degrades, not unsafe). Dir itself is never removed on drop. Fix: sweep stale chora-icons-* at startup or use a random name. |
| LOW | icons.rs:49-72 | Multiple Icons instances share one dir/serial counter: fine (atomic), but one instance's drop only removes its own files. |
| COSMETIC | icons.rs:268 tests | Test files live in temp_dir()/chora-icons-<pid>, same name scheme as the runtime dir. |

Checked OK: O_NONBLOCK open then fstat is_file on the handle (FIFO, dir, device refused); symlink to a regular file is followed but bytes are read once so no later race; length check before alloc, take(cap+1) bounds read, so no allocation above 512 KiB+1; bytes<24 guarded before indexing; copy created create_new + 0600 (O_EXCL, no symlink follow on final component); write error removes file; LRU 16.

Reran (CARGO_TARGET_DIR in the worktree):
- cargo test -p chora-kit --lib icons: 8/8 pass.
- Red: removing the O_NONBLOCK line -> a_fifo_as_a_sender_path_falls_back_at_once FAILS (1.1 s). Making for_sender return the original path instead of the copy -> a_sender_file_swapped_after_the_check_is_not_read and the_copies_are_removed_with_the_icons FAIL. Source restored, tree clean.
- gate-doors.sh: could not run. It needs the full workspace binaries (target/debug); the build died with ENOSPC (disk full). Not a result either way.
- Doors/icons: r2 touches only icons.rs (Icons::for_sender and a new Drop) and docs; plugins/chora-mc has no reference to Icons/icons::. The doors script uses stand-in grim/wpctl and the mc keys/capture files, which r2 does not change. The mc "missing" failures cannot come from this diff; flake explanation plausible but unreproduced by me.
- target dir deleted.
