# ci r1 (rc 4, rc 5, rc 7)

Lane: sonnet, cloud worker on branch claude/ci-rc from the session branch head after
train 4. One round. Stop at 250 k context and hand over.

Goal: everything CI and packaging need before the PM opens the draft PR.
1. rc 7: `cargo deny check` and `cargo machete` clean. Update dependencies where an
   advisory or licence fails; drop unused ones. If an advisory cannot be fixed, add a
   reasoned ignore with the advisory id in deny.toml.
2. rc 4: run every job of .github/workflows locally where possible (check, msrv,
   e2e, coverage, docs, pre-commit). Fix what fails. List the jobs that cannot run
   locally and why.
3. rc 5: build the packages the release workflow builds (RPM, Arch, distrobox image)
   as far as the container allows; install the RPM or Arch package into a clean
   container or chroot and start a headless session (smoke). Report what could not
   be built here.
Method: red first where a check failed; one commit per fix; the full merge gate
(tests/merge-gate.sh) green at the end.
Report: per check before/after, commits, what is left for GitHub CI.
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI
attribution lines; never --no-verify; stage by name; no force push; push only
claude/ci-rc; no pull request (the PM opens it).
