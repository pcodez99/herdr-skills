# Log

## 2026-11-27

- Reviewed wiki against code with two independent reviewers on different models over nine rounds, until both passed on the same tree; the fixes made the engine stricter (undeletable plans and checks, final approval, refused malformed plan lines, wiki sources confined to `knowledge/`, snapshot covering nested repositories) and the [architecture](wiki/architecture.md) exact.
- Summarised a talk on trusting coding agents in [Agent trust and verification](wiki/agent-trust-and-verification.md); the transcript is third-party and not redistributed.
- Applied the talk to the project: rules in [AGENTS.md](../AGENTS.md), Clippy `pedantic` and `unsafe_code` lints, [conventions tests](../tests/conventions.rs) for layers, wiki rules and the command map; updated [architecture](wiki/architecture.md) and [Agent trust and verification](wiki/agent-trust-and-verification.md).
- Aligned the Herdr Team skills with the talk: `propose` requires `Verify: …` on every criterion, implementers verify on the running product, the wiki template adds a feature map, reviewers use a different model where possible.
- Banned code comments and unreasoned lint exceptions, as in the talk; moved the knowledge they held into names, asserts, `#[expect]` reasons and [architecture](wiki/architecture.md).
- Created the wiki and the [architecture](wiki/architecture.md) page from the code at commit `1647259`.
