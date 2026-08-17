# Fuzzing

Fuzz targets will cover the lockfile parser, manifest parser, semantic version
ranges, dependency resolution, and archive metadata/extraction. Each target
must document its input grammar, invariants, seed corpus provenance, resource
limits, and how to turn a finding into a permanent regression test.

Fuzz tooling is introduced with the parser/resolver and security milestones;
empty targets are intentionally not committed in the bootstrap scaffold.

