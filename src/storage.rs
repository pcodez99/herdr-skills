use crate::Result;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};
use std::{path::Path, time::Duration};

const APPLICATION_ID: i64 = 0x4852_4452; // "HRDR"

/// Schema migrations; `PRAGMA user_version` counts the ones applied.
const MIGRATIONS: &[&str] = &["
CREATE TABLE sessions(
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL CHECK(trim(title) <> ''),
  created TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
) STRICT;

-- Interview checkpoints: each one supersedes the previous for its session.
CREATE TABLE notes(
  id INTEGER PRIMARY KEY,
  session INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  body TEXT NOT NULL CHECK(trim(body) <> ''),
  created TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
) STRICT;
CREATE INDEX notes_by_session ON notes(session, id);

CREATE TABLE plans(
  id INTEGER PRIMARY KEY,
  session INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  body TEXT NOT NULL CHECK(trim(body) <> ''),
  created TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  approved TEXT,
  approver TEXT CHECK((approved IS NULL) = (approver IS NULL)),
  verified TEXT CHECK(verified IS NULL OR approved IS NOT NULL)
) STRICT;
CREATE INDEX plans_by_session ON plans(session, id);
CREATE TRIGGER plans_text_is_immutable BEFORE UPDATE OF id, session, body, created ON plans
BEGIN SELECT RAISE(ABORT, 'A plan text never changes'); END;

CREATE TABLE checks(
  id INTEGER PRIMARY KEY,
  plan INTEGER NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
  criterion TEXT NOT NULL,
  result TEXT NOT NULL CHECK(result IN ('pass','fail')),
  evidence TEXT NOT NULL CHECK(trim(evidence) <> ''),
  reviewer TEXT NOT NULL,
  -- Working-tree hash; required for wiki checks, which hold only for that exact content.
  snapshot TEXT CHECK(criterion <> 'wiki' OR snapshot IS NOT NULL),
  created TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
) STRICT;
CREATE INDEX checks_by_plan ON checks(plan, criterion, id);
CREATE TRIGGER checks_are_append_only BEFORE UPDATE ON checks
BEGIN SELECT RAISE(ABORT, 'Checks are append-only'); END;

-- draft: newest, not approved. approved: the session's active plan.
CREATE VIEW plan_status(id, session, status) AS
SELECT p.id, p.session, CASE
  WHEN p.verified IS NOT NULL THEN 'verified'
  WHEN p.approved IS NOT NULL
   AND p.id = (SELECT max(id) FROM plans q WHERE q.session = p.session AND q.approved IS NOT NULL)
    THEN 'approved'
  WHEN p.id = (SELECT max(id) FROM plans q WHERE q.session = p.session) THEN 'draft'
  ELSE 'superseded' END
FROM plans p;
"];

pub struct Plan {
    pub session: i64,
    pub body: String,
    pub approver: Option<String>,
    pub status: String,
}

pub fn open(path: &Path) -> Result<Connection> {
    let db = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )?;
    db.busy_timeout(Duration::from_secs(10))?;
    db.pragma_update(None, "journal_mode", "WAL")?;
    db.pragma_update(None, "synchronous", "NORMAL")?;
    db.pragma_update(None, "foreign_keys", true)?;
    Ok(db)
}

/// Takes the write lock before reading, then creates or migrates the schema.
pub fn begin(db: &mut Connection) -> Result<Transaction<'_>> {
    let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = tx.query_one("PRAGMA user_version", [], |r| r.get(0))?;
    let version = usize::try_from(version).unwrap_or(usize::MAX);
    let app: i64 = tx.query_one("PRAGMA application_id", [], |r| r.get(0))?;
    let tables: i64 = tx.query_one("SELECT count(*) FROM sqlite_schema", [], |r| r.get(0))?;
    let ours = app == APPLICATION_ID && (1..=MIGRATIONS.len()).contains(&version);
    if !ours && (version, app, tables) != (0, 0, 0) {
        return Err("Unknown plan database schema".into());
    }
    if version < MIGRATIONS.len() {
        for migration in &MIGRATIONS[version..] {
            tx.execute_batch(migration)?;
        }
        tx.pragma_update(None, "user_version", MIGRATIONS.len() as i64)?;
        tx.pragma_update(None, "application_id", APPLICATION_ID)?;
    }
    Ok(tx)
}

