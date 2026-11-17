# review decisions r2

Branch claude/decisions, head 80a5931 (docs/decisions.md only, +24 lines, D78-D94). Verdict: FIX (one open MAJOR).

## Checked
- All 28 cited commits exist and match their claims (subjects, f67ad247 message numbers 11.8 s / 4.3 s, merges 13f86740, c949cc8).
- Code matches: schedstat per thread and once-a-minute host-busy log (safety.rs); sender icon caps 512 KiB / 512 px, O_NONBLOCK, regular file, private copy (kit icons.rs); logs 4 MiB, 3 old files, 0700/0600 (chora-scene logs.rs, docs/logs.md); mc root `Move index 0` (mc main.rs:3417); EDITORS is 12 entries and the chain matches D85; tiling.yaml `effects: [wobbly]`; confirm refused without a run_program rule (voice config.rs:1145); `(hidden)` for llm_headers.
- D87-D94 match answers.md 04:38 line and debt-rc 3b D1-D8 option by option (timing: D2 before, D1/D3/D8 after, D7 "sprint 1" not stated, fine). No invented decision. D82-D86 are supported by the code.
- Format: table columns, plain English, Commits cites match the existing style. Number order inside a table is already irregular in the file (D66/D67 after D77), so not a finding.
- The four disagreements in decisions-r1.md are real: (1) architecture.md:529 "Only the main thread counts" against f67ad247; (2) configuration.md:1819 ties the 30 s limit to program start, apps.rs `editor` has none; (3) 13f86740 has no chunk logic, fc7d69d is the agent-log redaction; (4) no separate commit for the handover, 18795b9 is the mc root change.

## Findings
- MAJOR D78: says "each window manager, mc, notify and the assistant has the setting `test_accept_injected_keys`". False. Only construct, mc, tiling and xp read the setting (`.accept_injected(...)` in camera.rs, mc main.rs:1626, tiling main.rs:579, xp wm.rs:713). Notify and the assistant have no such setting; they honor only the env var `CHORA_TEST_ACCEPT_INJECTED_KEYS`, and the notify test pins it off. Fix: name the four plugins with the setting and say notify and the assistant follow the env var only.
- MINOR D78: the list of flagged events leaves out `node.key`, which carries `injected` (event.rs:179, kit runtime.rs `from_person`), while the text says "every input event". Add `node.key`.
- MINOR D78: "a gate runs with it off" is unclear (the gate that tests the refusal?); name `tests/gate-injected-presses.sh`.
- COSMETIC: D87-D94 reasons repeat "decided 2026-11-17 by the owner" in every row; the section intro already says it. D89 cites "(D3)" meaning the API-stability decision, which reads like the audit's own D3 inside this section; write "(D3, stable API only from 1.0)".
- Note, not a finding: architecture.md:525-531 now contradicts D82 and the code; fix it in a docs change (D82 is right).

## Refactor proposals
- Keep the log of which plugins honor `test_accept_injected_keys` in one place (kit `Runtime`), so docs need not list them by hand (S).
- decisions.md rows of 1000+ chars (D62, D65, D78, D81): move the detail to the Designs prose (S).

## Re-review bbf1264
- Diff 80a5931..bbf1264 touches docs/decisions.md only (10 lines changed: D78, section intro, D87-D94).
- D78 fixed and true to code: node.key added; all nine flagged events listed; the setting is named for construct, mc, tiling and xp (the four `.accept_injected(...)` callers); notify and the assistant follow only the env var (kit `Runtime::with` reads it for every plugin); `tests/gate-injected-presses.sh` runs with the switch off. The COSMETIC findings are fixed (repeated "decided by the owner" removed, D3 qualified).
- D87-D94: only the "proposed" wording changed (intro says proposals; each row ends "May be revised or reverted when a later review finds a better design"; "decided ... by the owner" removed). The decision text is unchanged. Nothing else changed.
- Open findings: none. Verdict: MERGE-OK.
