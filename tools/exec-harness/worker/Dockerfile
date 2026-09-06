# Container image for the tier-1 exec sandbox.
# The @cloudflare/sandbox runtime expects its own base image; keep the tag in
# lockstep with the package.json dependency version (verify both at
# activation — see README activation checklist).
FROM docker.io/cloudflare/sandbox:0.12.9

# Detonation runs must never reach the network: the harness asserts observable
# blast radius, not exfiltration behavior. Baseline hardening — activation
# checklist additionally verifies egress policy at the Cloudflare config layer.
ENV HARNESS_ROLE=exec-tier1
