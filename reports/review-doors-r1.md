# Review: doors r1 (rc 1), claude/doors at ea47e27b

Reviewer lane: opus, read-only. Diff: `git diff claude/autonomous-dev-session-3w-i33pg7...ea47e27b`
(3 commits: af2d0567 refactor, 17082c87 test, ea47e27b feat). Brief: briefs/doors-r1.md.
The worker session (..N9VG6) hit the usage limit at 10:35 and stopped without a report; the branch is its
work in progress.

Gates (worktree /home/user/wt/doors):
- `tests/merge-gate.sh --worker ''`: build, test, clippy PASS (195 s, 71 s, 88 s).
- `tests/merge-gate.sh --red-first-check`: passes (the test commit 17082c87 comes before the feat).
- `tests/gate-doors.sh` (a step of the full merge gate since 17082c87): FAIL, 80 of 80 cells `missing`, "no tests/doors/{xp,tiling,mc,construct}.sh".
- gate-tiling-system, gate-tiling-doors: PASS. gate-tiling-power: FAIL "Super+Shift+e opened no power menu within 2 s"; it fails the same way on the session branch (fuzzel 1.9.2 here, see review-xtc-r1.md).

Findings (id | severity | file:line | text | evidence):
- dr-1 | BLOCK | tests/gate-doors.sh:13 | The doors step is wired into merge-gate.sh, but no look answers it: tests/doors/ holds only lib.sh. Merging turns the full merge gate red for every later merge | gate-doors.sh output: "FAIL: 80 cells are not pass"; commit 17082c87 body "No look answers yet: 80 cells are missing"
- dr-2 | BLOCK | (brief goals 1, 2, 3, 5) | Scope not done: no key-set change in any look (Super+F1/Super+/, Super+Shift+Esc, F1 free), no door in xp, mc or construct, no tiling helper door, no clock check, no "Chat with a helper" row outside tiling, xp-1, xp-3 and ti-1 untouched | diff touches no file in plugins/chora-xp, plugins/chora-mc, plugins/chora-construct; only goal 4 (shared folder) is done
- dr-3 | MAJOR | crates/chora-kit/src/words.rs:119 | feat ea47e27b adds Word::NoHelperDoor, which nothing reads; the commit is a feat with no behaviour and no cell that turns green | `grep -rn NoHelperDoor` finds only words.rs
- dr-4 | MINOR | crates/chora-kit/src/shared.rs:83 | `shared::script`, `shared::command` and `shared::data_dirs` are public API with no caller outside their tests; `data_dirs` duplicates `chora-tiling theme::data_dirs`. Brief: "no new API unless two looks need it" | only `shared::dirs()` is used (plugins/chora-tiling/src/theme.rs:616)
- dr-5 | MINOR | flavors/shared/power.sh:384 | The refactor commit af2d0567 changes behaviour (Lock runs swaylock itself instead of `loginctl lock-session`, new notice, new CHORA_MENU/CHORA_LOCKER); one reason per commit, and a behaviour change in a `refactor` escapes the red-first check. The change itself is sound (honest notice when the lock fails, gate-tiling-system covers it) | git show --stat af2d0567
- dr-6 | MINOR | plugins/chora-tiling/src/main.rs:243 | Merge with claude/xtc-fix: xtc's Apps list (`chora_kit::launcher::run`) takes only the first dir with applications/ and bypasses `theme::data_dirs`; once Screenshot and Chat with a helper live in flavors/shared/applications they vanish from the tiling Apps list. The launch() block and docs/configuration.md:1577 also conflict textually | see review-xtc-r1.md xr-3
- dr-7 | COSMETIC | docs/configuration.md:1574 | The shared folder is documented only inside the tiling section; the other looks will need a shared section once they use it | docs diff

What is usable now: af2d0567 (the move to flavors/shared, packaging in Cargo.toml, PKGBUILD and build-arch.sh, all tiling gates and unit tests updated) is correct on its own and could land alone if the PM wants it.

Verdict: FIX (dr-1, dr-2 BLOCK; dr-3 MAJOR). A new doors round must write tests/doors/LOOK.sh for the four looks, then the fixes, and either use NoHelperDoor or drop it.