pub fn start(tx: &Transaction, title: &str) -> Result<i64> {
    tx.execute("INSERT INTO sessions(title) VALUES (?)", [title])?;
    Ok(tx.last_insert_rowid())
}

/// One line per session with its latest plan and status; never any text body.
pub fn list(tx: &Transaction) -> Result<String> {
    let lines = tx
        .prepare(
            "SELECT json_object('session', s.id, 'title', s.title, 'plan', ps.id,
                                'status', coalesce(ps.status, 'interview'))
             FROM sessions s
             LEFT JOIN plan_status ps
               ON ps.id = (SELECT max(id) FROM plans WHERE session = s.id)
             ORDER BY s.id DESC",
        )?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(lines.into_iter().map(|l| l + "\n").collect())
}

/// What an interviewer needs to continue: the latest checkpoint and plan, nothing older.
pub fn resume(tx: &Transaction, session: i64) -> Result<String> {
    Ok(tx
        .query_one(
            "SELECT json_object('session', s.id, 'title', s.title,
               'checkpoint', (SELECT body FROM notes WHERE session = s.id ORDER BY id DESC LIMIT 1),
               'plan', ps.id, 'status', coalesce(ps.status, 'interview'))
             FROM sessions s
             LEFT JOIN plan_status ps
               ON ps.id = (SELECT max(id) FROM plans WHERE session = s.id)
             WHERE s.id = ?",
            [session],
            |r| r.get(0),
        )
        .optional()?
        .ok_or("Unknown session in this repository")?)
}

/// The latest result of each criterion of one plan, without evidence text.
pub fn status(tx: &Transaction, plan: i64, criteria: &[&str], snapshot: &str) -> Result<String> {
    // Criterion IDs are validated `R<n>` or fixed names, so plain quoting is safe.
    let criteria = format!("[\"{}\"]", criteria.join("\",\""));
    Ok(tx.query_one(
        "SELECT json_object('plan', ps.id, 'session', ps.session, 'status', ps.status,
           'criteria', json((SELECT json_group_object(c.value,
              (SELECT result FROM checks k WHERE k.plan = ps.id AND k.criterion = c.value
               ORDER BY k.id DESC LIMIT 1)) FROM json_each(?2) c)),
           'wiki', json_object('snapshot', ?3,
              'passed_by', (SELECT count(DISTINCT reviewer) FROM checks k WHERE k.plan = ps.id
                 AND k.criterion = 'wiki' AND k.snapshot = ?3 AND k.result = 'pass'),
              'failed', (SELECT count(*) FROM checks k WHERE k.plan = ps.id
                 AND k.criterion = 'wiki' AND k.snapshot = ?3 AND k.result = 'fail')))
         FROM plan_status ps WHERE ps.id = ?1",
        (plan, criteria, snapshot),
        |r| r.get(0),
    )?)
}

pub fn require_session(tx: &Transaction, session: i64) -> Result<()> {
    let found: bool = tx.query_one(
        "SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?)",
        [session],
        |r| r.get(0),
    )?;
    if found {
        Ok(())
    } else {
        Err("Unknown session in this repository".into())
    }
}

pub fn note(tx: &Transaction, session: i64, body: &str) -> Result<()> {
    tx.execute(
        "INSERT INTO notes(session, body) VALUES (?, ?)",
        (session, body),
    )?;
    Ok(())
}

pub fn propose(tx: &Transaction, session: i64, body: &str) -> Result<i64> {
    tx.execute(
        "INSERT INTO plans(session, body) VALUES (?, ?)",
        (session, body),
    )?;
    Ok(tx.last_insert_rowid())
}

pub fn plan(tx: &Transaction, id: i64) -> Result<Plan> {
    Ok(tx
        .query_one(
            "SELECT p.session, p.body, p.approver, ps.status
             FROM plans p JOIN plan_status ps ON ps.id = p.id WHERE p.id = ?",
            [id],
            |r| {
                Ok(Plan {
                    session: r.get(0)?,
                    body: r.get(1)?,
                    approver: r.get(2)?,
                    status: r.get(3)?,
                })
            },
        )
        .optional()?
        .ok_or("Unknown plan in this repository")?)
}

pub fn latest_plan(tx: &Transaction, session: i64) -> Result<i64> {
    Ok(tx.query_one(
        "SELECT max(id) FROM plans WHERE session = ?",
        [session],
        |r| r.get(0),
    )?)
}

