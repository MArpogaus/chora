# harness r1 (rc 3, rc 4)

Branch claude/harness (from 8bcbdc73), pushed at f3bab766. Five commits:

- 7b4fd988 test(gate): prove the red-first rule as a commit-msg hook
- 384d68b8 feat(gate): run the red-first rule as a commit-msg hook
- 3453d14f chore(lint): refuse own core connections in plugins at clippy time
- cfa06e16 test(gate): prove --jobs of the merge gate
- f3bab766 feat(gate): run the steps after clippy in parallel with --jobs

## 1. Architecture rules at compile time
plugins/clippy.toml (clippy reads only the nearest clippy.toml, so it repeats
the root settings) adds disallowed-methods Conn::connect, Client::connect,
Client::from_env, Client::from_stream and disallowed-types chora_scene::ipc::Client.
Hook clippy-config-sync fails when a root setting is missing from the plugin file.
- Red: connect calls planted in chora-notify give "use of a disallowed method
  `chora_kit::conn::Conn::connect`", "`chora_scene::ipc::Client::from_env`" and
  "disallowed type `chora_scene::ipc::Client`" (rc 101). Root setting changed to
  too-many-lines-threshold = 61: the sync hook fails with "plugins/clippy.toml lacks: ...".
- Green: workspace clippy -D warnings rc 0, and the hook passes on the tree.
- Not moved: the method-name strings (events.subscribe, output.list) and the
  reconnect loops. Clippy cannot match those, so the grep gate keeps them.
- FINDING: tests/gate-0.8-plugin-runtime.sh (CI job check) is RED on the tree:
  plugins/chora-assistant/src/chat/host.rs:674 "events.subscribe" (10384acf) and
  plugins/chora-construct/src/cube.rs:1592 "output.list" (9a07833a). Not fixed
  (no plugin churn); it belongs to the ci check lane or a plugin lane.

## 2. Red-first at commit time
tests/red-first.sh holds the rule of the gate's red-first step (the merge gate
sources it). The new commit-msg hook `red-first` runs it on every commit. It
fails a fix/feat commit when no test is staged, no earlier commit in
RED_FIRST_BASE..HEAD changed a test, and the body has no "No test:" line.
Other types pass. tests/merge-gate-selftest.sh has 19 new hook checks: red
(14 checks exit 127 before the script existed), then green. In the repo,
RED_FIRST_BASE=HEAD git commit -m "fix(gate): planted violation" was refused
with "RED-FIRST MISSING: fix(gate): planted violation".

## 3. Gate time (this container, 4 CPUs)
Before, serial, measured at 8bcbdc73: 1883 s (31 min) cold. With a warm cargo
cache: cargo part 141 s + gate scripts 1512 s = 27.5 min.
Cut: `--jobs N` runs the steps after clippy up to N at once. Red-first to
clippy stay serial. A failed step stops the others and the push. The gate
scripts mostly wait. With 3 at once the step times grew by at most 5 %
(matrix 215 to 225 s, layout 165 to 173 s, doors 286 to 286 s).
After, measured with --jobs 3: 560 s up to the tiling-audit failure, and 286 s
for the steps it stopped. Steps in order: 141 s + about 517 s = **about
11 min**. Two lanes measured 793 s for all the gate scripts (every step PASS).
Default stays 1. Recommendation: the PM and the lanes run --jobs 3. --jobs 4
would give about 9 min but is untested.
Not built: --since. docs/*.md and README.md go into choractl and voice through
include_str!, so a docs-only diff can still change binaries and tests. A
diff-based skip would drop checks for little gain.
Merge-gate result at f3bab766, --jobs 3: every step PASS except tiling-audit.
It failed the same way serially on the base 8bcbdc73 (friction 47, tile
border paper-2560x1600@2), but it PASSED once in the two-lane run: a FLAKE
of timing, not only a fixed colour. So "merge-gate passes" is blocked by
that step only.

## 4. Builder checklist
dev-rules.md: "Self-check before you report", six lines: state cleared on every
exit, skips named, commit types, no hand-written CHANGELOG, docs in step,
CI-only checks run locally.

## Open
- The tiling-audit flake (owner: the tiling audit lane), and the red CI grep gate above.
- Turning --jobs 3 on by default is the PM's choice.
