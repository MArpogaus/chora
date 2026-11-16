# Review harness rc r2 (3641d5e5), PM re-review of a 6-line diff, 11-16 03:45 UTC

Reviewer: the PM (not the builder); same-day re-review of the r1 FIX points, diff read in full.
1. r1 MAJOR closed: ci.yaml e2e runs tests/gate-tiling-bar-early-signal.sh (after the Apps cells);
   the lane's merge-gate e2e step ran it as cell 25, PASS.
2. r1 MINOR closed: waybar.sh traps USR1 USR2 RTMIN+1 by name in the parent before the fork and
   resets after it; the child inherits the ignore across exec (POSIX), waybar installs its own
   handlers later (r1 review). Shebang now bash (dash has no RTMIN+1); bash is in every target image.
3. The lane's 2d-guard failed on a shallow clone (same as security a r5); the train gate reruns it.
Verdict: MERGE-OK
