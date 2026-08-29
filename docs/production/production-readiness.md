# CorexPM Production Readiness & Independent Security Review Checklist

This document details the operational standards, threat mitigations, and security verification checklist for CorexPM `1.0.0` stable release.

## Security Review Checklist

- [x] **Archive Path Traversal Mitigation**: `corex-fetch` and `corex-store` strictly validate tarball paths against target directory boundaries, blocking relative `../` traversal or absolute path overwrites (`CXSEC0002`).
- [x] **Package Integrity Verification**: All downloaded archives are hash-verified against registry integrity specs (SHA-512 / SHA-256) prior to extraction or committing to the CAS (`CXSEC0001`).
- [x] **Immutable CAS Objects**: Committed package objects in `~/.corex/store/v1/packages/sha256/` are set to read-only.
- [x] **Lifecycle Script Policy**: Lifecycle scripts (`preinstall`, `postinstall`) are denied execution by default (`corex-policy`). Scripts run only when explicitly approved via `corexpm trust approve <pkg>`.
- [x] **Secret Redaction**: Environment variables and registry authentication tokens are automatically redacted from diagnostic logs and stdout output (`corex-scripts`).
- [x] **Platform Security Capability Enforcement**: Evaluates platform-level sandboxing (e.g. Linux Landlock / macOS AppSandbox / Windows AppContainer) and reports enforcement capabilities via `corexpm permissions`.

## Production Operational Standards

1. **Hermetic CI Installs**: Always use `corexpm ci` or `corexpm install --frozen` in production CI/CD pipelines to guarantee deterministic installs.
2. **Global Store Maintenance**: Run `corexpm store prune --grace-period 86400` periodically in automated cleanup jobs to reclaim unreferenced CAS packages safely.
3. **Audit Gates**: Run `corexpm audit --severity high` in pull-request pipelines to block vulnerable dependencies.
