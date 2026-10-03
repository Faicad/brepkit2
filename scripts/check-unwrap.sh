#!/usr/bin/env bash
# Warn about `.unwrap()` / `.expect()` in non-test Rust code.
#
# Workspace lints already deny these (see the `[workspace.lints]` deny list), so
# this exists to point at the exact line the moment it is written rather than at
# the end of a build. It is a nudge, not a gate: `cargo clippy` is the gate.
#
# Portable by design: takes a file path as an argument and uses nothing but
# POSIX tools, so any agent or CI step can call it. The Claude Code adapter in
# `.claude/hooks/` is a thin wrapper that extracts the path from its JSON stdin
# and calls this.
#
# Usage: scripts/check-unwrap.sh <file.rs> [more.rs ...]
# Exit:  0 always (advisory), 1 on bad usage.

set -euo pipefail

[ $# -ge 1 ] || {
  echo "usage: $(basename "$0") <file.rs> [more.rs ...]" >&2
  exit 1
}

status=0

for FILE in "$@"; do
  [ -f "$FILE" ] || continue

  case "$FILE" in
  *.rs) ;;
  *) continue ;;
  esac

  # Test code is allowed to unwrap.
  case "$FILE" in
  *test_utils* | */tests/* | */benches/* | */examples/*) continue ;;
  esac

  # Everything from the first `#[cfg(test)]` onward is test code.
  TEST_LINE=$(grep -n '#\[cfg(test)\]' "$FILE" 2>/dev/null | head -1 | cut -d: -f1 || true)

  if [ -n "$TEST_LINE" ]; then
    REGION=$(head -n "$((TEST_LINE - 1))" "$FILE" || true)
  else
    REGION=$(cat "$FILE")
  fi

  [ -n "$REGION" ] || continue

  BASE=$(basename "$FILE")

  if grep -q '\.unwrap()' <<<"$REGION"; then
    echo "⚠️  Found .unwrap() in non-test code of $BASE — use Result instead."
    status=1
  fi

  if grep -q '\.expect(' <<<"$REGION"; then
    echo "⚠️  Found .expect() in non-test code of $BASE — use Result instead."
    status=1
  fi
done

exit $status
