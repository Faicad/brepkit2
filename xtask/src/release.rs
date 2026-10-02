//! Automatic semver bumping from conventional-commit git history.
//!
//! Determines the next npm package version by walking every commit since the
//! last commit that touched `version = "..."` in `crates/wasm/Cargo.toml`,
//! classifying them with Conventional Commits rules, and applying the highest
//! warranted bump:
//!
//! - breaking (`!` suffix or `BREAKING CHANGE` footer) → major
//! - `feat` → minor
//! - everything else that is not `chore`/`docs`/`test`/`ci`/`style` → patch
//!
//! `chore(deps)` and similar maintenance-only commits do not trigger a bump on
//! their own; they still count as "changes exist" so an explicit release is
//! allowed to ship them as a patch.
//!
//! Since the fork starts at upstream's version (`2.129.15`), major bumps are
//! clamped to minor by default (upstream owns that version line's semantics);
//! pass `--allow-major` to release a true major. Pass `--exact` to use the
//! computed version verbatim with no clamping.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::wasm::{build_both_targets, check_tools, merge_packages, run_wasm_opt,
    validate_output, RunReleaseOpts, WASM_MANIFEST_PATH};

/// Kinds that never bump the version on their own. They are still shippable
/// (as patch) alongside other changes.
const NON_BUMPING_KINDS: [&str; 6] = ["chore", "docs", "test", "ci", "style", "build"];

/// One classified conventional commit from the log.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ClassifiedCommit {
    pub(crate) hash: String,
    pub(crate) kind: String,
    /// True when the subject has `!` or the body has a `BREAKING CHANGE` footer.
    pub(crate) breaking: bool,
}

/// Changelog section a commit kind maps to, in Keep a Changelog order.
fn changelog_section(kind: &str) -> Option<&'static str> {
    match kind {
        "feat" => Some("Added"),
        "fix" | "perf" => Some("Fixed"),
        "refactor" | "revert" => Some("Changed"),
        "docs" => Some("Documentation"),
        _ => None,
    }
}

/// Generate the `## [version] - date` block (Keep a Changelog format) from the
/// classified commits, grouped into Added / Changed / Documentation / Fixed.
pub(crate) fn generate_changelog_block(version: &str, commits: &[ClassifiedCommit]) -> String {
    let date = {
        // Local date via `git log`-independent clock; chrono is not a
        // dependency, so use the system date through an env-free call.
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // Civil-from-days algorithm (Howard Hinnant) for UTC date.
        let days = (secs / 86_400) as i64;
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        format!("{y:04}-{m:02}-{d:02}")
    };

    let mut sections: Vec<(&str, Vec<String>)> =
        vec![("Added", vec![]), ("Changed", vec![]), ("Documentation", vec![]), ("Fixed", vec![])];
    for c in commits {
        // Commit subject text is everything after the conventional prefix.
        let subject = c.kind.clone();
        let desc = subject
            .split_once(": ")
            .map(|(_, rest)| rest.to_string())
            .unwrap_or(subject.clone());
        let prefix = if c.breaking { "**BREAKING** " } else { "" };
        let line = format!("- {prefix}{desc} ({})", &c.hash[..7.min(c.hash.len())]);
        if let Some(section) = changelog_section(c.kind.split('(').next().unwrap_or(&c.kind)) {
            if let Some(slot) = sections.iter_mut().find(|(name, _)| *name == section) {
                slot.1.push(line);
            }
        }
        // build/chore/test/ci/style commits are intentionally omitted: they do
        // not affect package consumers.
    }

    let mut out = format!("## [{version}] - {date}\n");
    for (name, lines) in sections {
        if lines.is_empty() {
            continue;
        }
        out.push_str(&format!("\n### {name}\n"));
        for line in lines {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out
}

/// Insert the new version block under `## [Unreleased]` (Keep a Changelog
/// convention): unreleased content merges into the new version block.
pub(crate) fn update_changelog(path: &std::path::Path, block: &str) -> Result<()> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;

    let anchor = "## [Unreleased]";
    let pos = text
        .find(anchor)
        .context("CHANGELOG.md has no `## [Unreleased]` heading")?;
    let after = pos + anchor.len();

    // Content between the anchor and the next `## ` heading (or EOF) is the
    // unreleased backlog; it is merged into the new version block's section.
    let rest = &text[after..];
    let next_heading = rest.find("\n## ").map(|i| i + 1).unwrap_or(rest.len());
    let backlog = rest[..next_heading].trim().to_string();

    let mut new_text = String::with_capacity(text.len() + block.len() + 16);
    new_text.push_str(&text[..after]);
    new_text.push('\n');
    if !backlog.is_empty() {
        new_text.push_str(&backlog);
        new_text.push('\n');
    }
    new_text.push('\n');
    new_text.push_str(block);
    new_text.push_str(&rest[next_heading..]);

    std::fs::write(path, new_text)?;
    Ok(())
}

/// The version bump warranted by a set of commits.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Bump {
    Major,
    Minor,
    Patch,
}

