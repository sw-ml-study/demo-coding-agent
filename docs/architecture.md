# Architecture

## Governing principle

MLPL owns policy. Rust owns mechanisms. A local model owns inference.

The agent is not a port of OpenCode. It is the smallest loop that turns a
task into observations, decisions, and file changes, written so that every
decision is visible as MLPL data flowing through pure functions. Anything the
model cannot see in a transcript should not exist.

## Layering

```text
                 MLPL (agents/*.mlpl)
        +----------------------------------+
        | agent loop                       |
task -->| prompt / context construction    |
        | action protocol parser           |
        | permission policy (allow/ask/deny)|
        | retry / budget / stop logic       |
        +----------------+-----------------+
                         |
              +----------+-----------+
              |                      |
   sw-MLPL builtins          Rust agent-tools extension
   read_text  fs_walk        search (ripgrep crates)
   write_text write_atomic   run (argv allow-list)
   run_script (MLPL only)    git_diff  git_status
              |                      |
              +----------+-----------+
                         |
                     project root
                    (--source-dir)

                    llm_call(url, prompt, model, system)
                         |
                   Ollama / llama.cpp
```

The preference order for any capability is: an existing sw-MLPL builtin,
then a function in this repository's Rust extension, then an sw-MLPL change
justified by an executable probe. Saga 1 uses no Rust at all. Search is an
extension concern from the start: the ripgrep crates give `.gitignore`
awareness and binary handling that a `fs_walk` loop would have to reinvent.

## The loop as a pipeline

An agent step is a function from state to state:

```text
state
  |> build_context
  |> ask_model        (injected function reference; fake under test)
  |> parse_action     ("READ src/lib.rs" -> {tool: "read", path: "src/lib.rs"})
  |> authorize        (action + policy -> allow | ask | deny)
  |> execute          (builtin or extension call -> observation)
  |> update_state
```

State is one record:

```text
{
    task: "...",
    iteration: 4,
    history: "...",
    files: ["src/lib.rs"],
    tests_passed: 0,
    budget: 12
}
```

The loop repeats a bounded number of times and stops on `DONE`, on a denied
action, or when the budget is spent. Every stop reason is a value, not an
exception.

## The action protocol

The model answers with exactly one action in plain text:

```text
READ <path>
SEARCH <text>
WRITE <path>
<complete file contents>
END
RUN <command>
DONE <summary>
```

Native tool-calling JSON is deliberately avoided so the mechanism stays
readable. The parser turns each line into a record or an `err(...)`. Later
sagas may add `PATCH` with exact old/new text. This is the place where tagged
sum values would help MLPL; the records-plus-`Result` encoding is recorded as
awkward, not blocking.

## Model injection

`llm_call` needs a running server, so no test calls it directly. Agents take
the model as a function reference (`:u:ask_live` or `:u:ask_scripted`). The
scripted model replays a fixed transcript of replies, which makes every loop
test deterministic and offline. Live runs are `just` recipes that read
`OLLAMA_HOST` and `OLLAMA_MODEL`, defaulting to `http://localhost:11434` and
`qwen2.5-coder:7b`, and pass them to `llm_call` explicitly; there are no
implicit lookups inside the agent. A later provider function for an
OpenAI-compatible chat endpoint will plug into the same injection point, so
the loop never knows which server answered.

## Safety boundary

The sandbox root is the project under edit, passed as `--source-dir`. All
builtin file access is confined to it by sw-MLPL. The Rust extension confines
process execution to a fixed argv allow-list and the same root. The model is
never given a shell. See [permissions](permissions.md).

## Repository layout

```text
agents/                  MLPL agents: v0 read/think, loop, planner/builder/reviewer
prompts/                 system prompts as plain text files
examples/tiny-rust-project/  the project the agent edits under test
extensions/agent-tools/  Rust cdylib + extension.toml + module.mlpl (Saga 2)
tests/                   native mlplunit suites with scripted fake models
fixtures/                recorded transcripts and expected observations
scripts/                 thin portable checks; justfile is the entry point
docs/                    this design, plan, permissions, progression, ledger
```
