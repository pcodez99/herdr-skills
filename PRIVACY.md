# Privacy

Herdr Team by pcodez is an independent package of agent instructions, a local
planning executable and a Claude Code session-start hook. It has no hosted
backend, analytics, telemetry, tracking cookies, or publisher-operated data store.

The hook reads `HC_TERMINAL_ID` and prints instructions into the current agent
session. It does not send that value to pcodez or make network requests.

The local SQLite database retains interview checkpoints, proposals and evidence outside the project checkout until you remove it. Same-account tools with filesystem access can read it. Only include information appropriate for your agent provider and project wiki.

When you use the skill, your agent can read project files, run local commands,
start other agents, and exchange task context through Herdr. Project context and
terminal output may be processed by your selected AI provider under that
provider's terms and privacy policy. Herdr and the agent CLIs have their own
permissions, storage, networking, and account requirements. This package does not
replace or control those policies.

The package does not ask for API keys or account credentials. Use your own
authenticated agent CLI. Review the plan and requested permissions before
launching a team; use sample data when evaluating the plugin.

For questions, open a [GitHub issue](https://github.com/pcodez99/herdr-skills/issues).
Issues are public: do not include secrets, private code, or personal data.
