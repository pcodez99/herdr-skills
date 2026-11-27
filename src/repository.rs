use crate::Result;
use sha2::{Digest, Sha256};
use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, ErrorKind},
    os::unix::{
        ffi::{OsStrExt, OsStringExt},
        fs::MetadataExt,
    },
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

fn git(args: &[&str]) -> Result<Vec<u8>> {
    let result = Command::new("git")
        .args(args)
        .stderr(Stdio::inherit())
        .output()?;
    if !result.status.success() {
        return Err(format!("git {} failed ({})", args.join(" "), result.status).into());
    }
    Ok(result.stdout)
}

// Resolve existing symlinks before interpreting '..', including in missing suffixes.
fn resolve(path: &Path, depth: usize) -> Result<PathBuf> {
    if depth > 40 {
        return Err("Too many levels of symbolic links".into());
    }
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(name) => {
                out.push(name);
                match fs::symlink_metadata(&out) {
                    Ok(m) if m.is_symlink() => {
                        let dest = fs::read_link(&out)?;
                        out.pop();
                        out = resolve(&out.join(dest), depth + 1)?;
                    }
                    Err(e) if e.kind() != ErrorKind::NotFound => return Err(e.into()),
                    _ => {}
                }
            }
            _ => out.push(component.as_os_str()),
        }
    }
    Ok(out)
}

pub fn database() -> Result<PathBuf> {
    let top = git_path(&["rev-parse", "--show-toplevel"])?;
    let common = git_path(&["rev-parse", "--git-common-dir"])?;
    let key = format!("{:x}", Sha256::digest(common.as_os_str().as_bytes()));
    let state = env::var_os("XDG_STATE_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::home_dir().map(|p| p.join(".local/state")))
        .ok_or("Cannot determine home directory")?;
    if !state.is_absolute() {
        return Err("XDG_STATE_HOME must be absolute".into());
    }
    let folder = resolve(&state.join("herdr-team").join(key), 0)?;
    let mut protected = vec![common, top];
    for field in git(&["worktree", "list", "--porcelain", "-z"])?.split(|&b| b == 0) {
        if let Some(name) = field.strip_prefix(b"worktree ") {
            protected.push(resolve(Path::new(OsStr::from_bytes(name)), 0)?);
        }
    }
    let outside = "Plan storage must be outside every checkout and the Git directory";
    if protected.iter().any(|p| folder.starts_with(p)) {
        return Err(outside.into());
    }
    // Paths with different spellings can name the same directory on case-insensitive filesystems.
    let identities = protected
        .iter()
        .map(|p| directory_identity(p))
        .collect::<io::Result<Vec<_>>>()?;
    for parent in folder.ancestors() {
        if directory_identity(parent)?.is_some_and(|id| identities.contains(&Some(id))) {
            return Err(outside.into());
        }
    }
    fs::create_dir_all(&folder)?;
    let db = folder.join("herdr.sqlite3");
    if db.is_symlink() {
        return Err("Refusing a symlinked database".into());
    }
    Ok(db)
}

fn git_path(args: &[&str]) -> Result<PathBuf> {
    let mut bytes = git(args)?;
    while bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    Ok(fs::canonicalize(OsStr::from_bytes(&bytes))?)
}

fn directory_identity(path: &Path) -> io::Result<Option<(u64, u64)>> {
    match fs::metadata(path) {
        Ok(meta) => Ok(Some((meta.dev(), meta.ino()))),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn toplevel() -> Result<PathBuf> {
    git_path(&["rev-parse", "--show-toplevel"])
}

/// SHA-256 of the working tree as Git sees it: tracked and untracked files, ignored
/// ones excluded. Any byte changed in code or wiki yields a different snapshot.
pub fn snapshot() -> Result<String> {
    let top = toplevel()?;
    let listed = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .current_dir(&top)
        .stderr(Stdio::inherit())
        .output()?;
    if !listed.status.success() {
        return Err(format!("git ls-files failed ({})", listed.status).into());
    }
    let mut paths: Vec<&[u8]> = listed
        .stdout
        .split(|&b| b == 0)
        .filter(|p| !p.is_empty())
        .collect();
    paths.sort_unstable();
    paths.dedup();
    let mut hash = Sha256::new();
    for name in paths {
        let path = top.join(OsStr::from_bytes(name));
        let (kind, content): (&[u8], Vec<u8>) = match fs::symlink_metadata(&path) {
            Ok(m) if m.is_symlink() => (b"l", fs::read_link(&path)?.into_os_string().into_vec()),
            Ok(m) if m.is_file() => (
                if m.mode() & 0o111 != 0 { b"x" } else { b"f" },
                fs::read(&path)?,
            ),
            Ok(_) => (b"d", Vec::new()),
            Err(e) if e.kind() == ErrorKind::NotFound => (b"-", Vec::new()),
            Err(e) => return Err(e.into()),
        };
        // Length prefixes keep path and content boundaries unambiguous.
        for part in [name, kind, &content] {
            hash.update((part.len() as u64).to_le_bytes());
            hash.update(part);
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}
