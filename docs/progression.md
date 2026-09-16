# Progression

The agent grows in visible steps. Each version is a separate MLPL file so a
reader can diff the loop as it gains capability.

| version | shape                                     | tools                       | saga |
|---------|-------------------------------------------|-----------------------------|------|
| v0      | READ, THINK (done)                        | `read_text`                 | 1    |
| v2      | bounded READ, WRITE loop, allow/ask/deny (done) | `write_atomic`, history, stop reasons | 1 |
| v3      | RUN mlpl, verified DONE, live coding on the MLPL target (done) | `run_script`, injected verify | 2 |
| v1      | SEARCH                                    | ripgrep-backed `search` via extension | 2 |
| v3b     | RUN cargo, repeat until Rust tests pass   | allow-listed `run` via extension | 2 |
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

## First live coding run on the MLPL target

Task: add `u:mul` and its test to `examples/tiny-mlpl-project`, run the
tests, finish when they pass. Model: `qwen2.5-coder:7b`, budget 12, writes
approved, verify mode `tests`. Six attempts on 2026-09-15, each one changing
exactly one thing after reading the transcript:

| attempt | what the model did | what changed after it |
|---------|--------------------|-----------------------|
| 1 | Rewrote `lib.mlpl` without `def`; put a fabricated `OBSERVATION: wrote ...` inside its test WRITE so it was rejected; replied DONE claiming the tests passed with no RUN | Parser drops everything from a fabricated `OBSERVATION:` line on (tested); prompt forbids writing observations and demands verbatim rewrites |
| 2 | Wrote `blank line, fence, body, END, fence`; every WRITE rejected; RUN passed on the unchanged files; DONE claimed `u:mul` was added | Parser tolerates blanks before an opening fence and a fence after END (tested) |
| 3 | One reply contained the WRITE, a fabricated observation, a fabricated RUN, and a fabricated result; then DONE | Verified completion: DONE is accepted only when an injected verify function finds the evidence in history, else the loop observes `NOT VERIFIED` and continues |
| 4 | `READ <path> END` on one line; fabricated `ERROR: write_file: File already exists.` inside a WRITE; every WRITE missing END; RUN passed on unchanged files; DONE accepted because tests did pass | Verifier also requires at least one successful WRITE before the passing RUN |
| 5 | Prompt rewritten as concrete examples: the model copied the example path `src/lib.mlpl` verbatim for every action and exhausted the budget on missing-directory errors | Prompt uses `<path>` placeholders in every example plus one worked WRITE body |
| 6 | READ lib, WRITE lib with `u:add` kept and `u:mul` added, READ tests, WRITE tests with the includes and both tests, RUN, both passed, DONE | Nothing. The saved transcript is `fixtures/transcripts/mlpl-mul-qwen2.5-coder-7b.txt` |

| 7 | Later `just mlpl-demo` run: skipped both READs, wrote `lib.mlpl` with the prompt's example docstring text copied verbatim, wrote a test file in an invented syntax with `include "lib.mlpl"` (wrong path), and after the RUN failed repeated the identical WRITE and RUN until interrupted | Two loop guards: a WRITE to an existing file this run has not READ is refused with `read <path> before writing it`; an identical reply is flagged `REPEATED ACTION` once and stops the run as `stuck` the third time. The prompt's worked WRITE example became placeholders that cannot be copied as content |

| 8 | With the guards and the de-anchored prompt: one wasted SEARCH (placeholder observation), then read, write, read, write with a fabricated observation inside the reply that the parser dropped, RUN with both tests passing, verified DONE; seven steps | Nothing |

Steps in the successful run: six, the minimum. Protocol mistakes in that
run: none. One style slip: the `u:mul` test docstring lacks its trailing
`;`, which MLPL accepts because a newline also separates statements. What the failures taught: a 7B model follows a text protocol
only with placeholder examples, a forbidden-continuation rule, and a loop
that refuses unverified success. Fabricated observations are the dominant
failure and the verifier is what makes the demo honest.

## Model comparison on the same task

`just mlpl-demo`, same prompt, same guards, one run each, 2026-09-15:

