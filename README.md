<p align="center">
  <img src="assets/icon.png" width="144" height="144" alt="Orange and teal herding dog in profile">
</p>

<h1 align="center">Herdr Team</h1>

<p align="center"><em>Turn the stochastic into the deterministic.</em></p>

<p align="center">
  <img src="https://img.shields.io/badge/Claude_Code-supported-171717?style=flat-square" alt="Claude Code supported">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-171717?style=flat-square" alt="MIT license"></a>
</p>

Your master agent plans and reviews. Implementers work in their own Herdr tabs.
You see the work, approve the plan, and decide when it ships. What agents could
forget, the `plans` helper enforces: every plan is built on the project wiki, and it
closes only when independent reviewers confirm that code and wiki agree on the
exact working tree.

## Install

Use a local session inside **Herdr desktop** with Git, `hc` and
`herdr-app` on your PATH, and an authenticated Claude Code CLI.

The build targets one package file for macOS arm64, `herdr-claude.zip`. It
contains the native `skills/herdr-grill/bin/herdr-plans` executable and was
tested extracted on macOS 27.2.1 arm64. Extract the entire ZIP and keep the executable
permissions. The planning helper needs only Git: no Cargo or
first-run download. The ZIP also contains the
SessionStart hook used inside Herdr.

Marketplace installation from the source repository still needs a published
binary and an installation test. Linux build and checks
are available through the same Makefile. Linux package qualification is pending.

### Build from a source checkout

Run `make build` in this repository with Cargo, a native linker and a C
compiler for the bundled SQLite. The helper uses that build from the
checkout, including when the installed skill is a symlink.

To link the two skills for Claude Code, run these commands from this
repository after building.

```sh
mkdir -p "$HOME/.claude/skills"
ln -s "$PWD/skills/herdr-team" "$HOME/.claude/skills/herdr-team"
ln -s "$PWD/skills/herdr-grill" "$HOME/.claude/skills/herdr-grill"
```

The same commands work from an extracted package directory. Keep that
directory or the source checkout in place and start a new agent session
after adding the links.

### Developer checks and ZIPs

- `make check` runs Rust formatting, lint and tests, the hook checks and
  Claude manifest validation. It requires Cargo with rustfmt and Clippy and
  the Claude CLI.
- On macOS arm64, run `make release` followed by `make check-package` to produce
  and test the ZIP in `dist/`. `make package` requires the release executable
  at `target/release/herdr-plans` and rejects a missing or incompatible file.

## Put the team to work

> Use herdr-team to plan and implement this task with a team of agents in Herdr.

1. The wiki in `knowledge/` is the source of truth. The master creates it when
   missing, following [Open Knowledge Format 0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md).
2. Interview the dedicated `herdr-grill` agent. Each round saves one checkpoint,
   and the plan cites the wiki pages it is built on. Every criterion states how to
   verify it on the running product.
3. Approve the exact plan shown by the master. It assigns criteria to agents in
   fresh tabs, and each agent reads only the approved plan.
4. Implementers update code and wiki together and verify their own criteria on the
   running product. The master re-runs each verification and the integration,
   sending failures back for correction.
5. Two fresh reviewers compare wiki and code independently. Any discrepancy goes
   back to its owner and the loop restarts with new reviewers. `plans finish`
   closes the plan only when both pass on the current tree.

## Find or resume a plan

Ask the master to list this repository's sessions or resume a specific ID.
The bundled [plans](skills/herdr-grill/plans) script also works from the project
worktree: `list` (one line per session, no text), `resume SESSION` (latest
checkpoint only), `show PLAN` and `status PLAN` (results per criterion, no evidence).
No command replays history, so a new agent session starts with a clean context.

Each repository has its own SQLite database outside the checkout, at
`${XDG_STATE_HOME:-$HOME/.local/state}/herdr-team/<repo-key>/herdr.sqlite3`, with
`STRICT` tables `sessions`, `notes`, `plans` and `checks`, a `plan_status` view,
WAL, foreign keys and UTC ISO 8601 timestamps. Triggers keep checks append-only and plans undeletable, with fixed text and final
approval and verification. The [diagram](docs/herdr.drawio) shows the flow and the schema.
Linked worktrees share the database through the canonical Git common directory;
a separate clone gets its own. Plans stay out of the wiki and the repository.

Agents use your existing provider accounts and permissions. Provider usage may
incur costs. The package does not install Herdr or run in cloud-only chats.
The official CLI manuals come from `hc --skill` and `herdr-app --skill`.

## Small package. Clear boundaries.

- [The skill](skills/herdr-team/SKILL.md) — planning, ownership, gates, and handoffs.
- [Troubleshooting](skills/herdr-team/TROUBLESHOOTING.md) — when an agent gets stuck.
- [Privacy](PRIVACY.md) · [Support](https://github.com/pcodez99/herdr-skills/issues) · [MIT license](LICENSE).

By **[pcodez](https://github.com/pcodez99)**. An independent skill package for Herdr;
not an official Herdr or Anthropic product. Marketplace publication
requires a separate qualification step.
