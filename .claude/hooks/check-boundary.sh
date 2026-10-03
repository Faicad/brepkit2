#!/usr/bin/env bash
# Claude Code PostToolUse adapter → scripts/check-layer-imports.sh
#
# The check itself is vendor-neutral and lives in scripts/. This file only
# translates Claude Code's hook protocol (a JSON payload on stdin) into a plain
# argument list. The authoritative gate remains scripts/check-boundaries.sh,
# which reads Cargo.toml and covers every crate; this is the fast in-editor
# approximation.
#
# To use it, add to .claude/settings.json (not committed here):
#   "hooks": { "PostToolUse": [{ "matcher": "Edit|Write",
#     "hooks": [{ "type": "command",
#                 "command": "$CLAUDE_PROJECT_DIR/.claude/hooks/check-boundary.sh" }] }] }

set -euo pipefail

INPUT=$(cat)
FILE=$(jq -r '.tool_input.file_path // .tool_input.filePath // empty' 2>/dev/null <<<"$INPUT")
[ -z "$FILE" ] && exit 0

exec bash "$CLAUDE_PROJECT_DIR/scripts/check-layer-imports.sh" "$FILE"
