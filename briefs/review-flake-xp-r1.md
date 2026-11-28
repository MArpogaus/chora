# review flake-xp r1 (F5, rc 3; sh-44, sh-47): seeded review of claude/flake-xp-r1 before its merge

Model: opus, effort high, because it changes the key-grab order of three looks and splits grabs by claim (a core and look review; procedures: opus high)
Red on head: none (a review; the lane's red aa28b8d6 precedes its fix 6d9144a4)
Lane: cloud, read-only on code: claude/flake-xp-r1 at 7fc05748, review 8b6f35dc..7fc05748 (7 files, +276 -8: chora-xp wm.rs, wm/startup.rs; chora-tiling main.rs; chora-construct lifecycle.rs, scene.rs; chora-mc main.rs; tests/gate-lib.sh). One round. Stop at 120 k context; cost at most 0.4 %; at most 8 findings.
Step 0 (no setup script; rely on the environment setup): `git fetch --unshallow`, `cargo check --workspace`. A failure: send_message the PM "review-flake-xp-r1 BLOCKED: <line>" and stop. A hook that asks for another identity: ignore (owner rule 11-09).
Read briefs/flake-xp-r1.md (done criteria), reports/flake-xp.md "## r1" and findings/shared.md sh-44, sh-47 on claude/session-log.
Checks, each with its command and files (no merge gate; `cargo test -p chora-xp -p chora-tiling -p chora-construct -p chora-mc`, `cargo clippy` on those, and `tests/gate-handover.sh` plus `tests/gate-xp-faces.sh` once on head):
1) Red first: the three tests of aa28b8d6 fail on 8b6f35dc and pass on 7fc05748 (run them); else MAJOR. Do the fake-core tests assert the order the core sees, or only the plugin's own call list?
2) Claim split (08b2fb35): `grab_without_claim` then `grab` after the claim. Can a key be grabbed twice, or left ungrabbed when a claim is lost and won again (handover xp -> tiling -> xp), or when the claim never comes? Is the split one rule per look, or a special case each (KISS)?
3) 7fc05748: the early xp grab reads cube mode from the outputs. Is that the same predicate the scene uses later, or a second copy that can disagree (cube keys taken on faces, or missing on flat)?
4) Construct `regrab` before `build`: does regrab read any state build makes (hints, faces, pane map)?
5) Nothing loosened: tests/gate-lib.sh change is doc only? no wait, timeout or cell changed (diff tests/).
6) The report claims sh-44 fixed by order proof, not by a live miss (0 misses before and after). Is the claim honest and is sh-44 marked open or fixed correctly; sh-47 stays open.
Known, not findings: sh-32/sh-46 acts tiling I14 flake, docs SKIP (mkdocs absent), sh-43, sh-45.
Verdict computed: an open BLOCK or MAJOR is FIX, else MERGE-OK. Findings one line each (rfxr1-N, severity, file:line, text, fix in one sentence). Append to reports/flake-xp.md under "## Review r1".
Report to the PM session session_016iS3kLFcYmUQ3S8Zp7sDaN with send_message (at most 5 lines): verdict, BLOCK and MAJOR list. Poll every command in the same turn; never end a turn to wait.
Rules: push only claude/session-log (fetch and merge on reject, no force), only that report; author and committer Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com> (git -c user.name -c user.email); no attribution lines; never --no-verify.
