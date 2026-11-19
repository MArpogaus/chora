
## r1

Branch claude/nits-3, head 0b5f8c34 (merged claude/f24-rest 7231b51a and claude/kit-minors e4208314; the session branch had nothing new).
Gate `--jobs 3 --only pre-commit,ci-check,build,test,clippy,xp` all PASS (run /tmp/merge-gate.gURy9g).

1. kit5 r2-3: red 2234bbd5, fix 36ba05e4. `aside()` takes the output size; a card with no room beside the chat steps above it from a bottom corner, below from a top corner, else keeps the old clamp (a chat covering the whole height). Test: `the_card_steps_aside_of_the_chat_panel` (extended `wide` with a broad chat, bottom and top corner).
2. kit-minors NIT: renamed to `the_critical_ids_are_capped_at_64` (plugins/chora-mc/src/agent.rs). It sits in the red commit 2234bbd5, not a commit of its own.
3. F24R-1: red 39014c94 (tests/grab-gap-check-selftest.sh: early return, grab only inside an if, loop passes), fix 0b5f8c34. The check reads a grab callee of the same file and flags an early return or a first grab inside an if/else/match. Limits, documented in the selftest: a grab in a loop passes, a callee in another file is not read. Repo is clean.
Note: the first red test had a wrong top-corner case (no overlap); fixed in the fix commit.