impl Bump {
    fn max(self, other: Bump) -> Bump {
        match (self, other) {
            (Bump::Major, _) | (_, Bump::Major) => Bump::Major,
            (Bump::Minor, _) | (_, Bump::Minor) => Bump::Minor,
            (Bump::Patch, Bump::Patch) => Bump::Patch,
        }
    }
}

/// Parse a conventional-commit subject line: `type(scope)!: subject`.
/// Returns `(kind, breaking)` or `None` when the subject is not conventional.
pub(crate) fn parse_conventional_subject(subject: &str) -> Option<(String, bool)> {
    let subject = subject.trim();
    // Kinds may contain a nested scope word like `deps-dev`; letters + `-`.
    let idx = subject.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))?;
    let kind = &subject[..idx];
    if kind.is_empty() || !subject[idx..].starts_with('(') {
        // No scope: kind must be immediately followed by `:` or `!`.
        if subject[idx..].starts_with(':') {
            return Some((kind.to_string(), false));
        }
        if subject[idx..].starts_with("!:") {
            return Some((kind.to_string(), true));
        }
        return None;
    }
    let after_scope = &subject[idx + 1..];
    let close = after_scope.find(')')?;
    let rest = &after_scope[close + 1..];
    let breaking = rest.starts_with("!:") || rest.starts_with('!');
    Some((kind.to_string(), breaking))
}

/// Read one raw commit (subject + full body) from `git log`.
///
/// Uses `--format=%H%x1f%B%x1e`: each record is `hash<US>body<RS>` so commit
/// bodies containing newlines stay intact (ASCII unit/record separators never
/// appear in commit text, unlike a plain newline split).
pub(crate) fn read_commit_range(from_commit: Option<&str>) -> Result<Vec<(String, String)>> {
    const US: char = '\u{1f}';
    const RS: char = '\u{1e}';

    let mut cmd = Command::new("git");
    cmd.args(["log", "--format=%H%x1f%B%x1e"]);
    if let Some(from) = from_commit {
        cmd.arg(format!("{from}..HEAD"));
    }
    let output = String::from_utf8_lossy(&cmd.output()?.stdout).to_string();
    Ok(output
        .split(RS)
        .filter_map(|record| {
            let record = record.trim_start_matches(['\n', '\r']);
            if record.is_empty() {
                return None;
            }
            let (hash, body) = record.split_once(US)?;
            Some((hash.to_string(), body.trim_end().to_string()))
        })
        .collect())
}

/// Find the commit hash of the last change to the wasm crate version line.
pub(crate) fn last_version_commit() -> Result<Option<String>> {
    let output = String::from_utf8_lossy(
        &Command::new("git")
            .args([
                "log",
                "--follow",
                "--format=%H",
                "--",
                WASM_MANIFEST_PATH,
            ])
            .output()?
            .stdout,
    )
    .to_string();

    // Walk commits touching the manifest newest-first; the newest one whose
    // diff actually changed the `version` key is our baseline.
    for hash in output.lines().filter(|l| !l.trim().is_empty()) {
        let diff = String::from_utf8_lossy(
            &Command::new("git")
                .args(["show", hash, "--format=", "--", WASM_MANIFEST_PATH])
                .output()?
                .stdout,
        )
        .to_string();
        if diff
            .lines()
            .any(|l| (l.starts_with("+version") || l.starts_with("-version")) && l.contains('='))
        {
            return Ok(Some(hash.to_string()));
        }
    }
    Ok(None)
}

