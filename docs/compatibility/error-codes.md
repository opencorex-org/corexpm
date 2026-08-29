# Stable Diagnostic Error Code Catalog

CorexPM uses structured, zero-padded diagnostic error codes across all domain crates. Error codes provide actionable context and remediation guidance.

## Error Family Prefixes

| Prefix | Domain Family | Crate Owner | Description |
| --- | --- | --- | --- |
| `CXCLI` | Command Line Interface | `corex-cli` | Invalid arguments, missing parameters, CLI invocation errors. |
| `CXREG` | Package Registry | `corex-registry` | Network connection failures, 404 missing packages, invalid registry metadata. |
| `CXRESOLVE` | Dependency Resolver | `corex-resolver` | Unresolvable dependency cycles, peer dependency mismatches, tier incompatibilities. |
| `CXSTORE` | Content-Addressed Store | `corex-store` | Store lock timeouts, hash mismatches, corrupted package objects. |
| `CXLOCK` | Lockfile Engine | `corex-lockfile` | Lockfile syntax errors, version mismatches, foreign lockfile import errors. |
| `CXSEC` | Security & Integrity | `corex-security` | Archive path traversal attempts, tamper detection, integrity verification failures. |
| `CXSCRIPT` | Lifecycle Script Execution | `corex-scripts` | Denied script execution, script process non-zero exit codes. |
| `CXWORK` | Workspace Graph | `corex-workspace` | Workspace cycle detection, invalid workspace glob patterns. |
| `CXAUD` | Advisory Audit | `corex-audit` | Vulnerability severity filtering and advisory match reports. |

## Error Catalog Index

| Code | Family | Description | Actionable Guidance / Help |
| --- | --- | --- | --- |
| `CXCLI0001` | CLI | Missing or unknown subcommand/argument | Check `corexpm --help` for usage details. |
| `CXCLI0002` | CLI | Working directory read failure | Ensure read permissions exist for working directory. |
| `CXREG0001` | Registry | Package metadata request 404 | Verify package name and registry URL. |
| `CXRESOLVE0001` | Resolver | Unresolvable dependency constraint | Run resolution audit or inspect conflicting peer dependencies. |
| `CXSTORE0001` | Store | Store lock acquisition timeout | Ensure no concurrent CorexPM process holds global store lock. |
| `CXLOCK0001` | Lockfile | Lockfile JSON syntax error | Check `corex.lock.json` format or re-run `corexpm install`. |
| `CXLOCK0002` | Lockfile | Unsupported lockfile schema version | Upgrade CorexPM to parse newer lockfile formats. |
| `CXLOCK0010` | Lockfile | npm `package-lock.json` parse error | Verify `package-lock.json` syntax. |
| `CXLOCK0012` | Lockfile | Foreign lockfile read failure | Ensure foreign lockfile has read permissions. |
| `CXLOCK0013` | Lockfile | Foreign lockfile not found | Ensure a supported foreign lockfile exists in project root. |
| `CXSEC0001` | Security | Integrity hash mismatch | Package archive content sha512 differs from expected metadata. |
| `CXSEC0002` | Security | Path traversal attempt in archive | Rejected tarball entry attempting to write outside target directory. |
| `CXSCRIPT0001` | Script | Lifecycle script execution denied | Run `corexpm trust approve <package>` to permit lifecycle script. |
