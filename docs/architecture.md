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

State is one record, measured in `agents/loop.mlpl`:

```text
{
    task: "...",
    iteration: 4,
    history: "...ACTION:...OBSERVATION:...",
    files: "src/lib.rs\nnotes.txt\n",
    budget: 12,
    done: 0,
    reason: "",
    answer: ""
}
```

`files` is a newline-joined string rather than a list because string lists
cannot be appended to (ledger F8). The loop repeats a bounded number of
times and stops with `reason` set to `done`, `denied`, or `budget`. Every
stop reason is a value, not an exception; a malformed reply or a failed
builtin becomes a `PARSE ERROR:` or `ERROR:` observation for the next turn.

Two guards run before authorization, both pure functions over the state:
a WRITE to an existing file that this run has not READ is refused with
`read <path> before writing it` (a new file needs no read); a reply
identical to the previous one is not executed but observed as
`REPEATED ACTION`, and a third identical reply stops the run with reason
`stuck`. The state record carries `last_reply` and `repeats` for this.

DONE is gated. The loop takes an injected `verify(state)` function next to
`decide`: `u:verify_always` accepts any DONE; `u:verify_tests_passed`
requires at least one successful WRITE and, after the last WRITE, a RUN
observation with `status: ok` and no `failed:` line. A DONE without that
evidence becomes a `NOT VERIFIED: <reason>` observation and the loop
continues. This came from live runs in which the model fabricated
observations and declared success; see the progression doc.

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
readable. `agents/protocol.mlpl` turns one reply into `ok({tool: "read",
path})`, `ok({tool: "search", text})`, `ok({tool: "write", path, body})`,
`ok({tool: "run", argv})`, `ok({tool: "done", summary})`, or an `err`
naming the reason: empty reply, unknown verb, missing END, more than one
action, empty path, `..` component, absolute path. Verbs are whole words;
CRLF is normalized; the body keeps its inner newlines. Model habits seen in
live transcripts and now tolerated, each pinned by a test: a fence-only line
(three backticks, optionally with a language word) directly after
`WRITE <path>` and directly before `END` is dropped; fences inside the body
are content; blank lines before an opening fence and a fence-only line after
`END` are also dropped; a leading `ACTION:` header line is dropped; everything from a fabricated `OBSERVATION:` line
onward is ignored once the action is complete. Habits seen and deliberately
not tolerated: a body without `END`, `DONE` on the line after a body, `END`
sent as its own turn, and `READ <path> END`; the system prompt in
`prompts/act.md` forbids them instead, using `<path>` placeholders because a
concrete example path was copied verbatim by the 7B model. Later sagas may add
`PATCH` with exact old/new text. This is the place where tagged sum values
would help MLPL; the records-plus-`Result` encoding is recorded as awkward,
not blocking, in the capability ledger.

## Model injection

`llm_call` needs a running server, so no test calls it directly. Agents take
the model as a one-argument callable from prompt to reply. Measured shape in
v0: `call(:u:ask_live, host, model, system)` returns a partial (data, not a
closure) that the agent invokes with `call(ask, prompt)`; tests bind
`call(:u:ask_scripted, reply)` for a fixed answer or pass `:u:ask_echo` to
read back the exact prompt. A scripted model replays a fixed transcript of
replies, which makes every loop test deterministic and offline. Live runs are `just` recipes that read
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
examples/tiny-mlpl-project/  the first project the agent edits: lib.mlpl, tests/, vendored mlplunit
examples/tiny-rust-project/  the Rust project for the extension path
extensions/agent-tools/  Rust cdylib + extension.toml + module.mlpl (Saga 2)
tests/                   native mlplunit suites with scripted fake models
fixtures/                recorded transcripts and expected observations
scripts/                 thin portable checks; justfile is the entry point
docs/                    this design, plan, permissions, progression, ledger
```
