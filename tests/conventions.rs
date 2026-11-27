use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn markdown(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(markdown(&path));
        } else if path.extension().is_some_and(|e| e == "md") {
            files.push(path);
        }
    }
    files
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[test]
fn layers_import_only_downward() {
    let allowed: &[(&str, &[&str])] = &[
        ("cli", &["application", "domain"]),
        ("application", &["domain", "repository", "storage"]),
        ("domain", &[]),
        ("storage", &[]),
        ("repository", &[]),
    ];
    for (layer, deps) in allowed {
        let source = read(&root().join(format!("src/{layer}.rs")));
        for (other, _) in allowed {
            let used = ["crate", "super"].iter().any(|root| {
                source.contains(&format!("{root}::{other}"))
                    || source
                        .split(&format!("use {root}::{{"))
                        .skip(1)
                        .any(|block| {
                            block
                                .split(';')
                                .next()
                                .unwrap()
                                .split(|c: char| !c.is_alphanumeric() && c != '_')
                                .any(|word| word == *other)
                        })
            });
            assert!(
                other == layer || !used || deps.contains(other),
                "src/{layer}.rs must not use `{other}`; allowed: {deps:?}"
            );
        }
    }
}

#[test]
fn wiki_follows_its_rules() {
    let knowledge = root().join("knowledge");
    let index = read(&knowledge.join("index.md"));
    for page in markdown(&knowledge.join("wiki")) {
        let name = page.file_name().unwrap().to_str().unwrap();
        let text = read(&page);
        let front = text
            .strip_prefix("---\n")
            .and_then(|t| t.split_once("\n---\n"))
            .unwrap_or_else(|| panic!("{name}: missing frontmatter"))
            .0;
        assert!(
            front.lines().any(|l| l.starts_with("type: ")),
            "{name}: missing type"
        );
        if let Some(status) = front.lines().find_map(|l| l.strip_prefix("status: ")) {
            assert!(
                ["draft", "stable", "deprecated"].contains(&status),
                "{name}: bad status {status}"
            );
        }
        for resource in front.split("resource: ").skip(1) {
            let target = resource.split([',', ' ', '}']).next().unwrap();
            assert!(
                target.starts_with("http") || page.parent().unwrap().join(target).exists(),
                "{name}: missing source {target}"
            );
        }
        assert!(
            index.contains(&format!("(wiki/{name})")),
            "index.md does not list {name}"
        );
    }
    let raw = read(&knowledge.join("raw/index.md"));
    for entry in fs::read_dir(knowledge.join("raw")).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        assert!(
            name == "index.md" || raw.contains(&format!("({name})")),
            "raw/index.md does not list {name}"
        );
    }
    let mut files = markdown(&knowledge);
    files.push(root().join("AGENTS.md"));
    for file in files {
        let text: String = read(&file).split('`').step_by(2).collect();
        for link in text.split("](").skip(1) {
            let target = link.split(')').next().unwrap().split('#').next().unwrap();
            if target.is_empty() || target.contains("://") || target.starts_with("mailto:") {
                continue;
            }
            assert!(
                file.parent().unwrap().join(target).exists(),
                "{}: broken link {target}",
                file.display()
            );
        }
    }
}

#[test]
fn every_command_is_on_the_feature_map() {
    let out = Command::new(env!("CARGO_BIN_EXE_herdr-plans"))
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8(out.stdout).unwrap();
    let commands = help.split_once('{').unwrap().1.split_once('}').unwrap().0;
    let map = read(&root().join("knowledge/wiki/architecture.md"));
    for command in commands.split(',') {
        assert!(
            map.contains(&format!("| `{command}`")) || map.contains(&format!("| `{command} ")),
            "knowledge/wiki/architecture.md has no row for `{command}`"
        );
    }
}

fn first_comment(source: &str) -> Option<usize> {
    let chars: Vec<char> = source.chars().collect();
    let (mut i, mut in_string) = (0, false);
    while i < chars.len() {
        match (chars[i], chars.get(i + 1), chars.get(i + 2)) {
            ('\\', ..) if in_string => i += 1,
            ('"', ..) => in_string = !in_string,
            ('\'', Some('\\'), _) if !in_string => {
                i += 2;
                while chars[i] != '\'' {
                    i += 1;
                }
            }
            ('\'', Some(_), Some('\'')) if !in_string => i += 2,
            ('/', Some('/' | '*'), _) if !in_string => {
                return Some(1 + chars[..i].iter().filter(|&&c| c == '\n').count());
            }
            _ => {}
        }
        i += 1;
    }
    None
}

#[test]
fn code_has_no_comments() {
    let help = "move the why into a name, an assert, an #[expect] reason or the wiki";
    let mut rust = Vec::new();
    for dir in ["src", "tests"] {
        for entry in fs::read_dir(root().join(dir)).unwrap() {
            rust.push(entry.unwrap().path());
        }
    }
    for file in rust {
        if let Some(line) = first_comment(&read(&file)) {
            panic!("{}:{line}: comment; {help}", file.display());
        }
    }
    for script in ["Makefile", "hooks/activate.sh", "skills/herdr-grill/plans"] {
        let text = read(&root().join(script));
        if let Some((n, _)) = text
            .lines()
            .enumerate()
            .find(|(n, l)| l.trim_start().starts_with('#') && !(*n == 0 && l.starts_with("#!")))
        {
            panic!("{script}:{}: comment; {help}", n + 1);
        }
    }
}

#[test]
fn lints_are_silenced_only_with_expect() {
    let forbidden = [["allow", "("].concat()];
    for dir in ["src", "tests"] {
        for entry in fs::read_dir(root().join(dir)).unwrap() {
            let file = entry.unwrap().path();
            let source = read(&file);
            for pattern in &forbidden {
                assert!(
                    !source.contains(pattern.as_str()),
                    "{}: use #[expect(lint, reason = \"...\")] instead of any {pattern}",
                    file.display()
                );
            }
        }
    }
}
