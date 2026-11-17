# decisions r1 report

Branch claude/decisions, head 3cc1450 (one commit, docs/decisions.md only). pre-commit and `mkdocs build --strict` pass.

## Entries added
- D78 injected flag, D79 API keys out of agent env (+ confirm needs run_program rule), D80 sender icons (PNG caps, FIFO, private copy), D81 logs and chora-report, D82 watchdog wait of all threads, D83 mc root below other plugins, D84 placed pane stays in view, D85 Notepad fallback chain, D86 tiling wobbly default.
- D87-D94: debt-rc 3b D1-D8, status open, no decision text (new section "Open questions"). Not added: ti-12 (no focus-left/right key), it is in known-issues.md.

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
