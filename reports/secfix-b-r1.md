# secfix-b r1

Branch claude/secfix-b, 4 commits on 66588880:
- 2751e09 test: coder and agent children get no API key variable (red)
- 03e96c5 fix: keep API keys from the coder and the agents (rab-60)
- b175a65 test: a confirm list needs a rule for run_program (red)
- c2a882a fix: require a confirm rule for run_program (rab-61)

Red: the_harness_gets_no_api_key_variable, the_agent_gets_no_api_key_variable
and a_confirm_list_needs_a_rule_for_every_program failed before the fixes
(the key tests saw HOME; check accepted a list without run_program).
Green: all three pass; cargo test -p chora-voice 242 passed.

rab-60: without_keys on the coder and ACP agent Commands (Coder::set_keys,
Settings.keys, also on settings reload); agent-sandbox.sh unsets the names
in AGENT_SANDBOX_UNSET (test in tests/agent-sandbox.sh); docs/voice.md updated.
rab-61: Config::check rejects a confirm list with no rule matching run_program
for every program (argv0-restricted rules do not count); docs/voice.md updated.

Gate: tests/merge-gate.sh, all 28 steps passed once on c2a882a, no skips
(log dir /tmp/merge-gate.q5bcbP). An earlier run failed 2d-guard only because
the clone was shallow; fixed by git fetch --unshallow.
Open: the session stop hook marks the commits Unverified (author is Marcel
Arpogaus per the brief, not noreply@anthropic.com); left as briefed.
No --fast gate exists yet, so the full gate was used.
