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

Name each agent after what it owns: the name is its tab title and the
prefix of its reports. Lowercase, `a-z 0-9 - _`.

Examples: schema, API and UI for a new entity → `db`, `backend`, `frontend`. One change
in three packages → one agent per package. A rename inside one package → `rename` alone.

Done when every piece of the task has exactly one owner.

## 2. Wiki, plan and approval

The project's wiki in `knowledge/` is the source of truth. Create it if the project has
none: [WIKI.md](WIKI.md). Plans are built on it and never stored in it.

Rules the `plans` helper enforces, so no agent has to remember them:

| Rule | Enforced by |
| --- | --- |
| A plan cites the wiki pages it is built on | `propose` rejects a plan without `- [wiki] knowledge/…` lines naming existing pages |
| A plan's text never changes | database trigger; a change is a new plan |
| Only the newest plan can be approved, and only the approved one collects evidence | `approve`, `check` |
| Code and wiki agree | `finish` needs 2 `wiki` passes from distinct reviewer terminals, not the approving master's, on the current working tree, and no failure on it |
| Any later edit reopens the review | each `wiki` check stores a hash of the working tree; a changed byte invalidates it |

Load the bundled `herdr-grill` skill. From the target Git worktree use its
`plans` helper: one SQLite database per repository, outside the checkout, with the
tables `sessions`, `notes`, `plans` and `checks`. Run `plans start "task title"`, or
resume the session explicitly selected by the user. Show the session ID.

Open one dedicated interview tab in the team's Herdr space (see [LIFECYCLE.md](LIFECYCLE.md)).
Launch a fresh interviewer with the **most capable available model of the user's
chosen provider**, the current permission scope, the session ID, master's address,
and instructions to use herdr-grill. Keep this tab available until task closure.
Only the interviewer and master read the session, and only through `plans resume`:
the latest checkpoint, never the full history.

When the interviewer hands over a plan, read `plans show PLAN`. Check scope,
assignments, shared contracts, acceptance criteria and integration checks. If changes
are needed, request another proposal. Show the user the exact plan text and its ID. An
interview ending is not approval. After the user explicitly approves this plan, run
`plans approve PLAN`. The approved text stays in the database, unchanged.

## 3. Open the tabs and launch

Agents live in your own space. [LIFECYCLE.md](LIFECYCLE.md) has the commands to inspect
the space, reuse a tab the user prepared, and create or close tabs and spaces.

One command opens the tab and starts the agent on its first step:

```sh
herdr-app do tab.new --space <space id> --name <agent> --folder <repo> \
  --argv '["claude","--model","<model>","You are the <agent> agent of a herdr-team: use the herdr-team skill. Approved plan: run <absolute plans path> show <plan> in the repo. Scope <ownership>, criteria <R-IDs>. Master: <your address>. Run step <ID>, then report with hc send <your address> and stop."]'
```

- `claude`, in the permission mode your own session runs in, never a wider one.
- Model: one tier below yours for an agent following a detailed plan; your tier when the
  step is hard reasoning.
- Launch an agent when its first step is unblocked. Keep its prompt free of single
  quotes, since the whole `--argv` value sits inside them.

Done when `hc get <t>` shows each agent in status `working`. Agents are named after what
they own, so `herdr-app ls` is the team roster; the plan database holds no roster.

## 4. Run the steps

- Send the next step with `hc send <t>` once its gate has passed.
- Wait on addresses with `hc wait <t>`: tabs opened by `herdr-app` are not your `hc`
  children. Reports arrive as `<hc from="…">` input reading
  `<agent>: <step> done - …` or `<agent>: <step> blocked - …`.
- Verify each assigned criterion independently against the approved plan, code diff
  and observed behavior. Run its check. Send the actual command, result and relevant
  paths on stdin to `plans check PLAN R1 pass` or `fail`.
  On failure return the criterion, expected and actual result to the owner, then
  rerun the check after correction. Escalate unresolved blockers to the user.
- When the plan changes what the wiki describes, its owner updates the wiki pages in the
  same step as the code: both must agree before the review loop.
- A scope or contract change pauses affected work. The interviewer proposes a new
  plan. Obtain approval and brief affected agents with the new plan ID. Evidence
  belongs to its plan; recheck criteria against the new one.
- An agent in status `blocked` waits for a person: read its screen and ask the user.

## 5. Gate and commit

At each gate, run its checks yourself, then ask the user before you commit. Commits and
pushes are theirs to authorise.

## 6. Close the loop

1. Run project and cross-agent integration checks on the final combined changes.
   Rerun any criterion affected by later edits. Record integration evidence with
   `plans check PLAN integration pass` or `fail`.
2. **Wiki review loop.** Open two fresh reviewer tabs that took no part in the work,
   each with its own context. Each one compares the whole wiki with the implementation,
   independently of the other, and records `plans check PLAN wiki pass` or `fail` from
   its own terminal, listing every discrepancy as evidence.
   Any failure: send the discrepancies to the owner, who fixes the code (or the wiki,
   where the plan changed the intended behaviour). Close the reviewer tabs and open
   two fresh ones on the new tree. Repeat until `plans status PLAN` shows
   `"passed_by": 2, "failed": 0`. Then run `plans finish PLAN`: it re-hashes the tree
   and refuses if anything changed since the reviews.
3. Report: the team, steps, gates and commits, check results, review rounds, what is
   unverified or open.
3. Once the user accepts, close the tabs you created ([LIFECYCLE.md](LIFECYCLE.md)).
   Everything you did not create stays as it is.

When something misbehaves: [TROUBLESHOOTING.md](TROUBLESHOOTING.md).
