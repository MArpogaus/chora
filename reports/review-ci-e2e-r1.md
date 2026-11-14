# Review ci e2e r1 (claude/ci-e2e ca33d5cb, 4300c2d4), opus, read-only, 11-14 22:1x UTC

Verdict: FIX (C1).
| id | sev | file:line | text |
|---|---|---|---|
| R1 | MINOR | tests/nested.sh:92 | .rects is the right geometry; an unmapped window (empty rects) reads null and passes; assert all(.rects \| length > 0) and three surfaces. |
| C1 | MAJOR | tests/nested.sh:477 | the stale frame of the previous shear step blends red and green at 925,500 to about 128,128,0, which also passes green >= 100 and green > red, so the depth check can pass on the old frame. PM note: the correct frame measured 119,136,0, so a margin (green - red >= 64) would fail the right render; the sample point or the frame wait must change instead. |
| C2 | MINOR | tests/nested.sh:359 | helper without comment, away from step 21. |
