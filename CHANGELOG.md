# Changelog

All notable changes to this project are documented in this file.

This repository is a fork of [brepkit](https://github.com/andymai/brepkit) by
Andy Mai, taken at upstream tag `v2.129.15` (2026-08-07). The upstream release
history up to that point was removed from this file; see the upstream
repository for the full `2.x` changelog.

Upstream is dual-licensed MIT OR Apache-2.0, and this fork keeps that licensing
and the original copyright notices. See [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE).

## [Unreleased]

## [2.130.0] - 2026-10-11

### Added
- fix cone STEP semi-angle and add OCCT cross-kernel compat gate (e3fc3f0)

### Changed
- move skills to the vendor-neutral .agents/skills/ (1820d0a)

### Documentation
- record the chamfer corner-patch attempt and what it measured (ddafbf4)
- keep the degradation audit, drop the rest of the untracked pile (e036168)

### Fixed
- read SURFACE_CURVE family via 3D basis curve in STEP reader (ed5f2ff)
- commit the chamfer corner-patch implementation missing from the test-only commit (05b18c1)
- resolve rustdoc private-intra-doc-links and clippy lint failures (dba4ed5)
- exact ray-cast classification for circular face boundaries (6846344)
- exclude inner wire holes from parametric face integration (60da940)
- count cavity shells in solid volume, area and centroid (535c43c)
- honour circular boundaries exactly, not through their chords (E-04) (79b925f)
- trim each chamfered face on its own side, not one shared side (9108b8b)
- weld the fillet result into a closed shell (C-01) (d12e761)
- map path sampling into the curve's knot domain (D-04) (ca468ec)
- register cavity faces as an inner shell and classify through it (B-04) (b092817)
## [2.129.16] - 2026-10-02

### Fixed
- let a contact line trim through an existing boundary vertex (42539e2)
- give fillet_v2 the correct box-fillet volume (3d8743e)
