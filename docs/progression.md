# Progression

The agent grows in visible steps. Each version is a separate MLPL file so a
reader can diff the loop as it gains capability.

| version | shape                                     | tools                       | saga |
|---------|-------------------------------------------|-----------------------------|------|
| v0      | READ, THINK                               | `read_text`                 | 1    |
| v1      | READ, SEARCH, THINK                       | plus ripgrep-backed `search` via extension | 2 |
| v2      | READ, SEARCH, EDIT, TEST                  | plus `write_text`, allow-listed `RUN` via extension | 1, 2 |
| v3      | repeat until tests pass                   | bounded loop, history       | 2    |
| v4      | planner, builder, reviewer                | per-agent permissions       | 3    |
| v5      | budgets, compaction, loop detection       | pure policy functions       | 4    |
| v6      | git diff and status, exact patch          | extension                   | 2, 4 |

Deliberately postponed, in OpenCode's terms: TUI, MCP, multiple providers,
streaming, subagent concurrency, LSP, GitHub integration, session persistence,
embeddings or RAG, automatic context compaction, arbitrary shell access. Each
would obscure the experiment more than it would teach.

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
