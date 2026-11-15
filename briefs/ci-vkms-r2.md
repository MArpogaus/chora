# ci vkms r2 (rc 4): the vkms job on any runner

Lane: sonnet, cloud, new branch claude/ci-vkms2 from the session branch (8bcbdc73). One round. Stop
at 250 k context and hand over. Read dev-rules.md and conventions.md first. Setup as briefs/ci-r1.md.
The vkms job of PR #106 now passes its first steps on an AWS runner (kernel 7.0.0-aws, no Hyper-V),
then tests/gate-0.9-vkms-hotplug.sh fails "timeout: count 1" (run 43661778583, job 132652477090;
the log shows card0 and card1 and "Error importing dmabuf from card0 to card1 ... cpu-copy"): on
this runner another DRM card exists besides vkms, so chora takes card0 as its primary GPU and the
vkms hotplug output is not counted. Earlier runs on Hyper-V runners passed after the hyperv_drm unbind.
Find the runner's card0 (read the job log: `ls -l /dev/dri` output and the drivers), and make the job
runner-agnostic: either unbind any non-vkms DRM device generically, or point chora at the vkms device
explicitly (a supported setting, documented), whichever is simpler and true to how users run chora.
Never weaken the hotplug check. You cannot run the job here; reason from the logs, keep the change
small, and say what only GitHub proves.
Done when: tests/merge-gate.sh passes (ci.yaml lint, pre-commit). Report to the PM
session_016iS3kLFcYmUQ3S8Zp7sDaN (at most 8 lines).
Rules: author Marcel Arpogaus <38564291+MArpogaus@users.noreply.github.com>; no AI attribution lines;
never --no-verify; stage by name; no force push; push only claude/ci-vkms2.
