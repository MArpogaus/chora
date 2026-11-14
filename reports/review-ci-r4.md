# Review ci r4 (claude/ci-r4 a3880296), PM, read-only, 11-14 23:1x UTC

Verdict: MERGE-OK. Cause: chora-mc fonts.rs runs fc-list, fc-scan and fc-match; the Fedora 44 font packages do not pull fontconfig, so file_of() finds nothing and the audit tests unwrap None. Red: Fedora font files with the fc tools disabled fail exactly these 2. Fix: check and coverage install fontconfig; CONTRIBUTING.md says why. Merge gate 24/24. PM decision: the packages require fontconfig (chora-mc needs it at run time); added to backlog PK1.
