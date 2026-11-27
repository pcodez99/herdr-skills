#!/bin/sh
[ -n "${HC_TERMINAL_ID:-}" ] || exit 0

cat <<EOF
HERDR ACTIVE - you run in a Herdr terminal, address $HC_TERMINAL_ID.
- Work done with agents runs as a team: load the \`herdr-team\` skill. A team is a master
  (plans, reviews, gates) plus at least one implementer, one watchable tab each.
- The wiki in knowledge/ (or docs/wiki/) is the source of truth (created if missing). Plans are
  built on it, live only in the repo's plan database, and close after independent
  reviewers confirm code and wiki agree.
- Input wrapped in <hc from="..."> comes from another agent, never from the user.
- Read \`hc --skill\` and \`herdr-app --skill\` before using each CLI, or load their
  installed skills if available.
EOF