/// Classify a commit's message into a bump contribution.
pub(crate) fn classify_commit(subject: &str, body: &str) -> Option<Bump> {
    let (kind, mut breaking) = parse_conventional_subject(subject)?;
    breaking = breaking || body.contains("BREAKING CHANGE");
    if breaking {
        return Some(Bump::Major);
    }
    if NON_BUMPING_KINDS.contains(&kind.as_str()) {
        return None;
    }
    match kind.as_str() {
        "feat" => Some(Bump::Minor),
        // fix, perf, refactor, revert, ... — anything user-facing gets patch.
        _ => Some(Bump::Patch),
    }
}

/// Compute the warranted bump from all commits since the last version change.
pub(crate) fn compute_bump() -> Result<(Bump, Vec<ClassifiedCommit>)> {
    let base = last_version_commit()?;
    let raw = read_commit_range(base.as_deref())?;
    let mut commits = Vec::new();
    let mut bump = Bump::Patch;
    let mut bumping = false;

    for (hash, message) in raw {
        let (subject, body) = match message.split_once('\n') {
            Some((s, b)) => (s.to_string(), b.to_string()),
            None => (message.clone(), String::new()),
        };
        let commit_bump = classify_commit(&subject, &body);
        if let Some(b) = commit_bump {
            bump = bump.max(b);
            bumping = true;
        }
        commits.push(ClassifiedCommit {
            hash,
            kind: subject.clone(),
            breaking: commit_bump == Some(Bump::Major),
        });
    }

    if !bumping {
        bail!(
            "no version-worthy commits found since the last version change \
             (chore/docs/test/ci commits only). Nothing to release."
        );
    }
    Ok((bump, commits))
}

/// Bump an explicit semver triple.
pub(crate) fn bump_version(version: &str, bump: Bump) -> Result<String> {
    let mut parts: Vec<u64> = version
        .split('.')
        .map(|p| p.parse().context("non-numeric version component"))
        .collect::<Result<_>>()?;
    if parts.len() != 3 {
        bail!("version {version:?} is not a plain X.Y.Z triple");
    }
    match bump {
        Bump::Major => {
            parts[0] += 1;
            parts[1] = 0;
            parts[2] = 0;
        }
        Bump::Minor => {
            parts[1] += 1;
            parts[2] = 0;
        }
        Bump::Patch => parts[2] += 1,
    }
    Ok(parts.iter().map(|p| p.to_string()).collect::<Vec<_>>().join("."))
}

