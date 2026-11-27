use crate::{
    Result,
    application::{self, Outcome},
    domain::Command,
};
use std::{
    env,
    ffi::OsString,
    io::{self, Read, Write},
    process,
};

const COMMANDS: &[(&str, &str)] = &[
    ("start", "title"),
    ("list", ""),
    ("resume", "session"),
    ("note", "session"),
    ("propose", "session"),
    ("show", "plan"),
    ("status", "plan"),
    ("approve", "plan"),
    ("check", "plan criterion result"),
    ("finish", "plan"),
];

fn invalid(usage: &str, msg: &str) -> ! {
    eprintln!("{usage}\nherdr-plans: error: {msg}");
    process::exit(2)
}

fn body() -> Result<String> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    match String::from_utf8(bytes) {
        Ok(body) if !body.trim().is_empty() => Ok(body),
        _ => Err("Expected non-empty UTF-8 text on stdin".into()),
    }
}

fn parse() -> Result<Option<Command>> {
    let mut argv = env::args_os().skip(1);
    let commands = COMMANDS.iter().map(|p| p.0).collect::<Vec<_>>().join(",");
    let top = format!("usage: herdr-plans [-h] {{{commands}}} ...");
    let help = |s: &std::ffi::OsStr| s == "-h" || s == "--help";
    let Some(cmd) = argv.next() else {
        invalid(&top, "missing command")
    };
    if help(&cmd) {
        println!("{top}\n\nPer-repository plan history. Run from the project's Git worktree.");
        return Ok(None);
    }
    let Some(&(cmd, params)) = COMMANDS.iter().find(|p| cmd == p.0) else {
        invalid(
            &top,
            &format!("unknown command '{}'", cmd.to_string_lossy()),
        )
    };
    let usage = format!("usage: herdr-plans {cmd} {params}")
        .trim_end()
        .to_owned();
    let names: Vec<_> = params.split_whitespace().collect();
    let mut args: Vec<OsString> = Vec::new();
    let mut options = true;
    for arg in argv {
        if options && arg == "--" {
            options = false;
        } else if options && help(&arg) {
            println!("{usage}");
            return Ok(None);
        } else if options && arg.len() > 1 && arg.to_string_lossy().starts_with('-') {
            invalid(
                &usage,
                &format!("unknown option '{}'", arg.to_string_lossy()),
            )
        } else {
            args.push(arg);
        }
    }
    if args.len() != names.len() {
        invalid(&usage, &format!("expected {} argument(s)", names.len()))
    }
    let text = |i: usize| args[i].to_str().ok_or("Expected valid UTF-8 argument");
    let id = |i: usize| -> Result<i64> {
        let value = text(i)?;
        match value.parse() {
            Ok(id) => Ok(id),
            Err(_) => invalid(
                &usage,
                &format!("{} must be an integer, got '{value}'", names[i]),
            ),
        }
    };
    Ok(Some(match cmd {
        "start" => Command::Start(text(0)?.into()),
        "list" => Command::List,
        "resume" => Command::Resume(id(0)?),
        "note" => Command::Note(id(0)?, body()?),
        "propose" => Command::Propose(id(0)?, body()?),
        "show" => Command::Show(id(0)?),
        "status" => Command::Status(id(0)?),
        "approve" => Command::Approve(id(0)?),
        "check" => {
            let result = text(2)?;
            if !matches!(result, "pass" | "fail") {
                invalid(
                    &usage,
                    &format!("result must be pass or fail, got '{result}'"),
                )
            }
            Command::Check {
                plan: id(0)?,
                criterion: text(1)?.into(),
                result: result.into(),
                evidence: body()?,
            }
        }
        "finish" => Command::Finish(id(0)?),
        _ => unreachable!(),
    }))
}

pub fn run() -> Result<()> {
    let Some(cmd) = parse()? else { return Ok(()) };
    let mut out = io::stdout().lock();
    match application::execute(cmd)? {
        Outcome::Id(id) => writeln!(out, "{id}")?,
        Outcome::Text(text) => out.write_all(text.as_bytes())?,
        Outcome::Silent => {}
    }
    Ok(out.flush()?)
}
