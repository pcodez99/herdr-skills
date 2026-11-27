use crate::{
    Result,
    domain::{self, Command, INTEGRATION, WIKI, WIKI_REVIEWERS},
    repository,
    storage::{self, Plan},
};
use rusqlite::Transaction;
use std::env;

pub enum Outcome {
    Id(i64),
    Text(String),
    Silent,
}

pub fn execute(command: Command) -> Result<Outcome> {
    if let Command::Propose(_, body) = &command {
        domain::criteria(body)?;
        let top = repository::toplevel()?;
        for page in domain::sources(body)? {
            if !top.join(page).is_file() {
                return Err(format!("Wiki page not found: {page}").into());
            }
        }
    }
    let mut db = storage::open(&repository::database()?)?;
    let tx = storage::begin(&mut db)?;
    let outcome = match command {
        Command::Start(title) => {
            if title.trim().is_empty() {
                return Err("Expected a task title".into());
            }
            Outcome::Id(storage::start(&tx, &title)?)
        }
        Command::List => Outcome::Text(storage::list(&tx)?),
        Command::Resume(session) => Outcome::Text(storage::resume(&tx, session)? + "\n"),
        Command::Note(session, body) => {
            storage::require_session(&tx, session)?;
            storage::note(&tx, session, &body)?;
            Outcome::Silent
        }
        Command::Propose(session, body) => {
            storage::require_session(&tx, session)?;
            Outcome::Id(storage::propose(&tx, session, &body)?)
        }
        Command::Show(id) => Outcome::Text(storage::plan(&tx, id)?.body),
        Command::Status(id) => {
            let plan = storage::plan(&tx, id)?;
            let criteria = criteria(&plan)?;
            Outcome::Text(storage::status(&tx, id, &criteria, &repository::snapshot()?)? + "\n")
        }
        Command::Approve(id) => {
            let plan = storage::plan(&tx, id)?;
            if storage::latest_plan(&tx, plan.session)? != id {
                return Err("A newer plan exists; review it first".into());
            }
            if plan.approver.is_none() {
                storage::approve(&tx, id, &terminal()?)?;
            }
            Outcome::Silent
        }
        Command::Check {
            plan: id,
            criterion,
            result,
            evidence,
        } => {
            let plan = active(&tx, id)?;
            let reviewer = terminal()?;
            let snapshot = if criterion == WIKI {
                // Independence is structural: the approving master cannot review.
                if plan.approver.as_deref() == Some(reviewer.as_str()) {
                    return Err(
                        "The wiki review must come from an agent other than the master".into(),
                    );
                }
                Some(repository::snapshot()?)
            } else if criteria(&plan)?.contains(&criterion.as_str()) {
                None
            } else {
                return Err("Unknown criterion for this plan".into());
            };
            storage::check(
                &tx,
                &storage::Check {
                    plan: id,
                    criterion: &criterion,
                    result: &result,
                    evidence: &evidence,
                    reviewer: &reviewer,
                    snapshot: snapshot.as_deref(),
                },
            )?;
            Outcome::Silent
        }
        Command::Finish(id) => {
            let plan = active(&tx, id)?;
            let mut missing = Vec::new();
            for criterion in criteria(&plan)? {
                if storage::latest_result(&tx, id, criterion)?.as_deref() != Some("pass") {
                    missing.push(criterion.to_owned());
                }
            }
            let (passed, failed) = storage::wiki_reviews(&tx, id, &repository::snapshot()?)?;
            if failed > 0 || passed < WIKI_REVIEWERS {
                missing.push(format!(
                    "{WIKI} ({passed}/{WIKI_REVIEWERS} independent passes, {failed} failures on the current tree)"
                ));
            }
            if !missing.is_empty() {
                return Err(format!("Missing passing evidence: {}", missing.join(", ")).into());
            }
            storage::verify(&tx, id)?;
            Outcome::Silent
        }
    };
    tx.commit()?;
    Ok(outcome)
}

/// The plan's own criteria plus the integration check every plan needs.
fn criteria(plan: &Plan) -> Result<Vec<&str>> {
    Ok(domain::criteria(&plan.body)?
        .into_iter()
        .chain([INTEGRATION])
        .collect())
}

/// The Herdr terminal running this command identifies who approves or reviews.
fn terminal() -> Result<String> {
    env::var("HC_TERMINAL_ID")
        .ok()
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| "Run inside a Herdr terminal: HC_TERMINAL_ID identifies the agent".into())
}

/// Evidence is recorded only against the session's approved, still open plan.
fn active(tx: &Transaction, id: i64) -> Result<Plan> {
    let plan = storage::plan(tx, id)?;
    match plan.status.as_str() {
        "approved" => Ok(plan),
        "verified" => Err("Plan already verified".into()),
        _ => Err("Plan is not the active approved plan".into()),
    }
}
