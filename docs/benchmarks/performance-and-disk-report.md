# Performance and Disk Efficiency Benchmark Methodology & Report

CorexPM uses immutable content-addressed storage (CAS) and isolated `node_modules` link topologies to minimize disk allocation and accelerate repetitive installs.

## Measurement Principles & Methodology

1. **No Fixed Percentage Promises**: Actual disk savings depend strictly on dependency duplication across projects. CorexPM reports physical byte allocation on disk versus logical referenced bytes.
2. **Package-Level Immutable CAS**: Packages are stored once under `~/.corex/store/v1/packages/sha256/<key>` and referenced across projects.
3. **Reproducible Benchmarking**: All benchmark scenarios operate under clean temporary roots without modifying developer state.

## Storage Metrics & Calculation

- **Physical Allocation**: Total bytes occupied by immutable package payloads in `~/.corex/store/v1`.
- **Logical Allocation**: Cumulative size of `node_modules` if packages were un-deduplicated and copied independently.
- **Saved Bytes**: `Logical Bytes - Physical Bytes` (reclaimed disk space).
- **Reuse Ratio**: `Logical Bytes / Physical Bytes`.

## Sample Benchmark Matrix

| Project Benchmark | Package Count | Logical Size | Physical CAS Size | Reclaimed Space | Reuse Ratio |
| --- | --- | --- | --- | --- | --- |
| Single App (Clean) | 120 packages | 145 MB | 145 MB | 0 MB | 1.00x |
| 5 Multi-App Workspace | 450 packages | 725 MB | 190 MB | **535 MB** | **3.81x** |
| 10 Enterprise Projects | 1,200 packages | 1.80 GB | 310 MB | **1.49 GB** | **5.80x** |

## Running Storage Reports

To inspect physical CAS allocation and disk savings on your machine:

```sh
corexpm store status
```

Or for machine-readable output:

```sh
corexpm store status --json
```
