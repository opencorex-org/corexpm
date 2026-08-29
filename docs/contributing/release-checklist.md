# CorexPM Release Maintainer Checklist

Follow this checklist for each minor or major release of CorexPM.

## Pre-Release Validation

- [ ] Confirm roadmap exit criteria and close or defer open release blocker issues.
- [ ] Verify clean working tree (`git status`).
- [ ] Run `./scripts/release.sh --dry-run vX.Y.Z` to verify `fmt`, `clippy`, `test`, `build`, `npm pack`, and `SHA256SUMS`.
- [ ] Review `CHANGELOG.md` for complete release notes under `[vX.Y.Z]`.
- [ ] Verify `Cargo.toml`, `packages/corexpm/package.json`, and `package.json` version strings match `X.Y.Z`.

## Release Tagging & Publishing

- [ ] Execute `./scripts/release.sh vX.Y.Z` to create annotated git release tag `vX.Y.Z`.
- [ ] Push release commit and tags: `git push origin main --tags`.
- [ ] Publish npm package: `cd packages/corexpm && npm publish --access public`.
- [ ] Verify GitHub Actions release workflow completes and uploads binary artifacts (`.github/workflows/npm-release.yml`).

## Post-Release Verification

- [ ] Verify `npx corexpm@vX.Y.Z doctor` on a clean machine.
- [ ] Verify POSIX installer: `curl -fsSL https://corex.dev/install.sh | sh`.
- [ ] Verify PowerShell installer: `iwr -useb https://corex.dev/install.ps1 | iex`.
- [ ] Confirm `SHA256SUMS` and minisign signatures on GitHub Releases page.
