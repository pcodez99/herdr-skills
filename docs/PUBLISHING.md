# Publish Herdr Team

The packages are prepared for local validation and submission. They are not
approved directory listings. Publisher verification, portal checks, name
availability, and live workflow testing remain separate steps.

## Release 0.6.0

- Publisher: **pcodez**, with no personal email in package metadata.
- Stable package name: `herdr`; display name: **Herdr Team**.
- License: MIT, copyright 2026 pcodez.
- Icon: `assets/icon.png`, shared by both packages.
- Requires Herdr desktop, `hc`, `herdr-app`, and an authenticated agent CLI.
- No MCP server, hosted backend, or publisher account is required.
- Claude Code includes the skill and a local session-start hook.
- Codex includes the skill and declares `requires_local_executor: true`.
  Its submission ZIP excludes Claude hooks.

The GitHub project is the support and documentation home. Privacy information is
in [PRIVACY.md](../PRIVACY.md). The [MIT license](../LICENSE) governs this package;
third-party accounts and software have their own terms.

## Validate and package

From a clean clone, with Python 3.9+, Make, Zip, Claude Code, and Codex available:

```sh
make check
make package
codex plugin marketplace add "$PWD"
codex plugin list --marketplace herdr-skills --available --json
```

`make check` builds and checks the ZIP contents and runs Claude's strict local
directory profile. It does not run agents or complete either portal's checks.
`make package` uses an explicit list of package files, excluding Git metadata,
local manuals, tests, build files, and credentials. Outputs are ignored by Git:

| Package | Purpose |
| --- | --- |
| `dist/herdr-codex.zip` | OpenAI upload: portable manifest, skill, icon, license, README, privacy |
| `dist/herdr-claude.zip` | Standalone Claude testing: Claude manifest, skill, hook, icon, license, README, privacy |

The Claude directory reads GitHub, not this ZIP. Both manifests are at the
repository root, and both marketplaces point there. Raise the version in both
manifests before releasing another package update.

## Test before submission

Install each package in a clean agent setup without an existing `herdr-team`
symlink. Use a disposable sample repository inside Herdr. These scenarios are a
review checklist, not a record of executed agent tests:

| Scenario | Prompt or action | Expected behavior |
| --- | --- | --- |
| Installation | Install; start a fresh session. | One copy of `herdr-team` is available. |
| Plan | “Use herdr-team to add a small feature.” | Propose ownership, steps, and checks; wait for plan approval. |
| Launch | Approve the plan. | Start agents in visible tabs with the session's permission scope. |
| Review | Let an implementer report completion. | Master inspects changes and runs checks. |
| Finish | Accept work and authorize cleanup. | Commit/push only when authorized; close only team-created tabs. |
| No approval | Decline the plan. | Do not launch agents. |
| Missing prerequisite | Run without `hc` or `herdr-app`. | Explain what is missing; do not invent command results. |
| Existing user work | Prepare an unrelated busy tab. | Preserve it; ask before reusing a busy tab. |

Also check the Claude hook is silent outside Herdr and emits a single reminder
inside it. Automated tests cover its two branches; real agent-session behavior
still needs a desktop smoke test. No reviewer credentials are bundled.

## OpenAI directory: ChatGPT and Codex

1. Open [OpenAI Plugins](https://platform.openai.com/plugins). Select the owning
   organization/project and complete developer or business verification. The
   directory publisher name comes from the verified identity; local metadata
   does not override it.
2. Start a **Skills only** submission and upload `dist/herdr-codex.zip`.
   This package needs local execution and must be listed as **Desktop only**.
   It has no local MCP server. If the portal rejects the local-executor field,
   confirm submission access with OpenAI.
3. Resolve metadata and skill findings. Check the icon and listing copy, and
   complete review information requested by the portal using the scenarios above.
   Provide any required contact information privately in the portal.
4. Submit for review; publish after approval. Do not claim a directory listing
   before publication.

Sources: [Submission](https://developers.openai.com/plugins/deploy/submission),
[desktop-only skills](https://developers.openai.com/plugins/deploy/desktop-only),
[package format](https://developers.openai.com/plugins/build/plugins).

## Claude directory

1. Open [Claude's developer portal](https://claude.ai/directory/manage) with an
   eligible paid account and submission role. Connect a GitHub account that can
   push to `pcodez99/herdr-skills`.
2. Select **Submit new → Plugin bundle**. Enter repository
   `https://github.com/pcodez99/herdr-skills`, leave the plugin path empty (root),
   and track branch `main`.
3. Select **Validate**. Resolve blocking findings and revalidate after any push.
   Review the README, icon, publisher, license, and privacy link.
4. Explain that this workflow uses local Herdr terminals. The session-start hook
   is for Claude Code; it does not enable the workflow in cloud-only chats.
   Check the supported surfaces before publishing.
5. Submit, complete review, and publish the passing version in the portal.

This directory route can reach Claude Code through account sync. Anthropic's
`claude-plugins-official` marketplace is a separate channel; listing there requires
coordination with Anthropic, not just this portal submission.

Sources: [Publish a plugin](https://code.claude.com/docs/en/plugins/publish),
[submission instructions](https://claude.com/docs/plugins/submit),
[pre-submission checklist](https://claude.com/docs/plugins/pre-submission-checklist).
