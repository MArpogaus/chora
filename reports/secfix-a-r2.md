# secfix a r2 (bf5904f4 on claude/secfix-a), sonnet

Fixed both MEDIUM of review-secfix-a-r1 in crates/chora-kit/src/icons.rs.
- FIFO: sender path opened with O_NONBLOCK, fstat on the handle, is_file required. Red test: a FIFO path falls back within 1 s (failed before, passes after).
- TOCTOU: bytes read once (bounded 512 KiB), validated, written to a 0600 copy under $XDG_RUNTIME_DIR/chora-icons-<pid>/ (dir 0700); the renderer gets the copy. Last 16 copies kept, all removed on drop. Red test: sender file swapped to an SVG after for_sender; the copy is still the PNG (failed before).
- Boundary tests: 512x512 passes, 513 wide/high fail, 512 KiB passes, +1 fails, empty falls back. sh-16 marked fixed.
- The branch had the session branch merged first (b58d3a0e).
Gate (tests/merge-gate.sh once on head): all lanes PASS except doors: mc screenshot-door/volume-door "missing" (no grim / set-mute after the click). mc does not use Icons. Rerun gate-doors.sh alone on head: fail, pass; on the base b58d3a0e: pass once, two runs timed out in my wrapper. Treated as a flake of the mc door click, not of this change; logs /tmp/claude-0/gate.log, doors2.log, doors7.log (container-local).
