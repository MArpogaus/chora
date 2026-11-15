# ci f25 r1: not reproduced, no patch

Branch claude/ci-f25 exists at 2d9f306f with no commits. Nothing pushed except this report.

## What the CI log says (run 43699349842, e2e job, 2d9f306f)
- step 6 starts 09:33:47.67 and fails 09:33:49.03, 1.4 s in; points 320,400 and 960,400 passed, 5,5 failed.
- steps 1-5 took 11 s (step 4 waits >= 9 s), so startup is long over: not "the bar appears late".
- tests/agent.sh starts no waybar and no bar; pgrep waybar is 0 in every local run. The waybar.sh 10 s guess in F25 does not apply to this test.
- diff 61d885f0..2d9f306f of tests/agent.sh touches steps 1 and 18 only, not step 6.

## What pixel 5,5 is
- Nodes covering 5,5: tf-s2 (glow of the focused left window, -24..671), ts-s2 (shadow), the wallpaper.
- Locally the corner reads 10 8 15 on grim and capture alike with the left window focused, 0 0 0 with the right one focused (focus clicks, 5 samples each). 19 16 29 shows at 1270,790 with the right window focused.
- CI host 19 15 28 is 1.9x the capture 10 8 15 on all three channels: it looks like the glow blended twice on the host frame, or a frame with a different glow state. Both are host-side or state-side, the capture value is the one that matches every local run.

## Runs (local, container; stress-ng is not installed and lanes install nothing, so load = 8 `yes` processes on 4 cores)
- agent.sh cold: 2 of 2 PASS (step 6 included).
- 3 loops of capture vs grim at 5,5 before/after, 30 samples each, under load: always equal, 10 8 15.
- early start sampling (host/capture every ~0.3 s from the first frame): only transient is host 63 63 63 vs capture 0 0 0 in the first sample before chora paints; then 10 8 15 on both.
- CHORA_FRAME_BUDGET_US=1 (watchdog drops all effects): both 0 0 0, consistent, so F23-class dropping does not make the mismatch.
- Damage at the corner (translucent rect updated 6 times, focus flips, layout flips): host never accumulates.

## Verdict
Neither branch of step 1 of the brief could be shown: grim before/after never differed from the capture here, so I cannot say timing vs capture bug. No red commit exists, so no fix was made (a "wait for stable frame" would hide a possible host-presentation double blend on the CI renderer).

## Next (needs the CI artifact or a CI-like renderer)
1. Download artifact e2e-logs of run 43699349842 (failure.png, chora.log, failure.json) and read the tf-* / ts-* opacity and the log around 09:33:48; the lane has no artifact download.
2. Run the same step on Fedora 44 with the CI renderer (llvmpipe vs pixman in the nested chora) and sample host 5,5 several times in step 6.
3. If the host reads 19 15 28 stably while the capture reads 10 8 15: core finding (host path blends the glow twice), not a harness fix.
4. Earliest gate that would catch it: the e2e job of CI itself; merge-gate.sh has no agent.sh step (CI4), so it cannot.
