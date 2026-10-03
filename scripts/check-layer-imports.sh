#!/usr/bin/env bash
# Warn when a file imports from a higher layer.
#
# Portable by design: takes file paths as arguments and uses nothing but POSIX
# tools, so any agent or CI step can call it. The Claude Code adapter in
# `.claude/hooks/` is a thin wrapper that extracts paths from its JSON stdin and
# calls this.
#
# The authoritative gate is `scripts/check-boundaries.sh`, which reads each
# crate's Cargo.toml and also enforces the full table in AGENTS.md. This is the
# fast in-editor approximation, and it covers a subset: the four crates below.
# When they disagree, `check-boundaries.sh` wins.
#
# Usage: scripts/check-layer-imports.sh <file.rs> [more.rs ...]
# Exit:  0 always (advisory), 1 on bad usage.

set -euo pipefail

[ $# -ge 1 ] || {
  echo "usage: $(basename "$0") <file.rs> [more.rs ...]" >&2
  exit 1
}

status=0

for FILE in "$@"; do
  [ -f "$FILE" ] || continue

  # Only Rust sources under crates/ have a layer.
  case "$FILE" in
  */crates/*.rs) ;;
  *) continue ;;
  esac

  CRATE=""
  case "$FILE" in
  */crates/math/*) CRATE="math" ;;
  */crates/topology/*) CRATE="topology" ;;
  */crates/operations/*) CRATE="operations" ;;
  */crates/io/*) CRATE="io" ;;
  */crates/wasm/*) CRATE="wasm" ;;
  *) continue ;;
  esac

  VIOLATIONS=""

  check_import() {
    local forbidden="$1"
    local label="$2"
    if grep -q "use ${forbidden}" "$FILE"; then
      VIOLATIONS="${VIOLATIONS}  - imports ${label} (not allowed at this layer)\n"
    fi
  }

  case "$CRATE" in
  math)
    check_import "brepkit_topology" "brepkit-topology"
    check_import "brepkit_operations" "brepkit-operations"
    check_import "brepkit_io" "brepkit-io"
    check_import "brepkit_wasm" "brepkit-wasm"
    ;;
  topology)
    check_import "brepkit_operations" "brepkit-operations"
    check_import "brepkit_io" "brepkit-io"
    check_import "brepkit_wasm" "brepkit-wasm"
    ;;
  operations)
    check_import "brepkit_io" "brepkit-io"
    check_import "brepkit_wasm" "brepkit-wasm"
    ;;
  io)
    check_import "brepkit_wasm" "brepkit-wasm"
    ;;
  esac

  if [ -n "$VIOLATIONS" ]; then
    echo "⚠️  LAYER BOUNDARY VIOLATION in ${CRATE} crate: $FILE"
    printf "%b" "$VIOLATIONS"
    echo "See AGENTS.md 'Layer dependency rules' for allowed imports."
    status=1
  fi
done

exit $status
