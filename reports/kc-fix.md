
## r1
Finding kc-1 fixed at f666a025 on claude/kc-fix (one commit on f7569c24).
Choice (a), the kit helper: `FakeCore::outputs(json)` plants the output.list answer. Why: the allow was only there to hide a string from the gate-0.8 grep; (b) would loosen a gate that guards every plugin, (a) is 5 lines in cfg(test) kit code and the grep stays strict.
Red first: kit test the_fake_core_answers_output_list_with_its_outputs failed to compile (no method outputs), then passed. on_fake_core now calls core.outputs(..); its allow is removed; allow count -1, no new allow.
Gate: pre-commit,ci-check,build,test,clippy,gate-lib all PASS (partial run, nothing pushed by it).
