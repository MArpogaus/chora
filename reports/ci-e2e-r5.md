# ci e2e r5 (claude/ci-e2e3), sonnet, 11-15

Branch claude/ci-e2e3 after the merge of the session branch (886cbb1e). tests/merge-gate.sh passed as one run
(27 steps) on the second try; the first try failed in `test` on the unit test
words::tests::a_slow_backtrace_is_not_cut_by_the_alarm, a known flake (F16-F19 class, no file of this round
touches it).

| item | commit | what |
|---|---|---|
| MAJOR 1 | 2bc70557 (red), 977db9cf | `skip` in tests/gate-lib.sh exits 1 and prints the reason when CI is set; selftest case: no CI exits 0, CI=true fails with the reason. Gate 0.7-6 needs no check of its own. |
| MAJOR 2 | 157f511c | gate 0.8-10 header, docs/roadmap.md:236 and the ci.yaml step name say a user client puts the text node under screen:NAME; the gate also asserts the confined create under screen:* is refused ("no screen:* parents"). |
| MINOR | 4a5f9655 | e2e-sequence.sh fails on an empty command list and on a multi-line run in the e2e job; an allowlist entry needs a non-empty reason and matches the name literally (awk, no regex); with CI set a missing font fails tests/e2e-fonts.sh, comment corrected. |
| COSMETIC | 4a5f9655, ad5041d0 | merge-gate.sh header line fixed; one free_apply (gate_mcp.py), imported by mcp-client.py. |

CI tools: every tool of the e2e scripts (sway, foot, jq, desktop-file-utils, pipewire, xdg-desktop-portal-wlr,
xwayland-satellite, Xwayland, xeyes, thunar, python3-gobject-base) is in the dnf line of ci.yaml, so no step needs a
known-issues entry; a tool that goes missing there now fails its step instead of skipping.
Not checked here: the Fedora 44 run itself (xkeyboard-config of CI has fkeys; if not, gate 0.7-6 now fails there).
