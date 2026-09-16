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
filesystem, row by row in `tests/test_loop.mlpl`. The record has six fields
and RUN is classified by its first word:

| action                 | field    | live default | mechanism |
|------------------------|----------|--------------|-----------|
| READ                   | `read`   | allow        | `read_text` |
| SEARCH                 | `search` | allow        | `_agent_tools:search` (ripgrep crates) |
| WRITE                  | `write`  | ask          | `write_atomic` |
| PATCH                  | `write`  | ask          | `read_text` + `write_atomic`, OLD must match exactly once |
| RUN mlpl `<path>`      | `run`    | allow        | `run_script`, pure MLPL |
| RUN cargo `...`        | `run`    | allow        | `_agent_tools:run`, allow-listed subcommands only |
| RUN git `...`          | `git`    | ask          | `_agent_tools:run`, `diff` and `status` only |
| RUN anything else      | `shell`  | deny         | never executes; `shell: "allow"` still refuses |
| DONE                   |          | allow        | gated by the verify function |

Without the extension loaded, SEARCH and RUN cargo/git observe that the
extension is not loaded instead of failing. `scripts/run-loop` loads it
whenever `target/debug/libmlpl_extension_agent_tools.*` exists.

## Allow-list for `RUN`

Today, in pure MLPL (`agents/tools.mlpl`):

```text
mlpl <relative path>      run_script(path, {source_dir: ".", capture: 1})
```

`u:run_allowed(argv)` accepts exactly two words, the first `mlpl`, the second
a path that passes the same validation as READ and WRITE. The observation
is the child's status, its final value, any error, and one line per
finished test parsed from the captured events (`passed: name`,
`failed: name -- diagnostic`). A failing test file reports `status: err`.

Through the Rust extension, enforced again in Rust before anything starts:

```text
cargo test *   cargo check *   cargo clippy *   cargo fmt *
git diff *     git status *
```

The allow-list is matched on the argv words after splitting on whitespace,
never by handing the line to `/bin/sh -c`. Anything else observes
`run: command not allowed: <command>` and nothing executes, so the model can
choose differently.

## `ask` in a non-interactive run

`ask` resolves through an injected decision function `decide(action)`
returning 1 or 0. Tests pass `:u:decide_yes` or `:u:decide_no` to prove both
paths. Live runs choose in `scripts/run-loop`: `LOOP_APPROVE=1` gives
`decide_yes`; a terminal on stdin gives `decide_prompt`, which prints the
proposed action (for WRITE, the whole body) and approves only `y` or `yes`;
no terminal gives `decide_no`, so a piped or scripted run never writes.

`decide_prompt` cannot read the terminal itself: every sw-MLPL stdin builtin
refuses a TTY (ledger F9). The wrapper therefore feeds the agent's stdin
from a FIFO filled by `cat /dev/tty`, so MLPL reads a pipe while the lines
still come from the keyboard. `scripts/check-decide-prompt` proves y, yes,
n, empty, and EOF through pipes in the gate. A denial ends the run rather
than continuing, so the model cannot probe the policy by retrying, and the
stop message names the refused action and how to allow it.

## Per-agent policy

Saga 3 gives each agent its own record:

| agent    | read  | search | write | test  | git   |
|----------|-------|--------|-------|-------|-------|
| planner  | allow | allow  | deny  | deny  | deny  |
| builder  | allow | allow  | ask   | allow | deny  |
| reviewer | allow | allow  | deny  | allow | allow |

This mirrors OpenCode's split between full development agents and restricted
plan or review agents, using data instead of a framework.