| model | size | steps | outcome | wasted or repaired steps | wall |
|-------|------|-------|---------|--------------------------|------|
| qwen2.5-coder:7b | 4.7 GB | 7 | verified done, both tests pass | one SEARCH placeholder | ~90 s |
| `devstral:24b` (Devstral Small 1.x, 23.6B, Q4_K_M), before the header tolerance | 14 GB | 9 | verified done, both tests pass | two parse errors from a copied `ACTION:` header, one RUN before the test was written, then a correct write and rerun | 68 s |
| `devstral:24b` (Devstral Small 1.x), after | 14 GB | 6 | verified done, both tests pass | none | 32 s |
| `devstral-small-2:24b` (Devstral Small 2, 24B mistral3, Q4_K_M) | 15 GB | 6 | verified done, both tests pass | none; chose PATCH for the library edit unprompted | 70 s incl. 21 s model load |

Devstral read before writing without being told twice, reran the tests
after its second write, and made no syntax slips in the MLPL it wrote. Its
only habit was echoing the transcript's `ACTION:` header; once the parser
dropped it, the run was the minimum six steps with no wasted step. It is
the upgrade tier: it needs about 14 GB, so it does not fit the 12 GB floor.
Note the tags: Ollama's `devstral:24b` is the first-generation Devstral
Small (its built-in system prompt names the OpenHands scaffold);
Devstral 2 is `devstral-small-2:24b`. Both did the minimum six steps; the
second generation was the first model to reach for PATCH instead of
rewriting the whole library file.

## The Rust target

`just rust-demo`: read `src/lib.rs`, PATCH in a `#[cfg(test)]` module, RUN
`cargo test --manifest-path ...` through the extension, DONE when it passes.
Same prompt, same guards, 2026-09-15:

| model | steps | outcome | what happened | wall |
|-------|-------|---------|---------------|------|
| qwen2.5-coder:7b | 12 (budget) | not done | its test module omitted `use super::add`, so `cargo test` failed with E0425; it then re-sent PATCH blocks whose OLD text no longer matched the changed file, re-read, re-ran, and repeated until the budget ended | 26 s |
| devstral-small-2:24b, first run | 7 | verified done, 1 test passed | the demo's restore had reverted an uncommitted fixture fix, so the first `cargo test` failed with cargo's workspace error; the model read `Cargo.toml`, added the empty `[workspace]` table cargo suggested, reran, and passed. A correct self-repair of an environment fault that was ours | 70 s incl. 23 s load |
| devstral-small-2:24b, fixture fixed | 4 | verified done, 1 test passed | read, PATCH with `use super::*`, run, done; the minimum | 27 s |

PATCH did what it was added for: the 7B and the 24B both edited a region
instead of rewriting the file, and the rest of `lib.rs` stayed byte-identical.
The 7B's failure is a Rust knowledge gap (module scoping), not a protocol
slip; the loop's guards stopped the thrash at the budget with no false DONE.

## Demo recording

`demos/loop.tape` records `just replay`: `agents/replay_loop.mlpl` extracts
the model replies from the saved live transcript and feeds them to the real
loop, so the parser, the writes, and the `run_script` test run are live
while the model is deterministic. The tape needs no Ollama and finishes in
seconds. `demos/loop-live.tape` records `just mlpl-demo` with a real model.

Why a replay, and why fixed sleeps: the first live recording had VHS wait
fifteen minutes for a marker that never matched. The cause was using
`Wait+Screen /regex/` at all. In VHS 0.11 it matched the typed command line
(a tape waiting for `MARKER` returned before any output appeared), and when
the regex could only match program output it saw a stale screen and timed
out, while the GIF frames captured the run correctly. The sibling
`../sw-os-ml` tapes never use `Wait`; they `Sleep` a fixed time after
`Enter`, and that records the whole run. The replay is deterministic and
takes a few seconds, so a fixed sleep is exact; the live tape sleeps eight
minutes, which covers a warm model at a budget of twelve steps but not a
cold one, so `scripts/check-ollama` loads the model before the tape starts
and reports the load time. The gate's `scripts/check-replay` proves the
same run the GIF shows: six steps, both tests passed, verified done, and
the example restored. WebP was measured at 135% of the optimised GIF, as the sibling
also found, so only the GIF is committed.

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
