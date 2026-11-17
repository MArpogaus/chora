# decisions r1 report

Branch claude/decisions, head 80a5931 after r2 (r1 3cc1450 plus one commit, docs/decisions.md only). pre-commit and `mkdocs build --strict` pass.

## Entries added
- D78 injected flag, D79 API keys out of agent env (+ confirm needs run_program rule), D80 sender icons (PNG caps, FIFO, private copy), D81 logs and chora-report, D82 watchdog wait of all threads, D83 mc root below other plugins, D84 placed pane stays in view, D85 Notepad fallback chain, D86 tiling wobbly default.
- D87-D94: debt-rc 3b D1-D8; r1 had them open, r2 (owner, 2026-11-17 04:38) records the chosen option and consequences (section "Decisions of the debt audit"). Not added: ti-12 (no focus-left/right key), it is in known-issues.md.

## r2
- D88 timing: brief says nothing, answers.md says before the tag; D88 follows answers.md.
- D94 is recorded without a docs/api.md change, as the brief asks. D57 keeps its fixed 4000 until implemented.

## Code and docs disagree (not decided)
1. docs/architecture.md:525-531 still says only the main thread's wait counts; f67ad247 sums all threads (D82 follows the code).
2. debt-rc 3c item 8 ties a "30 s limit and exit-status file" to the Notepad fallback. The fallback code (apps.rs `editor`) has neither; the 30 s limit belongs to program start (configuration.md:1819, D1 question). D85 omits it.
3. debt-rc 3c item 2 says "secrets split across chunks"; secfix-b (13f86740) shows no chunk logic. It removes key variables and requires a run_program confirm rule. Chunk redaction is not in D79; the agent-log redaction is in fc7d69d (voice.md), not in D81.
4. Handover above the mc page (item 6): carried by 18795b9 (mc root to index 0); no separate commit found.

## Merges since 11-14 left out (design weight, no entry)
- 81fbea4 (core-r2/security): output.configure and output.reserve gated on methods.privileged; D33 does not list it.
- Doors merge d349a6d (claude/doors): doors have no D-entry (0 hits for "door" in decisions.md).
- Notify body cap (4e3c019) and the injection mark drawn in front of the cursor (90842d7): small, no entry needed.
- Pure ci, flakes, media, pk1 merges: no design weight.

## Refactor proposals
- docs/decisions.md lines ~48-94: rows in "Core" and "Tools" tables are not in number order and rows run 1000+ chars; split long rows into the "Designs" prose form (S).
- plugins/chora-xp/src/launch.rs and apps.rs: editor lookup and launch limits are separate mechanisms for "program found" (D87 question) (M).
- injected-input handling repeats a `hears` guard in each plugin (tiling, xp, construct `hears_pointer`, mc, notify, assistant); move to a chora-kit helper (M).

## r3
- D87-D94 now read "proposed (owner, 2026-11-17)" with the sentence that a later review may revise or revert them; head 5ed35a5, pre-commit and mkdocs --strict pass.

## r4 (review of r2)
- MAJOR D78 fixed (decisions.md:73): setting only in construct, mc, tiling, xp (`.accept_injected`); notify and assistant follow the env var only.
- MINOR node.key fixed in the D78 event list (also pressed/moved/released named).
- MINOR gate named: tests/gate-injected-presses.sh (runs with the switch off).
- COSMETIC fixed: "proposed (owner, 2026-11-17)" once in the intro, "debt-rc 3b D1 to D8", D89 cites "(D3, stable API only from 1.0)". Head bbf1264.
