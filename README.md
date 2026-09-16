# demo-coding-agent

<img src="assets/mlplcode-logo.png" alt="mlplcode badge: Software Wrighter" width="160" align="right">

A coding agent small enough to understand, written in sw-MLPL. The tool is
named **mlplcode**, in the spirit of OpenCode.

[sw-MLPL](https://github.com/sw-ml-study/sw-mlpl) is a small array/functional
language for machine-learning study: whole-array arithmetic, records, function
references, `Result` values, autograd, and native model helpers, with an
interpreter, a compiler, and a browser build. Try it in the
[sw-MLPL playground](https://sw-ml-study.github.io/sw-mlpl/) and browse the
[sw-ml-study demo repositories](https://github.com/orgs/sw-ml-study/repositories?q=demo)
that exercise it: native extensions, ML microscopes, algorithms, data
structures, and more. This repository is the coding-agent demo in that set.

The control loop is sw-MLPL. Rust supplies only filesystem and process
mechanisms that the language cannot express itself. A local LLM supplies
inference through the shipped `llm_call` builtin. The question this repository
answers is not "can MLPL call a model?" but "can MLPL itself express the
control plane of an autonomous software agent?" The target is roughly one LLM
primitive, five OS tools, and one to two hundred lines of MLPL.

[OpenCode](https://github.com/anomalyco/opencode) is the architectural
comparison point. Its conceptual core is a loop:

```text
task -> context -> model -> tool request? -> execute -> observation -> model
```

Everything else in a product-grade agent is engineering layered around that
loop. This repository builds the loop in the open and postpones the layers.

## How it is built

- **Policy in MLPL.** Prompt construction, action parsing, tool dispatch,
  permissions, retry, and stop logic are MLPL data and pure functions.
- **Mechanisms first from builtins.** sw-MLPL already ships sandboxed
  `read_text`, `write_text`, `fs_walk`, and friends, confined to the project
  root. The first agents use no Rust at all.
- **Mechanisms then from a narrow Rust extension.** Allow-listed process
  execution, git diff and status, and search live in `extensions/agent-tools`,
  built on the ABI and SDK pattern proven in
  [`demo-extensions`](../demo-extensions). The model is never handed a shell.
- **Language changes last.** An sw-MLPL request needs an executable probe
  recorded in the [capability ledger](docs/sw-mlpl-capabilities.md).
- **Tests need no model.** Every agent takes its model as an injected
  function reference; mlplunit suites replay scripted transcripts. Live
  Ollama runs are opt-in demos.

The model speaks a tiny text protocol instead of native tool-calling JSON so
the mechanism stays visible:

```text
READ <path>
SEARCH <text>
WRITE <path> ... END
RUN <command>
DONE <summary>
```

## Planned progression

| version | shape                               |
|---------|-------------------------------------|
| v0      | READ, THINK (done)                  |
| v0.5    | action protocol parser (done)       |
| v2      | bounded READ/WRITE loop with allow/ask/deny (done) |
| v1      | SEARCH via the Rust extension       |
| v3      | RUN and repeat until tests pass     |
| v4      | planner, builder, reviewer          |
| v5      | budgets, compaction, loop detection |

See [progression](docs/progression.md) for what is deliberately postponed.
There is no TUI in the plan: once the agent works, the front end is an org
file in Emacs using sw-MLPL's existing org-babel backend.

## Prerequisites

- The adjacent `../sw-mlpl` checkout with `target/release/mlpl-repl` or an
  absolute `MLPL` override (0.22.0 or newer for `llm_call` and the sandboxed
  filesystem builtins).
- `mlplunit` on `PATH`, an absolute `MLPLUNIT` override, or the adjacent
  `../../softwarewrighter/mlplunit` checkout.
- [`just`](https://github.com/casey/just) for repository task aliases.
- For live runs only: an Ollama server with `qwen2.5-coder:7b` pulled. See
  the model section below.
- Rust 1.85 or newer for `extensions/agent-tools`, the ripgrep-backed
  search and allow-listed `cargo`/`git` runner; `just check` skips the Rust
  checks with a notice when `cargo` is absent. `sw-checklist` gates the crate
  when installed.

The scripts select existing tools; they never install or overwrite them.

## Model

Live recipes default to `OLLAMA_HOST=http://localhost:11434` and
`OLLAMA_MODEL=qwen2.5-coder:7b`; override either in the environment.

```sh
ollama pull qwen2.5-coder:7b
```

The 7B coder model is the floor: it fits an RTX 3060 12 GB or a 16 GB
unified-memory Mac with room for context, and it keeps to the one-action
protocol. Every live recipe first loads the model through the Ollama API and
prints how long that took, allowing up to five minutes for a cold start, so
the first real `llm_call` never pays it; each call is then bounded by
`llm_call`'s own 120-second timeout. On smaller machines `qwen2.5-coder:1.5b` runs anywhere but drifts
out of the protocol more often. Bigger cards can point the same variable at
a larger model; `devstral:24b` (14 GB) completed the demo task with no
syntax slips and is the recommended upgrade tier. `just check` never contacts a model server, so a fork
without a GPU still gets a green gate.

An OpenAI-compatible chat endpoint is planned so any hosted model can drive
the same loop through the model-injection seam; see
[progression](docs/progression.md).

## Development process

Work is divided into durable Agentrail steps. In each fresh session run
`agentrail next`, then `agentrail begin`; implement only that step; run the
precommit gate; commit source and `.agentrail/` metadata; then run
`agentrail complete`. [`AGENTS.md`](AGENTS.md) holds the full protocol.

The precommit gate is:

```sh
just check
```

It checks repository structure, peer license parity, documentation links,
the generated Agentrail briefing, canonical MLPL style, and the mlplunit
suites. Rust crates, once present, add `sw-checklist` and scoped
`cargo test`.

Read the [architecture](docs/architecture.md), [delivery plan](docs/plan.md),
[saga queue](docs/sagas.md), [permissions](docs/permissions.md), and
[capability ledger](docs/sw-mlpl-capabilities.md) before implementing an
agent. The original design discussion is retained in
[`docs/research.txt`](docs/research.txt).

## Run the v0 agent

```sh
just v0
```

v0 is the whole loop with one tool and no iteration: read one file with
`read_text`, build a `TASK:` / `SOURCE:` prompt, ask the model once, print
the answer. It lives in [`agents/v0_read_think.mlpl`](agents/v0_read_think.mlpl)
as five small functions and [`agents/run_v0.mlpl`](agents/run_v0.mlpl) as the
live entry. The system prompt is the plain file
[`prompts/think.md`](prompts/think.md). `V0_FILE` and `V0_TASK` override the
file and the task.

The model is a function reference. `call(:u:ask_live, host, model, system)`
binds the server details into a one-argument partial; tests bind
`u:ask_scripted` to a fixed reply or pass `u:ask_echo` to inspect the exact
prompt. Against `qwen2.5-coder:7b` the fixture crate yields an explanation of
`add` and a suggested negative-number test.

## Run the bounded loop

```sh
just loop                  # at a terminal: the agent asks before each write
LOOP_APPROVE=1 just loop   # every write approved, for unattended runs
just loop < /dev/null      # no terminal: writes are refused, a dry run
```

Before each write the agent prints what it wants to write and waits for
`y`. Anything else refuses, and a refusal ends the run with a message that
says which action was refused and how to allow it. Give it your own task
with `LOOP_TASK="..."`, raise the step limit with `LOOP_BUDGET`, and gate
DONE on a passing test run with `LOOP_VERIFY=tests`.

[`agents/loop.mlpl`](agents/loop.mlpl) is the whole agent as data plus pure
functions. One state record `{task, iteration, history, files, budget, done,
reason, answer}` flows through build context, ask model, parse action,
authorize, execute, update. The model reply is parsed by
[`agents/protocol.mlpl`](agents/protocol.mlpl); the permission record
`{read, search, write, run}` maps each tool to `allow`, `ask`, or `deny`; an
`ask` resolves through an injected decision function, so an unattended run
never writes. The loop stops with a reason value: `done`, `denied`, or
`budget`. A malformed reply or a failed builtin becomes an observation the
model sees on its next turn. `RUN mlpl <path>` runs an MLPL test file
through `run_script` and observes its status and per-test results; with the
Rust extension built, `SEARCH` is ripgrep over the project and `RUN cargo
...` and `RUN git ...` execute allow-listed subcommands only, git under its
own `ask` permission. Anything else is shell and is denied without
executing. `LOOP_TASK` and `LOOP_BUDGET` override
the task and the step limit. The system prompt is
[`prompts/act.md`](prompts/act.md).

Tests drive the loop with a scripted transcript model that picks its reply
by counting prior actions in the prompt, so every stop reason, the ask
decision, parse-error recovery, and read errors are proven offline.

## Watch it code

![mlplcode adding u:mul and a passing test to the MLPL example, recorded with VHS](assets/demo/loop.gif)

*Six steps: read, write, read, write, run, done. The model replies are
replayed from the saved live `qwen2.5-coder:7b` transcript; the parser, the
file writes, and the test run happen for real. Recorded with
[VHS](https://github.com/charmbracelet/vhs) from `demos/loop.tape`;
`just mlpl-demo` runs the same task live, saves the transcript under
`out/transcripts/`, and restores the example even on Ctrl-C; `SAVE_FIXTURE=1`
replaces the committed transcript after a successful run. `just replay`
plays it in any terminal without a model server (faster than the GIF, which is slowed to reading pace) and `just
demo-tape` re-records it (an MP4 is produced alongside but not committed).
`demos/loop-live.tape` records a fresh live run instead.*

```sh
just mlpl-demo
```

The agent adds `u:mul` and a test to `examples/tiny-mlpl-project`, runs the
tests with `RUN mlpl ...`, and finishes only when a RUN observation shows
every test passed. With `qwen2.5-coder:7b` this takes six steps: read, write,
read, write, run, done. The transcript is saved under
`fixtures/transcripts/` and the example project is restored afterwards.
DONE is gated by an injected verify function, so a model that claims success
without evidence is told `NOT VERIFIED` and keeps working. The six live
attempts it took to get here, and what each one changed, are in
[progression](docs/progression.md).

## Tests and probes

```sh
just tests        # 73 native mlplunit tests, no model server needed
just replay-check # the committed transcript replays to a verified done
just llm-probe    # opt-in: one llm_call round trip against local Ollama
```

The mlplunit suites under `tests/` pin the measured behavior of every builtin
the agent depends on, the protocol parser, and the loop. The agent's first
coding target is `examples/tiny-mlpl-project`: a one-function `lib.mlpl`
and a `tests/test_add.mlpl` that runs under sw-MLPL's `run_script` with no
runner, because the mlplunit assertion library is vendored beside it with
its pin in `vendor/VENDOR.md`. `examples/tiny-rust-project` is the later
target for the Rust extension path.

## Current status

The agent codes. Saga 1 built the loop, the parser, and permissions; Saga 2
has so far added the MLPL example project, `RUN mlpl` through `run_script`,
verified completion, and a live run in which `qwen2.5-coder:7b` added a
function and a passing test, all without Rust. Next is the VHS recording of
that run, then the Rust `agent-tools` extension for ripgrep search,
allow-listed `cargo` and `git`, and the Rust example. The
[capability ledger](docs/sw-mlpl-capabilities.md) records measured rows and
eight findings for sw-MLPL.

Copyright (c) 2026 Michael A Wright. Distributed under the [MIT License](LICENSE).
