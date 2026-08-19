# Supported Platform Tier Policy

This document defines the supported platforms and compatibility tiers for CorexPM.

## Compatibility Tiers

CorexPM categorizes platforms into three support tiers.

### Tier 1: Fully Supported & Tested (CI Covered)

These platforms represent the primary target environments. Every commit must compile, pass all unit and integration tests, and undergo verification on these platforms.

- **macOS** (Apple Silicon `arm64` & Intel `x64`)
- **Linux** (Kernel 4.18+, `x64` & `arm64`) using `glibc` or `musl`
- **Windows** (Windows 10/11, `x64`)

### Tier 2: Supported (Limited CI)

These platforms are supported and expected to work, but are not tested on every commit. They are verified prior to minor releases.

- **Windows** (Apple Silicon / Qualcomm `arm64`)
- **FreeBSD** (`x64`)

### Tier 3: Best Effort (Community Supported)

These platforms may work, but we rely on community contributions to address issues. No official verification or support is guaranteed.

- Other Unix-like operating systems (OpenBSD, NetBSD)
- Alternative CPU architectures (32-bit x86, RISC-V, PowerPC)
- Non-standard runtimes or environments

## Platform Exclusions and Detection

CorexPM checks the host platform on startup or through the `doctor` command. Running CorexPM on an unsupported (Tier 3) platform will trigger warning diagnostics, guiding users to appropriate documentation.
