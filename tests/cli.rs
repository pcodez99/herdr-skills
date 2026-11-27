use serde_json::Value;
use std::{
    fs,
    io::Write,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
};
use tempfile::TempDir;

const SPEC: &[u8] = b"# Revisione \xc3\xa8\r\n\r\n- [wiki] knowledge/index.md\r\n- [R1] A result. Verify: run it  \r\n- [R2] Another result. Verify: run it\n";
const MASTER: &str = "t-master";

macro_rules! plan {
    ($criteria:literal) => {
        concat!("- [wiki] knowledge/index.md\n", $criteria).as_bytes()
    };
}

fn plans_bin() -> PathBuf {
    std::env::var_os("PLANS_BIN")
        .map_or_else(|| env!("CARGO_BIN_EXE_herdr-plans").into(), PathBuf::from)
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(ok.success(), "git {args:?}");
}

struct Fixture {
    _temp: TempDir,
    base: PathBuf,
    repo: PathBuf,
    state: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let repo = base.join("répo");
        fs::create_dir(&repo).unwrap();
        git(&repo, &["init", "-q"]);
        fs::create_dir(repo.join("knowledge")).unwrap();
        fs::write(
            repo.join("knowledge/index.md"),
            "---\nokf_version: '0.2'\n---\n",
        )
        .unwrap();
        let state = base.join("state");
        Self {
            _temp: temp,
            base,
            repo,
            state,
        }
    }

    fn exec(&self, bin: &Path, cwd: &Path, state: &Path, args: &[&str], body: &[u8]) -> Output {
        self.exec_as(MASTER, bin, cwd, state, args, body)
    }

    fn exec_as(
        &self,
        terminal: &str,
        bin: &Path,
        cwd: &Path,
        state: &Path,
        args: &[&str],
        body: &[u8],
    ) -> Output {
        let mut command = Command::new(bin);
        if let Some(path) = std::env::var_os("PLANS_PATH") {
            command.env("PATH", path);
        }
        let mut child = command
            .args(args)
            .current_dir(cwd)
            .env("XDG_STATE_HOME", state)
            .env("HOME", self.base.join("home"))
            .env("HC_TERMINAL_ID", terminal)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(body).unwrap();
        child.wait_with_output().unwrap()
    }

    fn run(&self, args: &[&str], body: &[u8], code: i32) -> Vec<u8> {
        let out = self.exec(&plans_bin(), &self.repo, &self.state, args, body);
        assert_eq!(
            out.status.code(),
            Some(code),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        if code == 0 {
            assert_eq!(String::from_utf8_lossy(&out.stderr), "");
        } else {
            assert!(out.stdout.is_empty() && !out.stderr.is_empty());
        }
        out.stdout
    }

    fn ok(&self, args: &[&str]) -> String {
        String::from_utf8(self.run(args, b"", 0))
            .unwrap()
            .trim()
            .to_owned()
    }

    fn with(&self, args: &[&str], body: &[u8]) -> String {
        String::from_utf8(self.run(args, body, 0))
            .unwrap()
            .trim()
            .to_owned()
    }

    fn fails(&self, args: &[&str], body: &[u8]) {
        self.run(args, body, 1);
    }

    fn review(&self, terminal: &str, plan: &str, result: &str) -> Option<i32> {
        let args = ["check", plan, "wiki", result];
        let out = self.exec_as(
            terminal,
            &plans_bin(),
            &self.repo,
            &self.state,
            &args,
            b"compared",
        );
        out.status.code()
    }

    fn json(&self, args: &[&str]) -> Value {
        serde_json::from_str(&self.ok(args)).unwrap()
    }

    fn db(&self) -> rusqlite::Connection {
        let folder = fs::read_dir(self.state.join("herdr-team"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        rusqlite::Connection::open(folder.path().join("herdr.sqlite3")).unwrap()
    }
}

#[test]
fn full_plan_cycle_with_rework() {
    let f = Fixture::new();
    for args in [
        &[][..],
        &["start"],
        &["absent"],
        &["list", "extra"],
        &["show", "x"],
        &["history", "1"],
    ] {
        f.run(args, b"", 2);
    }
    f.run(&["check", "1", "R1", "maybe"], b"", 2);
    f.fails(&["start", "  "], b"");
    let s = f.ok(&["start", "Piano è"]);
    f.fails(&["propose", &s], b"- [R1] Not built on the wiki");
    f.fails(&["propose", &s], b"- [wiki] knowledge/missing.md\n- [R1] x");
    assert_eq!(f.json(&["list"])["status"], "interview");
    f.fails(&["note", &s], b"");
    f.fails(&["note", &s], b"\xff");
    f.fails(&["note", "999"], b"x");
    f.fails(&["propose", "999"], plan!("- [R1] x"));
    f.fails(&["resume", "999"], b"");
    f.with(&["note", &s], b"Checkpoint \xc3\xa8\r\n");
    f.fails(&["propose", &s], plan!("- [R1] x\n- [R1] y"));
    f.fails(&["propose", &s], plan!("- [R0] x"));
    f.fails(&["propose", &s], plan!("- [R1] No way to check it"));
    let first = f.with(&["propose", &s], plan!("- [R1] Earlier. Verify: run it"));
    let plan = f.with(&["propose", &s], SPEC);
    assert_eq!(f.run(&["show", &plan], b"", 0), SPEC);
    f.fails(&["show", "999"], b"");
    assert_eq!(f.json(&["status", &first])["status"], "superseded");
    f.fails(&["approve", &first], b"");
    f.fails(&["check", &plan, "R1", "pass"], b"not approved");
    f.ok(&["approve", &plan]);
    f.ok(&["approve", &plan]);
    f.fails(&["check", &plan, "R9", "pass"], b"evidence");
    f.fails(&["check", &plan, "R1", "pass"], b"");
    f.with(&["check", &plan, "R1", "fail"], b"first run");
    f.with(&["check", &plan, "R2", "pass"], b"ok");
    f.fails(&["finish", &plan], b"");
    f.with(&["check", &plan, "integration", "pass"], b"ok");
    let status = f.json(&["status", &plan]);
    assert_eq!(status["status"], "approved");
    assert_eq!(
        status["criteria"],
        serde_json::json!({"R1": "fail", "R2": "pass", "integration": "pass"})
    );
    f.fails(&["finish", &plan], b"");
    f.with(&["check", &plan, "R1", "pass"], b"fixed");
    f.fails(&["finish", &plan], b"");
    assert_eq!(
        f.review(MASTER, &plan, "pass"),
        Some(1),
        "the master cannot review"
    );
    assert_eq!(f.review("t-a", &plan, "pass"), Some(0));
    assert_eq!(f.review("t-a", &plan, "pass"), Some(0));
    f.fails(&["finish", &plan], b"");
    assert_eq!(f.review("t-b", &plan, "pass"), Some(0));
    f.ok(&["finish", &plan]);
    f.fails(&["finish", &plan], b"");
    f.fails(&["check", &plan, "integration", "pass"], b"closed");
    assert_eq!(f.json(&["status", &plan])["status"], "verified");
    let resume = f.json(&["resume", &s]);
    assert_eq!(resume["checkpoint"], "Checkpoint è\r\n");
    assert_eq!(resume["plan"].to_string(), plan);
    assert_eq!(resume["status"], "verified");
}

#[test]
fn outputs_never_mix_plans_or_replay_history() {
    let f = Fixture::new();
    let a = f.ok(&["start", "Task A"]);
    let b = f.ok(&["start", "Task B"]);
    f.with(&["note", &a], b"A round 1");
    f.with(&["note", &a], b"A round 2");
    f.with(&["note", &b], b"B secret");
    let plan_a = f.with(&["propose", &a], plan!("- [R1] Body of A. Verify: run it"));
    f.with(&["propose", &b], plan!("- [R1] Body of B. Verify: run it"));
    let resume = f.ok(&["resume", &a]);
    assert!(resume.contains("A round 2"));
    for leak in ["A round 1", "B secret", "Body of"] {
        assert!(!resume.contains(leak), "{leak} leaked into resume");
    }
    let list = f.ok(&["list"]);
    assert_eq!(list.lines().count(), 2);
    for leak in ["round", "secret", "Body of"] {
        assert!(!list.contains(leak), "{leak} leaked into list");
    }
    let show = f.ok(&["show", &plan_a]);
    assert_eq!(show.as_bytes(), plan!("- [R1] Body of A. Verify: run it"));
    f.ok(&["approve", &plan_a]);
    f.with(&["check", &plan_a, "R1", "pass"], b"Evidence text");
    assert!(!f.ok(&["status", &plan_a]).contains("Evidence text"));
}

#[test]
fn a_newer_approved_plan_replaces_the_active_one() {
    let f = Fixture::new();
    let s = f.ok(&["start", "Rework"]);
    let old = f.with(&["propose", &s], plan!("- [R1] Old. Verify: run it"));
    f.ok(&["approve", &old]);
    let new = f.with(&["propose", &s], plan!("- [R1] New. Verify: run it"));
    f.with(&["check", &old, "R1", "pass"], b"ok");
    f.fails(&["check", &new, "R1", "pass"], b"ok");
    f.ok(&["approve", &new]);
    f.fails(&["check", &old, "R1", "pass"], b"ok");
    f.with(&["check", &new, "R1", "pass"], b"ok");
    assert_eq!(f.json(&["status", &old])["status"], "superseded");
}

#[test]
fn concurrent_writers_lose_nothing() {
    let f = Fixture::new();
    let s = f.ok(&["start", "Concurrent"]);
    let plan = f.with(&["propose", &s], SPEC);
    thread::scope(|scope| {
        for i in 0..12 {
            let (f, s, plan) = (&f, &s, &plan);
            scope.spawn(move || {
                f.with(&["note", s], format!("writer {i}").as_bytes());
                if i % 3 == 0 {
                    f.ok(&["approve", plan]);
                }
            });
        }
    });
    let notes: i64 = f
        .db()
        .query_one("SELECT count(*) FROM notes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(notes, 12);
    assert_eq!(f.json(&["status", &plan])["status"], "approved");
}

#[test]
fn worktrees_share_history_and_clones_do_not() {
    let f = Fixture::new();
    git(
        &f.repo,
        &[
            "-c",
            "user.name=T",
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "init",
        ],
    );
    let (linked, clone) = (f.base.join("linked"), f.base.join("clone"));
    git(
        &f.repo,
        &[
            "worktree",
            "add",
            "-q",
            "--detach",
            linked.to_str().unwrap(),
        ],
    );
    git(
        &f.base,
        &[
            "clone",
            "-q",
            f.repo.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    f.ok(&["start", "Shared task"]);
    let list = |dir: &Path| {
        let out = f.exec(&plans_bin(), dir, &f.state, &["list"], b"");
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap()
    };
    assert!(list(&linked).contains("Shared task"));
    assert_eq!(list(&clone), "");
    for state in [linked.join("state"), PathBuf::from("relative")] {
        let out = f.exec(&plans_bin(), &f.repo, &state, &["list"], b"");
        assert_eq!(out.status.code(), Some(1));
    }
    assert!(!linked.join("state").exists());
}

#[test]
fn empty_xdg_falls_back_to_private_home_state() {
    let f = Fixture::new();
    let out = f.exec(
        &plans_bin(),
        &f.repo,
        Path::new(""),
        &["start", "Default"],
        b"",
    );
    assert!(out.status.success());
    let root = f.base.join("home/.local/state/herdr-team");
    let folder = fs::read_dir(&root).unwrap().next().unwrap().unwrap().path();
    for path in [folder.join("herdr.sqlite3"), folder] {
        assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o077, 0);
    }
}

#[test]
fn launcher_resolves_a_linked_skill() {
    if std::env::var_os("PLANS_BIN").is_some() {
        return;
    }
    let f = Fixture::new();
    let skill = f.base.join("install/skills/herdr-grill");
    fs::create_dir_all(skill.join("bin")).unwrap();
    fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/skills/herdr-grill/plans"),
        skill.join("plans"),
    )
    .unwrap();
    let linked = f.base.join("linked-skill");
    symlink(&skill, &linked).unwrap();
    let launcher = linked.join("plans");
    let missing = f.exec(&launcher, &f.repo, &f.state, &["list"], b"");
    assert_eq!(missing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("herdr-plans is missing"));
    symlink(
        env!("CARGO_BIN_EXE_herdr-plans"),
        skill.join("bin/herdr-plans"),
    )
    .unwrap();
    let out = f.exec(&launcher, &f.repo, &f.state, &["start", "Linked"], b"");
    assert_eq!((out.status.code(), out.stdout), (Some(0), b"1\n".to_vec()));
}

#[test]
fn any_change_after_review_reopens_the_wiki_loop() {
    let f = Fixture::new();
    let s = f.ok(&["start", "Loop"]);
    let plan = f.with(&["propose", &s], plan!("- [R1] Done. Verify: run it"));
    f.ok(&["approve", &plan]);
    f.with(&["check", &plan, "R1", "pass"], b"ok");
    f.with(&["check", &plan, "integration", "pass"], b"ok");
    assert_eq!(f.review("t-a", &plan, "pass"), Some(0));
    assert_eq!(f.review("t-b", &plan, "fail"), Some(0));
    f.fails(&["finish", &plan], b"");
    fs::write(
        f.repo.join("knowledge/index.md"),
        "---\nokf_version: '0.2'\n---\n# Fixed\n",
    )
    .unwrap();
    assert_eq!(f.json(&["status", &plan])["wiki"]["passed_by"], 0);
    assert_eq!(f.review("t-a", &plan, "pass"), Some(0));
    assert_eq!(f.review("t-c", &plan, "pass"), Some(0));
    let status = f.json(&["status", &plan]);
    assert_eq!(
        (
            status["wiki"]["passed_by"].as_i64(),
            status["wiki"]["failed"].as_i64()
        ),
        (Some(2), Some(0))
    );
    fs::write(f.repo.join("untracked.txt"), "late edit").unwrap();
    f.fails(&["finish", &plan], b"");
    fs::remove_file(f.repo.join("untracked.txt")).unwrap();
    f.ok(&["finish", &plan]);
}

#[test]
fn edits_inside_nested_repositories_change_the_snapshot() {
    let f = Fixture::new();
    let s = f.ok(&["start", "Nested"]);
    let plan = f.with(&["propose", &s], plan!("- [R1] Done. Verify: run it"));
    let nested = f.repo.join("nested");
    fs::create_dir(&nested).unwrap();
    git(&nested, &["init", "-q"]);
    fs::write(nested.join("f"), "one").unwrap();
    let snapshot = || f.json(&["status", &plan])["wiki"]["snapshot"].clone();
    let before = snapshot();
    fs::write(nested.join("f"), "two").unwrap();
    assert_ne!(snapshot(), before);
}

#[test]
fn an_uninitialised_submodule_does_not_break_the_snapshot() {
    let f = Fixture::new();
    let s = f.ok(&["start", "Submodule"]);
    let plan = f.with(&["propose", &s], plan!("- [R1] Done. Verify: run it"));
    git(
        &f.repo,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000,1111111111111111111111111111111111111111,module",
        ],
    );
    fs::create_dir(f.repo.join("module")).unwrap();
    assert!(f.json(&["status", &plan])["wiki"]["snapshot"].is_string());
}
