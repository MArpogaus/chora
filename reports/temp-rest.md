# temp-rest

## r1

Branch claude/temp-rest, head 2e8e67f3 (red aaa5ccc1: row max 0). Gate pre-commit,ci-check,build,test,clippy,xp: green. Row "temp folder by pid outside the counted helper" is max 0, no allow beyond the helper file.
- guard.rs png: TempDir "chora-guard", returns (TempDir, path); caller binds _dir.
- transaction.rs png, sampling.rs checkerboard and colored, ipc.rs png: same, TempDir per call, file inside.
- testing.rs scratch_dir: TempDir::new(name).keep() (25 callers pass a PathBuf).
- testing/wayland.rs tempfile: file made inside a TempDir, dir dropped, the open fd stays.
- chora-xp sheets.rs theme: TempDir::unmade(..).keep() (the cache path outlives the helper; the theme makes the folder).
- No non-test site kept. autonomous-dev-session-3w-i33pg7 merged: already up to date.
