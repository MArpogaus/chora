
## r1
Finding kc-1 fixed at f666a025 on claude/kc-fix (one commit on f7569c24).
Choice (a), the kit helper: `FakeCore::outputs(json)` plants the output.list answer. Why: the allow was only there to hide a string from the gate-0.8 grep; (b) would loosen a gate that guards every plugin, (a) is 5 lines in cfg(test) kit code and the grep stays strict.
Red first: kit test the_fake_core_answers_output_list_with_its_outputs failed to compile (no method outputs), then passed. on_fake_core now calls core.outputs(..); its allow is removed; allow count -1, no new allow.
Gate: pre-commit,ci-check,build,test,clippy,gate-lib all PASS (partial run, nothing pushed by it).

## PM review r1 (2026-11-19 21:42 UTC; PM, not the builder)
Diff f7569c24..f666a025, 3 files. kc-1 closed: FakeCore::outputs plants output.list; the assistant's disallowed_methods allow is gone (no new allow); kit test covers default-empty and planted list.
- kcr-1 MINOR (process): test and fix in one commit; rules.md 8 asks for the red test commit first (the gate's red-first accepts a test in the same commit). Not reworked: the test fails to compile without the method, so red is evident.
- kcr-2 NIT: on_fake_core still names its folder by tag and pid; lane temp-dirs moves such sites (expect a small conflict there; temp-dirs merges the session branch).
Verdict (computed): no open MAJOR -> MERGE-OK. Train 75.
