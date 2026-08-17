# Benchmarks

CorexPM benchmarks compare cold install, warm install, offline install, and
existing-tree reconciliation for small, medium, large, and monorepo fixtures.

Required measurements include wall time, CPU time, peak memory, network bytes,
disk writes, physical/logical disk size, and file count. Reports capture exact
hardware, OS, filesystem, runtime, package-manager versions, commands, cache
state, repetitions, and raw samples.

Planned scenario directories are `cold-install`, `warm-install`, `monorepo`,
`disk-usage`, `lockfile`, and `resolver`. They will be created with the 0.3 and
0.4 implementations so that fixtures reflect real code paths.

