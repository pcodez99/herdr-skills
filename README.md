# herdr-skills

Work with a team of agents in Herdr desktop: a master that
plans, reviews and commits, plus at least one implementer, each in its own tab.

- [`skills/herdr-team`](skills/herdr-team/SKILL.md): the skill.
- `plugin/`: a Claude Code plugin with a single hook that points to the skill at
  session start inside Herdr. It ships no skills, so each skill exists only once.

## Installation

Install Herdr desktop with the `hc` and `herdr-app` commands available on your PATH,
and the agent CLI you intend to use (Claude Code or Codex). The commands below use a
POSIX shell and Git.

```sh
git clone https://github.com/pcodez99/herdr-skills.git
cd herdr-skills
```

Link the skill into your agent's skill directory. For Claude Code:

```sh
mkdir -p "$HOME/.claude/skills"
ln -s "$PWD/skills/herdr-team" "$HOME/.claude/skills/herdr-team"
```

For Codex, use `$HOME/.codex/skills` instead of `$HOME/.claude/skills`. Keep the
clone in place because the link points to it. If `herdr-team` already exists in the
destination, check the existing installation before replacing it.

Claude Code users can optionally install the session-start hook:

```sh
claude plugin marketplace add "$PWD"
claude plugin install herdr@herdr-skills
```

The hook runs only inside Herdr and does not install the skill itself.

`herdr-team` relies on the official `hc --skill` and `herdr-app --skill` manuals.
Read them directly from the installed binaries; they belong to Herdr and are not tracked here.

## Usage

In an agent session inside Herdr, ask the agent to use `herdr-team` for a task.
The skill plans the team and asks for approval before launching agents. Review the
plan and the requested permissions; each agent uses your existing CLI account.
