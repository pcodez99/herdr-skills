# Herdr Team

The project wiki in [knowledge/](knowledge/index.md) is the source of truth: read it
before planning or changing code, and keep it in agreement with the implementation.
Motto: turn the stochastic into the deterministic. The reasoning behind these rules is in
[Agent trust and verification](knowledge/wiki/agent-trust-and-verification.md).

## Hard rules first

This file keeps only what no check can enforce. Before adding a rule here, try in order:

1. remove the problem by design (types, layers, the plans engine);
2. a test in [tests/conventions.rs](tests/conventions.rs), a lint in `Cargo.toml`, or a
   check in the `Makefile`;
3. only then a line below.

A review comment that repeats is a missing check: add it in the same change. The
checks already in place are listed under
[hard constraints](knowledge/wiki/architecture.md#hard-constraints).

## Find before you change

The [architecture page](knowledge/wiki/architecture.md) is the feature map: every command,
the rule it enforces, the layers and the hard constraints. Locate the feature there, then
read its code and tests. Read the code before naming a cause: a cause without a
reproduction is a guess.

The shortest path is the right one. A new command touches, in order: `COMMANDS` and its
arm in `parse()` in `src/cli.rs`, `Command` in `src/domain.rs`, its rule in `src/application.rs`, its SQL in
`src/storage.rs` (schema changes append to `MIGRATIONS`, never edit one), a case in
`tests/cli.rs`, a row in the architecture table. The tests fail if a layer imports
upward or the row is missing.

## Verify, then claim

Done means `make check` passes; for anything shipped in the ZIP, `make check-package`
too. Then run the product, not just the tests: `target/debug/herdr-plans` in a scratch
Git repository with `XDG_STATE_HOME` in a temporary directory, and
`HC_TERMINAL_ID=t-test sh hooks/activate.sh` for the hook. Report the commands you ran
and their output as evidence; never report a check you did not run.

## Small, atomic changes

One concern per commit, Conventional Commits, code and wiki in the same commit, so
history stays readable context and every revert is clean.

## Known failure modes

Observed agent failures that no check catches yet. When one repeats, turn it into a
check or a skill instruction, then delete its line.

- Guessing an API or behaviour instead of reading the code or `--help`.
- Declaring a "smoking gun" before reproducing it.
- Editing the code without updating the wiki page that describes it.
