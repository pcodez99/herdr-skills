---
name: herdr-team
description: Run a task in Herdr desktop as a team - a master that plans, reviews and gates, plus at least one implementer, each in its own watchable tab. Use when the user wants work done with agents or a team, wants agents or tabs in Herdr opened, checked or closed, or when your launch prompt names you an agent of a herdr-team.
---

# Herdr team

A **master** plans, reviews and gates. **Implementers** write the code, one tab each in the
Herdr window, where the person can watch them. A team is always the master plus at least
one implementer: whoever checks the work is never the one who wrote it. Beyond that
minimum, the master sizes and names the team from the task.

If your launch prompt names you an agent of a team, read [TEAMMATE.md](TEAMMATE.md) and
stop reading here. Otherwise you are the master.

Herdr has two CLIs. `herdr-app` drives what the person sees: spaces > tabs > panes. `hc`
drives the terminals inside them: start, message, read, wait. Load the `hc` and
`herdr-app` skills before the first command of each (`hc --skill` and `herdr-app --skill`
print them when they are not installed): they are the authority on syntax for the
installed version.

## 1. Size the team

Read the task and the code it touches. Start from one implementer, then add an agent for
each further piece that:

- has a **perimeter**: paths you can write down, plus the contracts it shares with others;
- runs in parallel with another piece;
- repays a cold start: each agent boots without your context and adds one more output
  for you to review.

Count perimeters, not files: thirty identical edits in one package are one piece. A
reviewer, a test writer or a researcher joins when the task calls for one, alongside the
implementer.

Name each agent after what it owns: the name is its tab title, its plan file and the
prefix of its reports. Lowercase, `a-z 0-9 - _`.

Examples: schema, API and UI for a new entity → `db`, `backend`, `frontend`. One change
in three packages → one agent per package. A rename inside one package → `rename` alone.

Done when every piece of the task has exactly one owner.

## 2. Plan and get approval

Write the plan where the project keeps plans of this kind, in its format. Without a
convention, use `temp/plans/<task>/`, kept out of git (`.git/info/exclude`):

- `00-master-plan.md`: goal, out of scope, team table (agent, perimeter, model, terminal
  address, tab ID), shared contracts, steps and gates in order, open questions, status
  table, log.
- `NN-<agent>.md` for each agent: files to imitate, numbered steps (`DB-1`, `DB-2`), a
  "done when" line for each step.

The project's format wins: where it keeps one file per plan, the team table and the
steps go in that file. Size the plan to the task: one implementer with one step fits in
a single short file.

A **gate** (`G1`, `G2`) is a check the master runs before others build on a result.
Place one after each shared contract, and one at the end. The plan is the team's memory: after a compaction, the team table
plus `hc ls` recovers the team.

Present the plan and the team: names, perimeters, models, why this size. Done when the
user approves; tabs and agents come after.

## 3. Open the tabs and launch

Agents live in your own space. [LIFECYCLE.md](LIFECYCLE.md) has the commands to inspect
the space, reuse a tab the user prepared, and create or close tabs and spaces.

One command opens the tab and starts the agent on its first step:

```sh
herdr-app do tab.new --space <space id> --name <agent> --folder <repo> \
  --argv '["claude","--model","<model>","You are the <agent> agent of a herdr-team: use the herdr-team skill. Plan: <plan folder>. Master: <your address>. Run step <ID>, then report with hc send <your address> and stop."]'
```

- Same provider as you (`claude`, or `codex --no-daemon`), in the permission mode your
  own session runs in, never a wider one.
- Model: one tier below yours for an agent following a detailed plan; your tier when the
  step is hard reasoning.
- Launch an agent when its first step is unblocked. Keep its prompt free of single
  quotes, since the whole `--argv` value sits inside them.

Done when `hc get <t>` shows each agent in status `working` and the team table holds its
address and tab ID.

## 4. Run the steps

- Send the next step with `hc send <t>` once its gate has passed.
- Wait on addresses with `hc wait <t>`: tabs opened by `herdr-app` are not your `hc`
  children. Reports arrive as `<hc from="…">` input reading
  `<agent>: <step> done - …` or `<agent>: <step> blocked - …`.
- A report is a claim. Read the diff, the test output and the log before marking a step
  done.
- A contract changes in `00-master-plan.md` first, then in a message to every agent it
  touches.
- An agent in status `blocked` waits for a person: read its screen and ask the user.

## 5. Gate and commit

At each gate, run its checks yourself, then ask the user before you commit. Commits and
pushes are theirs to authorise.

## 6. Finish

1. Run the project's full verification and update the docs it requires.
2. Report: the team, steps, gates and commits, check results, what is unverified or open.
3. Once the user accepts, close the tabs you created ([LIFECYCLE.md](LIFECYCLE.md)).
   Everything you did not create stays as it is.

When something misbehaves: [TROUBLESHOOTING.md](TROUBLESHOOTING.md).
