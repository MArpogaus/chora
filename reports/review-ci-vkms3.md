# Review: ci-vkms3 (b5266fa4), 2026-11-15

Reviewer: independent opus subagent, read-only (builder: PM).
- vkms since Linux 6.15 is a faux device (/sys/devices/faux/vkms, driver faux_driver, no unbind
  file: EACCES). The new device-name check skips it; older kernels still match driver vkms.
- hyperv_drm and simpledrm cards are still unbound with the same value as before; connector
  entries (card0-Virtual-1) and driverless cards are skipped by the -e test; bash -e is safe.
- MINOR: a faux DRM card not named vkms would still fail; `[ -e "$driver/unbind" ] || continue`
  is sturdier. Not needed now (backlog CI7).
- Proof: only the vkms job of PR #106 runs it.
Earliest gate (routines): none local (no vkms here); a dry-run of the loop against a recorded
sysfs tree of the runner would catch it (CI7).

Verdict: MERGE-OK.
