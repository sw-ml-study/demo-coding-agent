# Coding agent delivery plan

## Outcome

Build a coding agent small enough to understand. The control loop, prompt
construction, action parsing, tool dispatch policy, permissions, and stop logic
are written in sw-MLPL. A local LLM supplies inference through the shipped
`llm_call` builtin. Filesystem and process mechanisms come first from the
sandboxed builtins sw-MLPL already ships and then from a narrowly scoped Rust
`agent-tools` extension that lives in this repository.

The README proposition is: how little machinery is required to turn an
array/functional language into a coding agent? The target answer is one LLM
primitive, roughly five OS tools, and one to two hundred lines of MLPL.
OpenCode is the architectural comparison point, not the implementation target.

## Evidence and current constraints

- `llm_call(url, prompt, model[, system])` is a shipped language builtin
  (sw-MLPL 0.22.0). It returns the model reply as an ordinary string. There is
  no streaming, native tool calling, or chat threading, and that is fine: the
  agent uses a small text action protocol instead.
- sw-MLPL already ships sandboxed, Result-speaking filesystem builtins:
  `read_text`, `write_text`, `write_atomic`, `fs_walk`, `file_metadata`, and
  `remove_path`, all confined to the `--source-dir` root with symlinks never
  followed. Reading, listing, and writing project files need no Rust.
- `run_script` executes an MLPL file in a fresh environment and returns its
  outcome as data. It runs MLPL only; it cannot run `cargo test` or `git`.
- sw-MLPL has no search-in-files, no allow-listed process execution, and no
  git access. These are the mechanisms a Rust extension must supply. Search
  is expressible as `fs_walk` plus `read_text` plus `str_find`, but a
  ripgrep-backed extension respects `.gitignore`, handles binaries, and
  scales; it is the planned path rather than a fallback.
- `../demo-extensions` proves the extension path this repository will reuse:
  a Rust `cdylib` exporting one `sw_mlpl_extension_v1` symbol, a versioned C
  ABI, a safe SDK, an `extension.toml` package manifest, a private
  `_namespace`, a public `module.mlpl` facade, and stock-CLI `load_extension`.
- Records, function references, `Result`, string builtins (`str_find`,
  `str_slice`, `str_split`, `str_join`, `str_eq`), loops, and `include` are
  sufficient for a prefix-based action protocol and a functional state
  pipeline. Tagged sum types would be nicer; their absence is awkward, not
  blocking.
- Native mlplunit tests need no model server. Every agent behavior is tested
  against a scripted fake model injected as a function reference. Live Ollama
  runs are opt-in demos, never the only acceptance evidence.

## Model defaults

Live runs default to `OLLAMA_HOST=http://localhost:11434` and
`OLLAMA_MODEL=qwen2.5-coder:7b`, both overridable by environment variable.
That model is the floor because it fits an RTX 3060 12 GB or a 16 GB
unified-memory Mac with room for an 8K context, and it follows a strict
one-action system prompt reliably. `qwen2.5-coder:1.5b` is the documented
"runs anywhere, expect protocol drift" fallback; larger variants are opt-in
through the same variable. The live-run script checks the server's tags
endpoint first and prints the exact `ollama pull` command when the model is
absent, rather than failing inside `llm_call`. `just check` never contacts a
model server, so forks without a GPU still get a green gate. Recorded
transcript fixtures name the model that produced them.

This section supersedes the "runs live only when `OLLAMA_HOST` and
`OLLAMA_MODEL` are set" wording in the step 003 prompt: the variables are
optional with those defaults.

Later, an OpenAI-compatible chat endpoint (`OPENAI_BASE_URL`,
`OPENAI_API_KEY`, `OPENAI_MODEL`) lets any hosted model drive the same loop.
Whether `llm_call` already speaks that wire format, or the request must go
through the extension, is measured in step 002 and recorded in the ledger.

## Architecture rule

```text
demo-coding-agent
  agents/*.mlpl          control loop, prompts, parsing, policy, state
  extensions/agent-tools narrow Rust mechanisms: search, run, git, patch
          |
          +-- reuse ABI/SDK/loader pattern ----> demo-extensions (read-only)
          |
          +-- reusable domain-neutral helpers -> demo-mlpl-libraries (later)
          |
          `-- proven language-wide blocker ----> sw-mlpl (last resort)
