# kit-sec r2 (F5, security): fix the review r1 findings

Model: sonnet, effort high (security checks of runtime dirs; plan given by the review)
Lane: cloud, branch claude/kit-sec at cb2976c1. One round, stop at 250 k. Cost about 0.8 %.
Step 0: run setup once: `git fetch origin claude/session-log && git show origin/claude/session-log:setup.sh > /tmp/setup.sh && bash /tmp/setup.sh`. Then `git fetch --unshallow` and `cargo check --workspace`; a failure: send_message the PM "kit-sec r2 BLOCKED: <MISSING line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read on claude/session-log: reports/kit-sec.md "## r1" and "## Review r1" (rw-1 to rw-7).
Each item: a red test commit first, then the fix commit.
1. rw-1 MAJOR: waybar.sh and wallpaper.sh make chora-tiling 0700 (`mkdir -p -m 700`); a test (shell or selftest) that runs each script with umask 022 and checks the mode.
2. rw-2: `private_dir` tightens a dir of the euid, no symlink, with no group or other write bits, to 0700 (dirs of the last release); write bits stay refused. CHANGELOG says it.
3. rw-3: check the runtime dir before `state.listen()` binds the Wayland socket (lib.rs:116); a test that no socket is made in a refused dir; CHANGELOG true.
4. rw-4: canonicalize the runtime dir once in `runtime_subdir_in`; check and join below the canonical path; a test with a symlink.
5. rw-5: a dir without owner write bits (0500) is refused for dirs chora writes, with path and mode in the message.
6. rw-6, rw-7 NITs: the refusal names the remedy ("chmod 0700 PATH, or set XDG_RUNTIME_DIR to a dir of yours") and prints with Display, not Debug; the icon temp fallback gets a random suffix.
Gate: merge origin/claude/autonomous-dev-session-3w-i33pg7, then `tests/merge-gate.sh --jobs 3 --only pre-commit,ci-check,build,test,clippy,doors,xp,tiling-look`; all PASS.
Poll each run in the same turn; never end a turn to wait.
Done when: each item names red commit, fix commit and test; gate green; reports/kit-sec.md "## r2" on claude/session-log; head sha.
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 6 lines).
Rules: push claude/kit-sec and claude/session-log only (fetch and merge on reject, no force); author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email on every commit); no attribution lines; never --no-verify; no system packages.
