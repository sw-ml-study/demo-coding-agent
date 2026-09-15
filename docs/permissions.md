# Permissions

OpenCode is not sandboxed; its permission system is an interaction and
awareness layer around powerful shell and filesystem access. This repository
makes the model more constrained than OpenCode, on purpose.

## Two layers

1. **Mechanism confinement (Rust and sw-MLPL).** Every filesystem builtin is
   confined to the `--source-dir` root and never follows symlinks. The Rust
   `agent-tools` extension accepts only argv arrays that match a fixed
   allow-list and runs them in the same root. There is no primitive that runs
   a model-generated string as a shell command.
2. **Policy (MLPL data).** Which permitted mechanism the agent may actually
   use is a record the program passes in:

```text
permissions = {
    read:   "allow",
    search: "allow",
    write:  "ask",
    test:   "allow",
    git:    "ask",
    shell:  "deny"
}
```

`u:authorize(action, permissions)` in `agents/loop.mlpl` is a pure function
returning `"allow"`, `"ask"`, or `"deny"`, tested without a model or a
filesystem. The measured record has one field per tool: `{read, search,
write, run}`; `done` is always allowed. `git` and `shell` fields arrive with
the extension in Saga 2.

## Initial allow-list for `RUN`

```text
cargo test *
cargo check *
cargo clippy *
cargo fmt *
git diff *
git status *
```

The allow-list is matched on the argv prefix after splitting on whitespace,
never by handing the line to `/bin/sh -c`. Anything else is a deny, and the
denial is recorded as an observation so the model can choose differently.

## `ask` in a non-interactive run

`ask` resolves through an injected decision function `decide(action)`
returning 1 or 0. Tests pass `:u:decide_yes` or `:u:decide_no` to prove both
paths. Live runs default to `decide_no`, so `just loop` is a dry run that
stops with reason `denied` and prints the refused action; `LOOP_APPROVE=1`
switches to `decide_yes`. The agent never blocks waiting on a prompt it
cannot show. A denial ends the run rather than continuing, so the model
cannot probe the policy by retrying.

## Per-agent policy

Saga 3 gives each agent its own record:

| agent    | read  | search | write | test  | git   |
|----------|-------|--------|-------|-------|-------|
| planner  | allow | allow  | deny  | deny  | deny  |
| builder  | allow | allow  | ask   | allow | deny  |
| reviewer | allow | allow  | deny  | allow | allow |

This mirrors OpenCode's split between full development agents and restricted
plan or review agents, using data instead of a framework.