pub fn approve(tx: &Transaction, plan: i64, approver: &str) -> Result<()> {
    tx.execute(
        "UPDATE plans SET approved = strftime('%Y-%m-%dT%H:%M:%SZ','now'), approver = ? WHERE id = ?",
        (approver, plan),
    )?;
    Ok(())
}

pub fn verify(tx: &Transaction, plan: i64) -> Result<()> {
    tx.execute(
        "UPDATE plans SET verified = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?",
        [plan],
    )?;
    Ok(())
}

pub struct Check<'a> {
    pub plan: i64,
    pub criterion: &'a str,
    pub result: &'a str,
    pub evidence: &'a str,
    pub reviewer: &'a str,
    pub snapshot: Option<&'a str>,
}

pub fn check(tx: &Transaction, c: &Check) -> Result<()> {
    tx.execute(
        "INSERT INTO checks(plan, criterion, result, evidence, reviewer, snapshot)
         VALUES (?, ?, ?, ?, ?, ?)",
        (
            c.plan,
            c.criterion,
            c.result,
            c.evidence,
            c.reviewer,
            c.snapshot,
        ),
    )?;
    Ok(())
}

/// Wiki review results on one snapshot: (distinct passing reviewers, failures).
pub fn wiki_reviews(tx: &Transaction, plan: i64, snapshot: &str) -> Result<(i64, i64)> {
    Ok(tx.query_one(
        "SELECT count(DISTINCT CASE WHEN result = 'pass' THEN reviewer END),
                count(CASE WHEN result = 'fail' THEN 1 END)
         FROM checks WHERE plan = ? AND criterion = 'wiki' AND snapshot = ?",
        (plan, snapshot),
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?)
}

pub fn latest_result(tx: &Transaction, plan: i64, criterion: &str) -> Result<Option<String>> {
    Ok(tx
        .query_one(
            "SELECT result FROM checks WHERE plan = ? AND criterion = ? ORDER BY id DESC LIMIT 1",
            (plan, criterion),
            |r| r.get(0),
        )
        .optional()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_schema_and_takes_write_lock_before_reading() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().canonicalize()?.join("herdr.sqlite3");
        let mut db = open(&path)?;
        let mut other = open(&path)?;
        other.busy_timeout(Duration::ZERO)?;
        let tx = begin(&mut db)?;
        assert_eq!(
            other
                .execute_batch("BEGIN IMMEDIATE")
                .unwrap_err()
                .sqlite_error_code(),
            Some(rusqlite::ErrorCode::DatabaseBusy)
        );
        let session = start(&tx, "Task")?;
        let id = propose(&tx, session, "- [R1] Test\r\n")?;
        tx.commit()?;
        let tx = begin(&mut other)?;
        assert_eq!(plan(&tx, id)?.body, "- [R1] Test\r\n");
        assert!(note(&tx, 999, "orphan").is_err());
        Ok(())
    }

    #[test]
    fn plan_text_and_checks_cannot_be_rewritten() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let mut db = open(&dir.path().canonicalize()?.join("herdr.sqlite3"))?;
        let tx = begin(&mut db)?;
        let id = propose(&tx, start(&tx, "Task")?, "- [R1] Test")?;
        approve(&tx, id, "t1")?;
        let mut c = Check {
            plan: id,
            criterion: "R1",
            result: "pass",
            evidence: "ok",
            reviewer: "t1",
            snapshot: None,
        };
        check(&tx, &c)?;
        assert!(
            tx.execute("UPDATE plans SET body = 'other' WHERE id = ?", [id])
                .is_err()
        );
        assert!(tx.execute("UPDATE checks SET result = 'fail'", []).is_err());
        c.result = "maybe";
        assert!(check(&tx, &c).is_err());
        c.result = "pass";
        c.criterion = "wiki";
        assert!(check(&tx, &c).is_err(), "a wiki check needs its snapshot");
        Ok(())
    }

    #[test]
    fn rejects_unknown_schema_without_changing_database() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().canonicalize()?.join("unknown.sqlite3");
        let mut db = open(&path)?;
        db.execute_batch("CREATE TABLE events(id INTEGER); INSERT INTO events VALUES(42);")?;
        db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
        let original = std::fs::read(&path)?;
        assert_eq!(
            begin(&mut db).unwrap_err().to_string(),
            "Unknown plan database schema"
        );
        assert_eq!(std::fs::read(&path)?, original);
        Ok(())
    }
}
