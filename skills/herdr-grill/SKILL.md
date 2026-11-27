---
name: herdr-grill
description: Interview the user about a project, save and resume planning sessions in the repository's local plan database, and hand a numbered plan to the Herdr Team master for approval.
---

# Herdr Grill

Work from the target project's Git worktree. In the examples, `plans` means
the absolute path of the `plans` script beside this installed SKILL.md. It runs
the bundled native executable or an explicit `make build` from a source
checkout. Run it in the **project** working directory.
If the sandbox blocks the external state directory, request access to it before
continuing.

1. Use the session ID supplied by the master. For a new standalone interview,
   run `plans start "task title"` and tell the user the returned ID. For a
   resumed interview use `plans list` if needed, confirm the intended session,
   and run `plans resume SESSION`: it returns the latest checkpoint only.
2. Read the project's wiki in `knowledge/` (or `docs/wiki/`) first: it is the source of truth. Then the
   relevant code. Ask about decisions you cannot
   resolve from those sources. Give a recommendation and explain its trade-off.
   After each round save one complete checkpoint with `plans note SESSION` (UTF-8
   text on stdin): every confirmed answer, open question and change of mind so far.
   Each checkpoint replaces the previous one on resume, so it must stand alone.
   Do not save secrets. Never ask settled questions again.
3. When the user finishes, send clean Markdown to `plans propose SESSION` on
   stdin. Include the goal, ownership, dependencies, constraints, excluded scope,
   and testable criteria, one per line in the form
   `- [R1] Observable outcome. Verify: command or steps`, at the start of the line;
   mention criteria elsewhere without brackets. The verification runs the
   real product (the CLI, the app in a browser or simulator, the endpoint), not just
   reads the diff; `propose` refuses a criterion without it. If the wiki does not say
   how to run the product and reach each feature, the plan's first criterion adds
   that to the wiki: agents cannot verify what they cannot run.
   List the wiki pages the plan is built on, one per line as
   `- [wiki] knowledge/wiki/page.md` (or `- [wiki] docs/wiki/page.md`); `propose` refuses a plan without them. When the
   work changes what the wiki says, include the page updates in the plan.
   It prints the plan ID.
4. Give the master the session and plan IDs. Stay available for clarifications.
   A change creates another plan; earlier plans stay intact. The master owns
   approval and verification. Plans stay in the database, never in the wiki. A
   standalone interview can stop at this handoff.

The session history belongs to the master and interviewer. Never hand an
implementer the transcript, database path or an earlier plan. Use stdin or
a quoted heredoc for content; never interpolate user answers into shell commands.
