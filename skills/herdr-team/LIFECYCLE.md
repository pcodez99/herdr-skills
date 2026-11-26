# Spaces and tabs

`herdr-app commands` lists every command with its arguments; `herdr-app do <command>`
takes `--key value` pairs or one JSON object.

## Inspect

```sh
herdr-app ls        # spaces > tabs > terminals, with every ID
herdr-app status    # what the person is looking at now
```

Your space is the one whose tabs contain `$HC_TERMINAL_ID`. Note the tab the person is
looking at, so you can return them to it.

A terminal has an address (`t7`). Tabs (`tb…`), spaces (`sp…`) and panes (`p…`) have
window IDs. Terminals outlive the window; closing a tab or a space ends its terminals.

## Reuse or create

| The space has…                                   | Do                                                     |
| ------------------------------------------------ | ------------------------------------------------------ |
| a tab with the agent's name, at an idle shell    | reuse it: `hc send <t> '<launch command>'`             |
| a tab with that name, busy                       | tell the user and wait for their choice                |
| no such tab                                      | `tab.new` with `--argv`, as in the skill               |
| no space showing your terminal, or the user asks | `herdr-app do space.new --name <task> --folder <repo>` |

`tab.new` prints the new `pane` and `terminal`. The tab ID comes from `herdr-app ls`
afterwards. A program with no prompt argument takes its first message with `--task`;
`hc get <t>` then shows `first_message.sent`.

Opening a tab can move the person's view. Once the team is up, return them:
`herdr-app do tab.select --id <the tab they were on>`.

## Arrange

`tab.rename`, `tab.move` (order the team), `tab.select` or `agent.focus --terminal <t>`
(show one), `tab.settle` (put a finished tab away while it keeps running).

## Close

Close a tab once its agent is idle and its work is committed or handed over:

```sh
hc get <t>
herdr-app do tab.close '{"id":"<tab id>","confirmed":true}'
```

Close the tabs you created, and a space you created once the user agrees. A tab the user
prepared goes back to a shell. Hidden helpers end with `hc kill <t>; hc rm <t>`. The app,
the core (`hc server`) and other people's tabs and spaces stay running: they hold the
rest of the user's work.
