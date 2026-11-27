use crate::Result;
use std::collections::BTreeSet;

pub enum Command {
    Start(String),
    List,
    Resume(i64),
    Note(i64, String),
    Propose(i64, String),
    Show(i64),
    Status(i64),
    Approve(i64),
    Check {
        plan: i64,
        criterion: String,
        result: String,
        evidence: String,
    },
    Finish(i64),
}

/// Checks every plan needs besides its own criteria: the combined result, and
/// agreement between implementation and wiki confirmed by independent reviewers.
pub const INTEGRATION: &str = "integration";
pub const WIKI: &str = "wiki";
/// Distinct reviewer terminals, other than the approving master's, that must pass
/// the wiki check on the same working-tree snapshot.
pub const WIKI_REVIEWERS: i64 = 2;
/// Every wiki page a plan is built on lives under this folder.
pub const WIKI_ROOT: &str = "knowledge/";

/// Wiki pages a plan is built on, declared as `- [wiki] knowledge/wiki/page.md`.
pub fn sources(body: &str) -> Result<Vec<&str>> {
    let sources: Vec<_> = body
        .lines()
        .filter_map(|line| line.strip_prefix("- [wiki] "))
        .map(str::trim)
        .collect();
    let valid = |path: &&str| {
        path.starts_with(WIKI_ROOT)
            && !path
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
    };
    if sources.is_empty() || !sources.iter().all(valid) {
        return Err(format!(
            "Build the plan on the wiki: list its pages as '- [wiki] {WIKI_ROOT}wiki/page.md'"
        )
        .into());
    }
    Ok(sources)
}

/// Acceptance criteria declared as `- [R1] Expected result`, one per line.
pub fn criteria(body: &str) -> Result<BTreeSet<&str>> {
    const ERROR: &str = "Use unique acceptance criteria such as '- [R1] Expected result'";
    let mut ids = BTreeSet::new();
    for line in body.split('\n') {
        let Some((id, text)) = line.strip_prefix("- [").and_then(|s| s.split_once("] ")) else {
            continue;
        };
        let Some(number) = id.strip_prefix('R') else {
            continue;
        };
        if number.is_empty()
            || number.starts_with('0')
            || !number.bytes().all(|b| b.is_ascii_digit())
            || !text.starts_with(|c: char| !c.is_whitespace())
        {
            continue;
        }
        if !ids.insert(id) {
            return Err(ERROR.into());
        }
    }
    if ids.is_empty() {
        return Err(ERROR.into());
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn criteria_require_unique_positive_ids_and_nonblank_descriptions() {
        assert!(criteria("- [R1] First\n- [R1] Duplicate").is_err());
        assert!(criteria("- [R0] Zero\n- [R01] Padded\n- [R2]  ").is_err());
        assert_eq!(
            criteria("preamble\r\n- [R2] Test\r\n- [R10] Review").unwrap(),
            BTreeSet::from(["R2", "R10"])
        );
    }

    #[test]
    fn sources_stay_inside_the_wiki() {
        assert!(sources("- [R1] No sources").is_err());
        assert!(sources("- [wiki] knowledge/../secret.md").is_err());
        assert!(sources("- [wiki] src/main.rs").is_err());
        assert_eq!(
            sources("- [wiki] knowledge/index.md\r\n- [wiki] knowledge/wiki/db.md").unwrap(),
            ["knowledge/index.md", "knowledge/wiki/db.md"]
        );
    }
}
