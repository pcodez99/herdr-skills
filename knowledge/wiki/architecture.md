---
type: Architecture
title: Architecture
description: Components, plan database schema, enforced rules and distribution of Herdr Team.
status: draft
generated: { by: claude-code/claude-opus-5-5, at: '2026-11-27T12:13:26Z' }
sources:
  - { resource: ../../src/storage.rs, id: schema }
  - { resource: ../../src/application.rs, id: rules }
  - { resource: ../../src/domain.rs, id: domain }
  - { resource: ../../src/repository.rs, id: repository }
  - { resource: ../../skills/herdr-team/SKILL.md, id: skill }
  - { resource: ../../Makefile, id: build }
  - { resource: ../../docs/herdr.drawio, id: diagram }
  - { resource: ../../tests/conventions.rs, id: conventions }
  - { resource: ../../Cargo.toml, id: lints }
---

# Architecture

Motto: turn the stochastic into the deterministic. Whatever an agent could forget,
the `herdr-plans` engine enforces. The [diagram](../../docs/herdr.drawio) shows the
flow and the schema.

## Components

| Part | Role |
| --- | --- |
| `skills/herdr-team/` | Master and teammate instructions, wiki template with the feature map ([WIKI.md](../../skills/herdr-team/WIKI.md)). |
| `skills/herdr-grill/` | Interviewer instructions and the `plans` launcher (POSIX sh), which runs the packaged binary or, in the source checkout, the local build. |
| `src/` | `herdr-plans`, a Rust CLI over SQLite (rusqlite, bundled). |
| `hooks/` | Claude Code SessionStart hook; silent outside Herdr. |

`src/` layers: `cli` parses arguments and stdin, `application` applies the rules,
`domain` parses criteria and wiki sources, `storage` owns SQL, `repository` resolves
Git identity, the database path and the working-tree snapshot.[^rules]

## Plan database

One file per repository at `${XDG_STATE_HOME:-~/.local/state}/herdr-team/<key>/herdr.sqlite3`,
where `<key>` is the SHA-256 of the canonical Git common directory: linked worktrees share
it, clones do not. `XDG_STATE_HOME` must be absolute; empty means the default. The file
must lie outside every checkout: paths are resolved through symlinks before `..`, and
directories compared by device and inode, so no other spelling of a checkout passes. The
database is opened without following symlinks. Every file the CLI creates, SQLite
journals included, is private to the user (umask `077`).[^repository]

`STRICT` tables, WAL, foreign keys, UTC ISO 8601 timestamps, `application_id` `HRDR`,
migrations counted by `user_version` and never edited: a change is a new migration. A
database with another schema is refused before anything is written to it. Each
command's database work runs in one `IMMEDIATE` transaction, waiting up to 10 s for
another writer.[^schema]

- `sessions(id, title, created)`
- `notes(id, session, body, created)`: interview checkpoints; the latest replaces the rest on resume.
- `plans(id, session, body, created, approved, approver, verified)`: never deleted; the
  text never changes, and approval and verification, once set, are final (triggers).
  `approved` and `approver` are set together; `verified` needs `approved`.
- `checks(id, plan, criterion, result, evidence, reviewer, snapshot, created)`: append-only,
  no update or delete (triggers); a `wiki` check needs its snapshot.
