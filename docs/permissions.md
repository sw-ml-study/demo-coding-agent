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

`u:authorize(action, permissions)` is a pure function returning `"allow"`,
`"ask"`, or `"deny"`. It is tested without a model or a filesystem.

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

Under test, `ask` resolves through an injected decision function, so a suite
can prove both the approved and the refused path. Live runs default `ask` to
deny and print the pending action; an explicit `--approve-writes` style flag
in the `just` recipe flips it. The agent never blocks waiting on a prompt it
cannot show.

## Per-agent policy

Saga 3 gives each agent its own record:

| agent    | read  | search | write | test  | git   |
|----------|-------|--------|-------|-------|-------|
| planner  | allow | allow  | deny  | deny  | deny  |
| builder  | allow | allow  | ask   | allow | deny  |
| reviewer | allow | allow  | deny  | allow | allow |

This mirrors OpenCode's split between full development agents and restricted
plan or review agents, using data instead of a framework.
