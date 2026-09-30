# docs/AGENTS.md — Documentation Standard

This file defines the documentation standard for the brepkit2 repository. Read this before writing or editing any documentation.

## Tier taxonomy

| Tier | Path | Purpose | Bilingual | Notes |
|---|---|---|---|---|
| Standing docs | `docs/*.md` | Current-state reference: architecture, API, workflows | Yes (`.md` + `.zh.md` + `.i18n.yaml`) | One home per fact |
| mdBook guide | `book/src/*.md` | User-facing guide rendered by mdBook | No | Linked from `book/src/SUMMARY.md` |
| Plans | `docs/plans/` | Design documents: requirements, technical analysis, implementation plan | No | Archived monthly into `yyyy-mm/` folders |
| Analysis | `docs/analysis/` | Technical analysis/comparison documents | No | No user requirements |
| Agent Notes | `.agents/notes/` | Decision records: why, what was given up, alternatives | Yes | See [README.md](../.agents/notes/README.md) |
| Skills | `.claude/skills/` | Reusable workflows | No | SKILL.md per skill |
| Root instructions | `CLAUDE.md`, `AGENTS.md` | Session-level standing instructions | No (English only) | |
| Subtree instructions | `*/CLAUDE.md`, `*/AGENTS.md` | Subtree-specific instructions | No (English only) | |

## One home per fact

Each rule has exactly one authoritative home. Other locations link, never duplicate. If a fact changes, update the authoritative source — do not add a changelog entry.

## Record current state, not history

Documentation describes the current state of the system. Change history lives in commit messages and PRs. Do not write "previously we did X, now we do Y" — just describe Y.

## One physical line per paragraph

Prose paragraphs must not span multiple physical lines (hard wraps). Each paragraph is one long line. Code blocks and tables are exempt. The `verify-md-wrap` gate enforces this.

## Slop checklist

Avoid:
- **Duplicated rules**: the same rule in multiple places
- **Narrative history**: "we used to do X, then we switched to Y"
- **Implementation status comments**: "TODO", "FIXME", "not yet implemented"
- **Hand-written tables of contents**: let the renderer generate them
- **Reasoning trails**: explaining why you decided something in the doc (put it in an Agent Note)
- **Paragraph walls**: unbroken walls of text — use structure (headings, lists)
- **Emphasis inflation**: excessive bold/italic

## Bilingual pairing

In-scope documents (standing docs, Agent Notes, README) must have:
- `foo.md` (English)
- `foo.zh.md` (Chinese)
- `foo.i18n.yaml` (consistency record with git blob hashes)

**Exceptions** (not in scope): `docs/plans/`, `docs/analysis/`, `book/src/`, `CLAUDE.md`, `AGENTS.md`, `docs/AGENTS.md`, `SKILL.md` files.

See [docs/i18n/README.md](i18n/README.md) for the pairing contract.

## `docs/plans/` management

- New design documents go in `docs/plans/` root with `yyyy-mm-dd-topic.md` naming
- Status field: **方案（未实施）** / **实施中** / **已落地** / **已废弃**
- Monthly on the 1st, last month's documents are moved into `docs/plans/yyyy-mm/` folders (`npm run archive-plans`)
- Yearly on January 1st, the previous year's month folders are wrapped into `docs/plans/yyyy/`
- Archived documents are frozen; only status annotations may be updated
- **严禁修改或引用已归档的 plans 子文件夹（`docs/plans/yyyy-mm/`）里的文件**

## Non-`docs/plans/` documents must not reference `docs/plans/`

`docs/plans/` documents are provisional: they record an in-session proposal, get archived monthly, and their status flips from 方案（未实施） to 已落地/已废弃. No document outside `docs/plans/` may reference or link to a `docs/plans/` document — a plan must never become a citation target. Every non-`docs/plans/` document is self-contained: standing facts live in standing docs, decision records in Agent Notes, and technical analysis in `docs/analysis/`. Mentioning the `docs/plans/` tier itself in governing text (this file, `CLAUDE.md`, `AGENTS.md`) is governance prose, not a reference.

## Checks

Run `npm run doc-sync` before submitting any documentation change. It chains the per-document gates (links, wraps, pairing, Agent Note format, Mermaid, doc references, plans archiving state). `./scripts/check-doc-paths.sh` separately verifies every Rust source path named in `CLAUDE.md` and the skills still resolves.
