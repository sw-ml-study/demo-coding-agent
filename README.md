# demo-coding-agent

A coding agent small enough to understand.

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
| v0      | READ, THINK                         |
| v1      | READ, SEARCH, THINK                 |
| v2      | READ, SEARCH, EDIT, TEST            |
| v3      | repeat until tests pass             |
| v4      | planner, builder, reviewer          |
| v5      | budgets, compaction, loop detection |

See [progression](docs/progression.md) for what is deliberately postponed.

## Prerequisites

- The adjacent `../sw-mlpl` checkout with `target/release/mlpl-repl` or an
  absolute `MLPL` override (0.22.0 or newer for `llm_call` and the sandboxed
  filesystem builtins).
- `mlplunit` on `PATH`, an absolute `MLPLUNIT` override, or the adjacent
  `../../softwarewrighter/mlplunit` checkout.
- [`just`](https://github.com/casey/just) for repository task aliases.
- For live runs only: an Ollama or llama.cpp server and a coding model such
  as `qwen2.5-coder`, selected by explicit environment variables.
- Rust 1.85 or newer and `sw-checklist`, only once `extensions/agent-tools`
  exists.

The scripts select existing tools; they never install or overwrite them.

## Development process

Work is divided into durable Agentrail steps. In each fresh session run
`agentrail next`, then `agentrail begin`; implement only that step; run the
precommit gate; commit source and `.agentrail/` metadata; then run
`agentrail complete`. [`AGENTS.md`](AGENTS.md) holds the full protocol.

The precommit gate is:

```sh
just check
```

Today it checks repository structure, peer license parity, documentation
links, the generated Agentrail briefing, and canonical MLPL style. Each step
that adds executable behavior extends it with mlplunit suites and, for Rust
crates, `sw-checklist` and scoped `cargo test`.

Read the [architecture](docs/architecture.md), [delivery plan](docs/plan.md),
[saga queue](docs/sagas.md), [permissions](docs/permissions.md), and
[capability ledger](docs/sw-mlpl-capabilities.md) before implementing an
agent. The original design discussion is retained in
[`docs/research.txt`](docs/research.txt).

## Current status

Foundation only. No agent runs yet. The next step measures the builtins an
agent depends on and adds mlplunit infrastructure.

Copyright (c) 2026 Michael A Wright. Distributed under the [MIT License](LICENSE).
