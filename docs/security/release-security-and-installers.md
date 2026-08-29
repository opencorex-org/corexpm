# CorexPM Release Security, Verification, & Rollback Guidance

CorexPM release builds adhere to strict supply chain integrity standards, binary checksum verification, and zero-downtime rollback policies.

## Release Verification & Checksums

Each CorexPM release artifact published to GitHub Releases or official channels includes:

1. **SHA-256 Checksums (`SHA256SUMS`)**:
   - SHA-256 hashes for all platform binaries (`corexpm-macos-arm64`, `corexpm-linux-x64`, `corexpm-windows-x64.exe`).
   - Verification command:
     ```sh
     sha256sum -c SHA256SUMS
     ```

2. **Cryptographic Signatures (`SHA256SUMS.sig`)**:
   - Signed with GPG / Minisign release key.
   - Verification command:
     ```sh
     minisign -Vm SHA256SUMS -p corex-pub.key
     ```

3. **Build Provenance Attestation**:
   - Every binary includes embedded SHA-256 manifest provenance verified by `corex_security::ProvenanceVerifier`.

## Installation Methods

### Unix Shell Installer (macOS / Linux)
```sh
curl -fsSL https://corex.dev/install.sh | sh
```

### Windows PowerShell Installer
```powershell
iwr -useb https://corex.dev/install.ps1 | iex
```

## Zero-Downtime Rollback Guidance

If a regression or compatibility issue occurs after updating CorexPM:

1. **Revert Binary Version**:
   - Download the previous known-good binary (e.g. `v0.8.0` or `v1.0.0`).
   - Replace `~/.corex/bin/corexpm`.
2. **Lockfile Continuity**:
   - `corex.lock.json` schema v1 is backward-compatible across 1.x releases.
3. **Store & Cache Isolation**:
   - Content-Addressed Store payloads (`~/.corex/store/v1/packages`) are immutable and isolated. Downgrading the CLI binary will not corrupt committed CAS packages.
