# temp-rest

## r1

Branch claude/temp-rest, head 2e8e67f3 (red aaa5ccc1: row max 0). Gate pre-commit,ci-check,build,test,clippy,xp: green. Row "temp folder by pid outside the counted helper" is max 0, no allow beyond the helper file.
- guard.rs png: TempDir "chora-guard", returns (TempDir, path); caller binds _dir.
- transaction.rs png, sampling.rs checkerboard and colored, ipc.rs png: same, TempDir per call, file inside.
- testing.rs scratch_dir: TempDir::new(name).keep() (25 callers pass a PathBuf).
- testing/wayland.rs tempfile: file made inside a TempDir, dir dropped, the open fd stays.
- chora-xp sheets.rs theme: TempDir::unmade(..).keep() (the cache path outlives the helper; the theme makes the folder).
- No non-test site kept. autonomous-dev-session-3w-i33pg7 merged: already up to date.

## PM review r1 (2026-11-19 22:50 UTC)
Diff 86f42d1f..2e8e67f3 (8 files, +45 -49). Checks: copies row max 0 with the red commit aaa5ccc1 first; each PNG helper returns its TempDir and every caller binds `_dir` for the test's life (guard, ipc x2, transaction, sampling x4); wayland tempfile drops the dir after open (fd stays valid, as the old remove_file did); scratch_dir and xp sheets keep() where the path outlives the helper. Lane gate green. No MAJOR: MERGE-OK, train 78 after temp-dirs r2 (it builds on 86f42d1f).
