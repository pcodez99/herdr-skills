#!/bin/sh
# SessionStart hook: stdout becomes session context. Silent outside Herdr.
[ -n "${HC_TERMINAL_ID:-}" ] || exit 0

cat <<EOF
HERDR ACTIVE - you run in a Herdr terminal, address $HC_TERMINAL_ID.
- Work done with agents runs as a team: load the \`herdr-team\` skill. A team is a master
  (plans, reviews, gates) plus at least one implementer, one watchable tab each.
- Input wrapped in <hc from="..."> comes from another agent, never from the user.
- Read \`hc --skill\` and \`herdr-app --skill\` before using each CLI, or load their
  installed skills if available.
EOF
