# Release process

CorexPM releases are reproducible native artifacts, not just repository tags.

## Pre-release checklist

1. Confirm roadmap exit criteria and close or defer every release blocker.
2. Run formatting, linting, unit, integration, compatibility, security, and
   required platform checks.
3. Review dependency advisories, licenses, and source provenance.
4. Update the changelog, version, migration notes, and known limitations.
5. Build release artifacts in the release workflow from a clean tag.
6. Produce SHA-256 checksums, signatures, provenance, and an SBOM.
7. Install and smoke-test each supported artifact independently.
8. Publish the release and verify download, signature, and rollback guidance.

Initial target triples are macOS arm64/x64, Linux arm64/x64, and Windows
arm64/x64, subject to the accepted platform tier policy. Packaging via Homebrew
or WinGet follows trustworthy signed binary releases.

Stable releases require documented lockfile/config compatibility and a defined
security support window. The word `latest` must never be the only reproducible
installer input.

