#!/usr/bin/env bash
# Print a ripple-effect reminder when an enum definition file is edited.
#
# Adding a variant to a shared enum is a workspace-wide change: every `match`
# over it needs a new arm. This lists the known sites so they are visited while
# the change is fresh, instead of surfacing as a wall of clippy errors later.
#
# Portable by design: takes file paths as arguments, POSIX tools only. The Claude
# Code adapter in `.claude/hooks/` is a thin wrapper that extracts the path from
# its JSON stdin and calls this.
#
# The site lists reflect what AGENTS.md documents. They are a prompt to go
# look, not a claim that the list is complete — `rg` is the authority:
#
#   rg -n 'EdgeCurve::' crates/
#   rg -n 'FaceSurface::' crates/
#
# Usage: scripts/check-enum-ripple.sh <file.rs> [more.rs ...]
# Exit:  0 always (advisory), 1 on bad usage.

set -euo pipefail

[ $# -ge 1 ] || {
  echo "usage: $(basename "$0") <file.rs> [more.rs ...]" >&2
  exit 1
}

status=0

for FILE in "$@"; do
  case "$FILE" in
  */topology/src/edge.rs)
    echo "🔔 RIPPLE-EFFECT REMINDER: you edited edge.rs (contains the EdgeCurve enum)."
    echo "   If you added/changed a variant, check all match sites in AGENTS.md → 'Adding an EdgeCurve variant'."
    echo "   Key files: tessellate.rs, transform.rs, copy.rs, measure.rs, boolean.rs,"
    echo "   step/writer.rs, iges/writer.rs, kernel.rs (8 sites)."
    status=1
    ;;
  */topology/src/face.rs)
    echo "🔔 RIPPLE-EFFECT REMINDER: you edited face.rs (contains the FaceSurface enum)."
    echo "   If you added/changed a variant, check all match sites in AGENTS.md → 'Adding a FaceSurface variant'."
    echo "   Key files: tessellate.rs, transform.rs, copy.rs, section.rs, distance.rs,"
    echo "   boolean.rs (4 sites), step/writer.rs, iges/writer.rs, kernel.rs (8 sites)."
    echo "   ⚠️  offset_face.rs, step/writer.rs, iges/writer.rs have wildcard catch-alls!"
    status=1
    ;;
  */math/src/analytic_intersection.rs)
    echo "🔔 RIPPLE-EFFECT REMINDER: you edited analytic_intersection.rs (contains the AnalyticSurface enum)."
    echo "   Check match sites in analytic_intersection.rs (4 sites) and boolean.rs (4 sites)."
    status=1
    ;;
  esac
done

exit $status