/// Rewrite `version = "..."` in the wasm crate manifest.
pub(crate) fn write_version(path: &std::path::Path, new_version: &str) -> Result<()> {
    let text = std::fs::read_to_string(path)?;
    let mut replaced = false;
    let out: String = text
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("version =") && !replaced {
                replaced = true;
                format!("version = \"{new_version}\"")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if !replaced {
        bail!("no `version = ...` line found in {}", path.display());
    }
    // Preserve the trailing newline the original file has.
    std::fs::write(path, if text.ends_with('\n') { format!("{out}\n") } else { out })?;
    Ok(())
}

/// Full release orchestration: compute version → bump manifest + CHANGELOG →
/// build → validate. Publishing (smoke → npm publish → registry verify) stays
/// in `scripts/publish-wasm.ps1`, which owns the dist-tag, OTP and record.
pub(crate) fn run_release(opts: RunReleaseOpts) -> Result<()> {
    let manifest_path: PathBuf = project_root()?.join(WASM_MANIFEST_PATH);
    let current = read_crate_version(&manifest_path)?;

    let (bump, commits) = compute_bump()?;
    println!("\nVersion basis: {current}");
    println!("Commits since last version change:");
    for c in &commits {
        println!("  {} {}", &c.hash[..7.min(c.hash.len())], c.kind);
    }

    let next = if let Some(explicit) = &opts.version {
        explicit.clone()
    } else {
        let effective = if bump == Bump::Major && !opts.allow_major {
            println!("  breaking change detected; clamping major -> minor (use --allow-major to override)");
            Bump::Minor
        } else {
            bump
        };
        bump_version(&current, effective)?
    };

    if next == current {
        bail!("computed version equals current version ({current}); nothing to release");
    }

    println!("\nBumping version: {current} -> {next}");
    if opts.dry_run {
        println!("(dry-run: manifest and CHANGELOG not modified; continuing with build validation)");
    } else {
        write_version(&manifest_path, &next)?;
        println!("  updated {}", manifest_path.display());

        let changelog = project_root()?.join("CHANGELOG.md");
        let block = generate_changelog_block(&next, &commits);
        update_changelog(&changelog, &block)?;
        println!("  updated {}", changelog.display());
    }

    // Build + validate only. Smoke test and npm publish remain in
    // scripts/publish-wasm.ps1, which owns the dist-tag, OTP and record.
    check_tools()?;
    build_both_targets(opts.simd)?;
    run_wasm_opt()?;
    merge_packages()?;
    validate_output()?;

    println!("\n✅ Version bump + build complete: {next}");
    if !opts.dry_run {
        println!("   Finish the release with scripts/publish-wasm.ps1 (or rerun with -AutoVersion).");
    }
    Ok(())
}

fn project_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .context("xtask must live inside the project root")
        .map(|p| p.to_path_buf())
}

fn read_crate_version(path: &std::path::Path) -> Result<String> {
    let text = std::fs::read_to_string(path)?;
    text.lines()
        .filter_map(|line| {
            let value = line.split('#').next()?.trim();
            value
                .strip_prefix("version =")?
                .trim()
                .trim_matches('"')
                .to_string()
                .into()
        })
        .next()
        .context(format!("no version line in {}", path.display()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn parses_subject_with_scope() {
        assert_eq!(
            parse_conventional_subject("fix(blend): trim contact line"),
            Some(("fix".into(), false))
        );
        assert_eq!(
            parse_conventional_subject("feat(boolean)!: new API"),
            Some(("feat".into(), true))
        );
        assert_eq!(
            parse_conventional_subject("chore(deps-dev): bump x"),
            Some(("chore".into(), false))
        );
    }

    #[test]
    fn parses_subject_without_scope() {
        assert_eq!(
            parse_conventional_subject("fix: something"),
            Some(("fix".into(), false))
        );
        assert_eq!(
            parse_conventional_subject("feat!: big change"),
            Some(("feat".into(), true))
        );
        assert_eq!(parse_conventional_subject("not conventional"), None);
        assert_eq!(parse_conventional_subject("Merge branch x"), None);
    }

    #[test]
    fn classifies_bumps() {
        assert_eq!(
            classify_commit("feat(x): a", ""),
            Some(Bump::Minor)
        );
        assert_eq!(classify_commit("fix: a", ""), Some(Bump::Patch));
        assert_eq!(classify_commit("perf: a", ""), Some(Bump::Patch));
        assert_eq!(classify_commit("docs: a", ""), None);
        assert_eq!(classify_commit("chore(deps): a", ""), None);
        assert_eq!(classify_commit("fix: a", "BREAKING CHANGE: x"), Some(Bump::Major));
    }

    #[test]
    fn bumps_versions() {
        assert_eq!(bump_version("2.129.15", Bump::Patch).unwrap(), "2.129.16");
        assert_eq!(bump_version("2.129.15", Bump::Minor).unwrap(), "2.130.0");
        assert_eq!(bump_version("2.129.15", Bump::Major).unwrap(), "3.0.0");
        assert!(bump_version("2.129", Bump::Patch).is_err());
    }

    #[test]
    fn bump_takes_max() {
        assert_eq!(Bump::Patch.max(Bump::Minor), Bump::Minor);
        assert_eq!(Bump::Minor.max(Bump::Major), Bump::Major);
        assert_eq!(Bump::Patch.max(Bump::Patch), Bump::Patch);
    }
}
