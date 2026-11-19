
## r1
feedback/README.md: `task:` and `check:` lines in the finding form, high + `check: none` not a BLOCK candidate. triage.py: groups new findings by (flavor, check) with counts, marks HIGH, NO CHECK, `--format` exits 1 when a run file dated 2026-11-20+ lacks either line; old files read both as none. tests_triage.py: 4 tests green, ruff clean.

## PM review r1 (2026-11-19 22:58 UTC)
feef974: tests_triage.py 4/4 green here; `triage.py` groups 15 mc findings under (mc, none) with HIGH, NO CHECK marks; `--format` OK on today's files. Old files parse. Nit: --format also prints MISSING UX lines (harmless). Accepted; P2 item 3 done.
