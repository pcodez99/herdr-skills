# Troubleshooting

| Symptom                                            | Cause and fix                                                                                                              |
| -------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Agent sits on a trust or permission dialog         | `hc read <t>`, then `hc keys <t> enter` (or `down enter`).                                                                 |
| Agent stays silent                                 | `hc get <t>` for its status, `hc read <t>` for its screen. Read before sending again: the first message may have arrived.  |
| `herdr-app do …` rejects an argument               | Names change between releases: `herdr-app commands`. The JSON form avoids quoting problems.                                |
| zsh prints `parse error` after `hc send <t> …`     | The tab hosted an agent before, so the text arrives wrapped in `<hc from=…>` tags. Open a fresh tab with `tab.new --argv`. |
| `command not found` in an `hc run` terminal        | A terminal started with a command skips the login profile. Wrap it: `hc run "zsh -lc '<command>'"`.                        |
| A whole test suite fails while another agent tests | A collision on the shared tree or test database. Rerun when the other agent is idle.                                       |
| Checks fail in a perimeter you do not own          | Usually another agent's step in progress. Check the status table and log, then tell the master.                            |
| You lost track of the team                         | The team table in the master plan, then `hc ls` and `herdr-app ls`.                                                        |
