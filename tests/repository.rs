use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{PermissionsExt, symlink},
    },
    path::Path,
    process::Command,
};

#[test]
fn state_uses_common_dir_key_and_rejects_checkout_aliases_and_database_links() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("Repo");
    fs::create_dir(&repo).unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&repo)
            .status()
            .unwrap()
            .success()
    );
    let run = |state: &Path| {
        Command::new(env!("CARGO_BIN_EXE_herdr-plans"))
            .arg("list")
            .env("XDG_STATE_HOME", state)
            .current_dir(&repo)
            .output()
            .unwrap()
    };
    let alias = temp.path().join("linked");
    symlink(&repo, &alias).unwrap();
    let case_alias = temp.path().join("rEPO");
    for alias in [&alias, &case_alias].into_iter().filter(|p| p.is_dir()) {
        let state = alias.join("state");
        let output = run(&state);
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("Plan storage must be outside"));
        assert!(!state.exists());
    }

    let state = temp.path().join("state");
    let common = repo.join(".git").canonicalize().unwrap();
    let key = format!("{:x}", Sha256::digest(common.as_os_str().as_bytes()));
    let folder = state.join("herdr-team").join(key);
    let db = folder.join("herdr.sqlite3");
    assert!(run(&state).status.success());
    assert_eq!(
        fs::metadata(&db).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(&folder).unwrap().permissions().mode() & 0o777,
        0o700
    );

    let saved = folder.join("saved.sqlite3");
    fs::rename(&db, &saved).unwrap();
    symlink(&saved, &db).unwrap();
    let output = run(&state);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Refusing a symlinked database"));
}
