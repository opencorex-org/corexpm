# Threat model

## Protected assets

- developer source code, credentials, home-directory data, and workstation;
- CI secrets, build outputs, and release integrity;
- global store correctness and other projects that share it;
- dependency graph and lockfile reproducibility; and
- registry credentials and privacy-sensitive logs.

## Trust boundaries

Untrusted inputs include registry metadata, archives, manifests, lockfiles from
unreviewed changes, dependency lifecycle scripts, package binaries, plugin
code, proxy responses, and local filesystem state writable by other processes.
The registry is a transport and identity source, not authority to execute code.

## Principal threats and mitigations

| Threat | Initial mitigation |
| --- | --- |
| tampered download | expected-integrity verification before commit/execution |
| archive path traversal or link escape | validated streaming extraction into an isolated root |
| malicious lifecycle script | deny by default; explicit policy; minimal environment |
| shared-store mutation | immutable permissions, atomic commits, overlay builds |
| concurrent corruption | project/object locks, unique staging, atomic rename |
| poisoned cache/index | validate content on trust transition; rebuildable metadata |
| credential disclosure | separate credential source, URL/header redaction, log tests |
| dependency confusion | explicit registry/scope mapping captured in resolution |
| lockfile manipulation | frozen validation, integrity, reviewable deterministic diffs |
| symlink/junction attack | no-follow validation and platform-specific adversarial tests |
| resource exhaustion | size/count/depth limits, bounded concurrency, cancellation |

## Residual risk

Allowing a lifecycle script can grant substantial authority where operating
systems do not provide a dependable sandbox. CorexPM must describe whether a
control is enforced, detected, or merely declared. Trusting a package is not
equivalent to proving it safe.

This model is reviewed whenever a new network source, persistent format,
execution path, plugin hook, credential source, or filesystem mutation is
introduced.

