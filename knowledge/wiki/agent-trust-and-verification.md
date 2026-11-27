---
type: Reference
title: Agent trust and verification
description: How an engineer at Cursor built enough trust in coding agents to auto-merge their work, and what it implies for Herdr Team.
status: draft
generated: { by: claude-code/claude-opus-5-5, at: '2026-11-27T12:20:00Z' }
---

# Agent trust and verification

Summary of a recorded talk with Q&A, given by an engineer at Cursor. The source is an
automatic transcript, not redistributed here because it is third-party content; the
event and date are not stated in it.[^talk]

## What the talk claims

- **Trust is the bottleneck.** Without it you micromanage a handful of agents and cannot
  parallelise. The speaker climbed a "trust curve" over five months, to shipping about a
  thousand PRs in the last month, with agents that auto-merge PRs reviewed afterwards
  on `main`.[^talk]
- **Verification closes the loop.** The most important skill is letting the agent run
  the real product (Chrome DevTools Protocol, simulators, traces, heap snapshots) and
  check its own work. It makes code correct, not necessarily good; without it the human
  is the verifier and the bottleneck.[^talk]
- **Feature map.** A verification skill also needs a map of the product: every feature,
  how to reach it in the UI, shortcuts and selectors. It turns vague reports and bare
  screenshots into reproducible paths. It is generated from the code and maintained
  by a separate skill.[^talk]
- **Skills from observed failures.** Each recurring failure (guessing, not reading the
  code, overconfident "smoking guns") becomes a skill, found by reading tool calls and
  thinking blocks.[^talk]
- **Evals as unit tests for skills.** A coordinator writes a rubric and runs sub-agents
  in neutrally named directories, because agents behave differently when they know they
  are evaluated. A judge from a different model cross-checks the coordinator for bias,
  and a loop repeats until every case scores full marks. Evals run across models.[^talk]
- **Hard constraints beat soft ones.** Layers, strongest first: codebase conventions
  and architecture, static analysis (CI checks, lints, compiler diagnostics, import
  boundaries), then rules, skills, AI review and style guides. Only the first two turn
  CI red; the rest can be forgotten. The speaker's CI even bans code comments, because
  agents mostly write historical, irrelevant ones, such as turning a one-off review
  remark into a global rule. Every review comment that repeats is a smell: turn
  it into a lint, a CI failure, or remove the problem by design.[^talk]
- **Make the shortest path the right path.** Agents copy patterns and take shortcuts, so
  conventions should make the convenient solution the correct one, with each feature
  colocated in one directory.[^talk]
- **Greenfield code needs constraints first.** Vibe-coded prototypes grow an "organic
  architecture" nobody understands. The speaker spent over 600 PRs refactoring one to a
  strict architecture, after which non-engineers could contribute safely.[^talk]
- **Scale only after trust.** Start locally to observe agents, then cloud agents and
  automations (e.g. one that reproduces every bug report), then auto-merge. Jumping
  straight to many agents wastes tokens. The up-front token cost is framed as an ROI
  decision.[^talk]
- **Small, atomic PRs.** No hard size cap, but split work so Git history stays readable
  context and reverts stay easy.[^talk]

## Implications for Herdr Team

Inference, not from the source: the talk supports the project's motto, "turn the
stochastic into the deterministic". Applied so far:

- **Hard constraints on the process.** Plan rules live in the
  [plans engine](architecture.md), not only in skill text, and the wiki review needs
  two distinct reviewer terminals besides the approver's, which the skill fills with
  fresh agents, like the talk's separate judge; the engine cannot prove the agents are
  independent.
- **Hard constraints on the code.** Clippy `pedantic`, denied `unsafe_code`, reasoned
  `#[expect]` only, no code comments, downward-only layer imports and the wiki rules fail
  `make check`; see
  [hard constraints](architecture.md#hard-constraints).
- **Feature map.** The [architecture](architecture.md) command table maps every command to
  its rule; a test fails if a command is missing.
- **Soft rules last.** The project [AGENTS.md](../../AGENTS.md) keeps only what no check
  enforces, requires verification by running the product, and lists observed failure
  modes to promote into checks.

- **Verification in the team.** `propose` refuses a criterion without `Verify: …`;
  implementers run it on the real product before reporting and the master re-runs it.
  The wiki template requires a feature map, and the plan builds one first when missing.
- **Different judge.** At least one wiki reviewer runs on a different model from the
  implementers when one is available.
- **Failures become checks.** The master's report proposes a hard check for any failure
  or review remark seen twice; teammates follow the observed failure modes.

Open ideas, not decided or planned:

- evals for the plugin's skills, run in neutrally named directories across models.

[^talk]: Automatic transcript of the talk, provided by the maintainer on 2026-11-27; not redistributed.
