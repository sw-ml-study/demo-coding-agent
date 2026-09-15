# Progression

The agent grows in visible steps. Each version is a separate MLPL file so a
reader can diff the loop as it gains capability.

| version | shape                                     | tools                       | saga |
|---------|-------------------------------------------|-----------------------------|------|
| v0      | READ, THINK (done)                        | `read_text`                 | 1    |
| v2      | bounded READ, WRITE loop, allow/ask/deny (done) | `write_atomic`, history, stop reasons | 1 |
| v1      | SEARCH                                    | ripgrep-backed `search` via extension | 2 |
| v3      | RUN, repeat until tests pass              | allow-listed `run` via extension | 2 |
| v4      | planner, builder, reviewer                | per-agent permissions       | 3    |
| v5      | budgets, compaction, loop detection       | pure policy functions       | 4    |
| v6      | git diff and status, exact patch          | extension                   | 2, 4 |
| v7      | driven from an org file in Emacs          | sw-MLPL `ob-mlpl.el`, no TUI | 5   |

Deliberately postponed, in OpenCode's terms: TUI, MCP, streaming, subagent
concurrency, LSP, GitHub integration, session persistence, embeddings or RAG,
automatic context compaction, arbitrary shell access. Each would obscure the
experiment more than it would teach. One provider addition is planned: an
OpenAI-compatible chat endpoint so any hosted model can drive the same loop
through the existing model-injection seam. A TUI is replaced by Emacs:
sw-MLPL's org-babel backend already runs `#+begin_src mlpl` blocks, which is
a lighter path to an interactive front end than a terminal UI.

## Demo recording

The agent is a plain CLI (`just loop`), so a
[VHS](https://github.com/charmbracelet/vhs) tape can record a run and export
GIF, WebP, or MP4 for the README. That is planned for the end of Saga 2,
when `RUN cargo test` works and the recording shows the agent adding a test
and making it pass rather than only reading and writing.

## The smallest possible first proof

```text
task   = "Explain the most likely bug in src/lib.rs."
source = unwrap(read_text("src/lib.rs"))
prompt = "TASK:\n" + task + "\n\nSOURCE:\n" + source
answer = llm_call(HOST, prompt, MODEL, "You are a careful Rust programmer.")
disp(answer)
```

That is already a coding agent with one tool. Everything after it is
iteration policy.