```

Escalation order for any missing primitive:

1. Use an existing sw-MLPL builtin if one fits.
2. Add a function to this repository's Rust `agent-tools` extension using the
   public ABI and SDK that `../demo-extensions` documents for external crates.
3. Request an sw-MLPL change only after an executable probe proves that
   neither a builtin nor an extension can express the behavior. Record the
   probe in `docs/sw-mlpl-capabilities.md` before asking.

MLPL owns policy: what to read, what to ask, which action to take, whether an
action is permitted, when to stop. Rust owns mechanisms: bytes in and out of
the operating system. Rust never decides which tool the agent should use, and
no MLPL program ever hands the model a raw shell.

## Saga 1: foundation and the smallest agent

1. Establish the repository foundation: Agentrail briefing and project rules in
   `AGENTS.md`, peer copyright and MIT license, README, architecture,
   permissions, progression, capability ledger, cross-repository handoffs, and
   a documentation-only `just check` gate.
2. Add root mlplunit and tool-selection infrastructure and probe the current
   public builtins an agent depends on: `llm_call` argument contract,
   `read_text`/`fs_walk`/`write_text` sandbox behavior and error shapes,
   `run_script` outcome records, and the string helpers needed for a prefix
   protocol. Freeze the measured results in the capability ledger.
3. Implement v0: one tool, one thought. `agents/v0_read_think.mlpl` reads a
   file from `examples/tiny-rust-project`, builds a prompt, asks the model, and
   prints the answer. The model call is injected so mlplunit proves the prompt
   and the flow with a scripted fake; a `just v0` recipe runs it live against
   Ollama using the model defaults above.
4. Implement the action protocol parser: `READ`, `SEARCH`, `WRITE ... END`,
   `RUN`, and `DONE` lines become records such as `{tool: "read", path: ...}`
   or `err(...)`. Malformed, multi-action, and out-of-sandbox inputs are tested
   before any dispatch exists.
5. Implement v2/v3: the bounded observe, decide, act, update loop with history
   accumulation, read and write dispatch over the sandboxed builtins, a hard
   iteration limit, and `DONE` detection. `SEARCH` and `RUN` return a
   "not available yet" observation until Saga 2 supplies the extension. A
   scripted multi-turn fake model proves the loop deterministically; a live
   run is documented as a demo.

Exit: `just check` passes from a clean checkout; the agent reads and edits a
file in the example project under test with no Rust code; the capability
ledger distinguishes supported, awkward, and blocked behavior with evidence.

## Saga 2: mechanisms and permissions

1. Add `extensions/agent-tools` as a Rust `cdylib` on the demo-extensions ABI
   and SDK, proven by `sw-checklist`, scoped `cargo test`, and a stock-CLI
   `load_extension` check. First functions: `_agent_tools:search(pattern)`
   over the project root, built on the ripgrep crates (`grep-searcher`,
   `grep-regex`, `ignore`) with an `rg` subprocess as the documented
   fallback, and `_agent_tools:run(argv)` against a fixed allow-list
   (`cargo test`, `cargo check`, `cargo clippy`, `cargo fmt`, `git diff`,
   `git status`). The public `agent_tools` facade is MLPL.
2. Express permissions in MLPL as data: `{read: "allow", search: "allow",
   write: "ask", test: "allow", git: "ask", shell: "deny"}`. Authorization is
   a pure function from action plus policy to allow, ask, or deny, tested
   without a model or a filesystem.
3. Close the loop: `SEARCH` and `RUN cargo test` reach the extension, the agent
   iterates until tests pass or the budget is spent, and the example Rust
   project gains a real unit test through a recorded live run.
4. Add `git_diff`, `git_status`, and an exact old/new `patch` mechanism so the
   model edits regions instead of rewriting whole files.

Exit: the agent completes "add a unit test and make it pass" against the
example project with every OS effect passing through a permission decision.

## Saga 3: planner, builder, reviewer

1. Split the single loop into `u:planner` (read-only tools), `u:builder`
   (read, search, write, patch), and `u:reviewer` (diff and test only). They
   share one model and differ only by system prompt and permission set.
2. Make agent state one record (`task`, `iteration`, `files`, `observations`,
   `tests_passed`, `budget`) flowing through a pipeline of pure functions:
   build context, ask model, parse action, authorize, execute, update state.
3. Record and compare transcripts of the single-agent and split-agent runs on
   the same task.

Exit: the split agents solve the Saga 2 task with fewer or equal iterations,
and the reviewer never writes.

## Saga 4: iteration policy

Add, each as a tested pure function over state: token and context budget,
bounded history compaction, repeated-action loop detection, retry budget for
malformed model output, and cancellation. Document which OpenCode features are
deliberately still absent and why.

Exit: the agent fails loudly and cheaply on a task it cannot finish.

## Saga 5: reflection and reuse

1. Write the OpenCode comparison: what their loop, permissions, and agent
   modes look like against this repository's data-plus-functions version.
2. Extract domain-neutral helpers (action parsing, policy evaluation, bounded
   history) into a handoff for `demo-mlpl-libraries` once three agents use them.
3. Record sw-MLPL findings, especially where tagged sum values or `match`
   would have simplified the dispatch, as classified capability notes.

Exit: another MLPL repository can reuse the agent core without copying it.

## Cross-cutting gates

- Every executable behavior starts with native mlplunit coverage driven by a
  scripted fake model; `just check` is the full precommit gate.
- Every `.mlpl` file has a module comment, every user function has a
  first-body docstring, and canonical formatting is checked before commit.
- `sw-checklist` and scoped `cargo test` gate every Rust crate under
  `extensions/`; they are not a gate for MLPL-only steps.
- No tool ever executes a model-generated string as a shell command.
- Every write and every process execution is confined to the project root.
- Documentation, permissions, progression, capability ledger, and handoffs
  change with the behavior they describe.
- Sibling repositories remain read-only from this project.

## Non-goals

- A TUI, MCP, streaming, session persistence, embeddings or RAG, LSP, GitHub
  integration, or multi-agent concurrency. Multiple providers are postponed,
  not excluded: an OpenAI-compatible endpoint is the one planned addition.
- A generic `shell()` builtin or arbitrary command execution.
- Automatic context compaction before a measured budget problem exists.
- Reproducing OpenCode's implementation or its plugin surface.
- Modifying any sibling repository from this project.