- `plan_status` view: `draft`, `approved` (the session's active plan), `superseded`, `verified`.

Notes and sessions without plans have no such protection.

The triggers stop mistakes through SQL, not a person who owns the file: anyone with
write access can drop them.

## Commands and enforced rules

The feature map: every command of `herdr-plans`, what it reads and the rule it
enforces. A test fails if a command has no row here.[^conventions] `list`, `resume` and
`status` print JSON; `list` and `resume` report `interview` for a session with no plan.

| Command | Reads | Prints | Rule |
| --- | --- | --- | --- |
| `start title` | | session id | The title is not blank. |
| `list` | | one line per session, newest first: id, title, latest plan, status | |
| `resume session` | | the session line plus its latest checkpoint | |
| `note session` | stdin | | The session exists. |
| `propose session` | stdin, wiki files | plan id | Unique `- [R<n>]` criteria (`n` positive, no leading zero) at the start of the line, each with text before and after `Verify:`; any other line containing a marker is refused: `[`, optional whitespace, `R` or `r`, optional whitespace, then a digit or `]` (so `[R2]`, `[R]` and even `[R2D2]`), or `[wiki]` in any case or spacing. Criteria are mentioned elsewhere without brackets; a well-formed line may contain such tokens. At least one `- [wiki] …` line, each starting with `knowledge/` or `docs/wiki/` (a plan may mix them), and every such line names an existing file, with no empty, `.` or `..` path segment, that, symlinks resolved, stays inside the folder it starts with; that folder may be a symlink to a wiki elsewhere, and sources then stay inside its target. The snapshot hashes such a symlink by its target path, not the files behind it, so edits to an external wiki do not invalidate reviews. These checks run before the transaction. |
| `show plan` | | plan text | |
| `status plan` | working tree | plan, session, status, latest result per criterion and `integration`, wiki snapshot with `passed_by` and `failed`; no evidence | |
| `approve plan` | | | Only the session's newest plan; records the approving terminal (`HC_TERMINAL_ID`, required for the first approval). Approving it again, from any terminal, does nothing and keeps the first approver. A newer draft leaves the approved plan active until it is approved itself. |
| `check plan criterion result` | stdin evidence; working tree for `wiki` | | `result` is `pass` or `fail`, else a usage error. Needs a non-blank `HC_TERMINAL_ID`. Only the active approved, unverified plan, for one of its criteria, `integration` or `wiki`. Criteria and `integration` take any terminal, the approver's included, and the latest result counts. A `wiki` check comes from a terminal other than the approver and stores the working-tree snapshot. |
| `finish plan` | working tree | | Only the active approved, unverified plan. Latest result of every criterion and `integration` is `pass`; at least 2 distinct reviewers passed `wiki`, with no failure, on the current snapshot. Needs no `HC_TERMINAL_ID`. |

`-h` and `--help` print usage and exit 0. Usage errors (unknown commands, non-integer
ids, wrong argument counts, non-UTF-8 arguments; any argument of two or more characters
starting with `-` needs `--` before it) exit 2, rule errors 1. Stdin must be UTF-8 and not blank; bad stdin is a rule error. `Verify:` is checked
for presence only: running it is the agents' job.[^rules]

The snapshot is a SHA-256 over every file Git tracks, even one matching an ignore rule,
plus every untracked file not ignored (including the user's global excludes), deleted tracked files included: path, kind (file, executable, symlink, or a nested
repository or checked-out submodule, hashed recursively the same way; a submodule that
is not checked out adds only its path; files inside its folder are not hashed) and content, link target or
nested hash, each length-prefixed; permission bits other than
executable do not count. Any change invalidates earlier wiki reviews. A `wiki` failure
counts until the tree changes, even if the same reviewer later passes, so the review
loop restarts. The hash is of content, so undoing a change brings back the reviews
recorded on that content, failures included. Linked worktrees share the database, but
each command hashes the worktree it runs in.[^repository]

Reviewer identity is the self-reported `HC_TERMINAL_ID`: the engine proves two distinct
terminals other than the approver's, not that the agents behind them are independent.
The [herdr-team skill](../../skills/herdr-team/SKILL.md) provides independence by
opening fresh reviewer agents.[^rules]

A command that passes its argument, stdin and plan checks opens the database, creating
the state folder and file if needed, read-only commands included. The first one that
succeeds migrates the schema; a failing one rolls its migration back, leaving a WAL
database with no schema, which the next run migrates.
Outputs keep agent context clean: no command replays history.[^schema]

## Hard constraints

Rules that turn `make check` red, so no agent has to remember them:[^conventions]

- `unsafe_code` is denied in `Cargo.toml`, except the one `umask` call in `main.rs`,
  under `#[expect]` with its reason; Clippy `pedantic`, `allow_attributes` and
  `allow_attributes_without_reason` warn there, and `make check` runs Clippy with
  `-D warnings`, so they fail it. A lint is silenced only with
  `#[expect(lint, reason = "...")]`: a test also refuses the text `allow(` anywhere in
  the Rust sources of `src/` and `tests/`, inner and `cfg_attr` forms included; rustfmt and
  Clippy catch spaced forms.
- No comments in Rust (`src/`, `tests/`), nor full-line comments in the `Makefile`,
  `hooks/activate.sh` and the `plans` launcher: the why goes in a name, an assert, an
  `#[expect]` reason or this wiki. SQL comments inside the first migration stay, because
  migrations are never edited.
- Layers import only downward: `cli` may use `application` and `domain`; `application`
  may use `domain`, `storage` and `repository`; the lower three use none of each other,
  through `crate::` or `super::`.
- Pages in `knowledge/wiki/` have frontmatter with `type`, a valid `status` when present,
  sources that resolve, and an index entry; every raw source is indexed; relative links
  in `knowledge/` and the root `AGENTS.md` resolve.
- Every CLI command has a row in the table above.

## Build and distribution

`make check` runs rustfmt, Clippy, the Rust tests (including the hard constraints above),
hook checks and Claude plugin validation. `make release` builds the release binary;
`make package` puts an existing macOS arm64 release binary in `dist/herdr-claude.zip`,
refusing any other, and `make check-package` packages and tests that ZIP, running the CLI tests through the
extracted launcher (`PLANS_BIN`) with only Git on `PATH`. The launcher runs
`bin/herdr-plans` from the package, else `target/debug`, else `target/release` in the
source checkout. Linux builds but is not qualified.[^build]

[^schema]: [storage.rs](../../src/storage.rs)
[^rules]: [application.rs](../../src/application.rs), [domain.rs](../../src/domain.rs)
[^repository]: [repository.rs](../../src/repository.rs)
[^build]: [Makefile](../../Makefile)
[^conventions]: [conventions.rs](../../tests/conventions.rs), [Cargo.toml](../../Cargo.toml)
