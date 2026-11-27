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

pub const INTEGRATION: &str = "integration";
pub const WIKI: &str = "wiki";
pub const WIKI_REVIEWERS: i64 = 2;
pub const WIKI_ROOT: &str = "knowledge/";

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

fn criterion(line: &str) -> Option<(&str, &str)> {
    let (id, text) = line.strip_prefix("- [")?.split_once("] ")?;
    let number = id.strip_prefix('R')?;
    let valid = !number.is_empty()
        && !number.starts_with('0')
        && number.bytes().all(|b| b.is_ascii_digit())
        && text.starts_with(|c: char| !c.is_whitespace());
    valid.then_some((id, text))
}

pub fn criteria(body: &str) -> Result<BTreeSet<&str>> {
    const ERROR: &str =
        "Use unique acceptance criteria such as '- [R1] Expected result. Verify: how to check it'";
    let mut ids = BTreeSet::new();
    for (id, _) in body.split('\n').filter_map(criterion) {
        if !ids.insert(id) {
            return Err(ERROR.into());
        }
    }
    if ids.is_empty() {
        return Err(ERROR.into());
    }
    Ok(ids)
}

fn has_marker(line: &str) -> bool {
    line.split('[').skip(1).any(|rest| {
        let rest = rest.trim_start();
        let wiki = rest
            .get(..4)
            .is_some_and(|word| word.eq_ignore_ascii_case("wiki"))
            && rest[4..].trim_start().starts_with(']');
        let criterion = rest.strip_prefix(['R', 'r']).is_some_and(|tail| {
            tail.trim_start()
                .starts_with(|c: char| c.is_ascii_digit() || c == ']')
        });
        wiki || criterion
    })
}

pub fn require_verifiable(body: &str) -> Result<()> {
    for line in body.split('\n') {
        if has_marker(line) && criterion(line).is_none() && !line.starts_with("- [wiki] ") {
            return Err(format!(
                "Malformed plan line '{}': criteria are '- [R1] Outcome. Verify: command or steps' and sources '- [wiki] knowledge/page.md', each at the start of its line; mention criteria elsewhere without brackets",
                line.trim()
            )
            .into());
        }
    }
    for (id, text) in body.split('\n').filter_map(criterion) {
        let stated = text
            .split_once("Verify:")
            .is_some_and(|(outcome, how)| !outcome.trim().is_empty() && !how.trim().is_empty());
        if !stated {
            return Err(format!(
                "{id}: state how to check it on the running product, as '- [{id}] Outcome. Verify: command or steps'"
            )
            .into());
        }
    }
    Ok(())
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
    fn every_criterion_states_its_verification() {
        assert!(require_verifiable("- [R1] Works. Verify: make check").is_ok());
        assert!(require_verifiable("- [R1] Works").is_err());
        assert!(require_verifiable("- [R1] Works. Verify:  ").is_err());
        assert!(require_verifiable("- [R1] Verify: make check").is_err());
        for malformed in [
            "- [R0] x",
            "- [R01] x",
            "- [R2]  x",
            "- [r3] x",
            "  - [R4] x",
            "- [R5]x",
            "* [R6] x",
            "- [R] x",
            "- [ R7] x",
            "[R8] x",
            "1. [R9] x",
            "> - [R10] x",
            "* [wiki] knowledge/x.md",
            "  - [wiki] knowledge/x.md",
            "- [WIKI] knowledge/x.md",
            "See [R2] first",
        ] {
            let body = format!("- [R1] Works. Verify: run\n{malformed}. Verify: run");
            assert!(require_verifiable(&body).is_err(), "{malformed}");
        }
        assert!(require_verifiable("- [R1] Works. Verify: run\n- [README](README.md)").is_ok());
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
