# sw-MLPL capability ledger

Classification of what the coding agent needs from the language, measured
against sw-MLPL 0.22.0 (`../sw-mlpl` release build, commit e6070964). Entries
move from "documented" to "measured" once a probe or mlplunit test in this
repository exercises them; Saga 1 step 2 owns that measurement.

Categories: **measured** (pinned by a test or probe in this repository),
**supported** (works as documented, not yet pinned), **awkward** (expressible but
clumsy, worth noting upstream), **extension** (not in the language, belongs
in this repository's Rust `agent-tools` extension), **blocked** (neither a
builtin nor an extension can express it; needs a probe before any request).

| need                                  | primitive                                  | status     | evidence |
|---------------------------------------|--------------------------------------------|------------|----------|
| ask a local model                     | `llm_call(url, prompt, model[, system])`   | measured   | `just llm-probe`: qwen2.5-coder:7b answered a one-word system-prompted ping with `PONG`; reply is a plain string |
| read a project file                   | `read_text(path)` -> `ok(text)`/`err`      | measured   | `tests/test_fs_builtins.mlpl`: exact text, missing file is `err("read_text: No such file...")` |
| list project files                    | `fs_walk(root, {recursive, kind, pattern})`| measured   | root-relative lexical paths that feed `read_text` directly |
| write a whole file                    | `write_text`, `write_atomic`               | measured   | create, replace, read back, `remove_path`; missing parent directory is an `err` (see findings) |
| file metadata                         | `file_metadata(path)`                      | measured   | `{kind, size, modified_unix_ms}` |
| sandbox confinement                   | `--source-dir` root                        | measured   | `../` and an outside-target symlink give `err("...: outside the sandbox")` for read and write; an inside-target symlink is readable (see findings) |
| run an MLPL script as a child         | `run_script(path, opts)`                   | measured   | `tests/test_run_script.mlpl`, `tests/test_tools.mlpl`: `{status, value, value_raw, error, events, events_kind}`; a failing mlplunit-style file is `status: "err"`; `capture: 1` events parse with `parse_json` into `{kind, name, status, diagnostic}`; MLPL only, not a process runner |
| prefix-parse an action line           | `str_find`, `str_slice`, `str_split`, `str_eq`, `str_len`, `list_len`, `list_get` | measured | `tests/test_string_protocol.mlpl`; no `str_starts_with` or `str_trim` exist, prefix is `str_find(s, verb) == 0` |
| join observations                     | `str_concat(a, b)`, `str_join(parts, sep)` | measured   | `+` on two strings is an error (see findings); `str_join([], sep)` is `""` |
| inject a fake model                   | `:u:` references, `call` partials          | measured   | `tests/test_v0_read_think.mlpl`: a three-argument partial of `u:ask_live` and a one-argument partial of `u:ask_scripted` both invoke with `call(ask, prompt)` |
| search text across files              | none                                       | extension  | ripgrep crates (`grep-searcher`, `grep-regex`, `ignore`) in `agent-tools`; `rg` subprocess fallback. Pure MLPL `fs_walk`+`read_text`+`str_find` is possible but ignores `.gitignore` and binaries |
| tagged action values                  | records + `Result`                         | measured, awkward | `agents/protocol.mlpl`: `ok({tool: ..})`/`err(reason)` works and tests read fields directly; dispatch on `tool` is a five-deep nested `if`/`else` chain (see F7) |
| run `cargo test` / `git`              | none                                       | extension  | allow-listed argv runner in `agent-tools` |
| git diff / status                     | none                                       | extension  | `agent-tools` |
| exact old/new patch                   | `read_text` + `str_find` + `write_atomic`  | supported  | pure MLPL; no extension planned |
| streaming or native tool calling      | none                                       | non-goal   | text protocol by design |
| OpenAI-compatible chat endpoint       | none; `llm_call` speaks Ollama `/api/generate` only (`contracts/eval-contract/llm-call.md`) | extension or upstream | planned so any hosted model can drive the loop; an HTTP POST from the `agent-tools` extension is the first candidate, an `llm_call` wire-format option the upstream alternative |

## Upstream status (2026-09-15)

The sw-mlpl maintainer agent has F1 through F5 captured and queued, not yet
implemented, and intends to lead a demo-coding-agent fix saga with F2 (the
misleading unknown-function diagnostic) because it makes every other
missing-builtin guess self-explanatory. Until then this repository keeps its
workarounds: `str_concat`/`str_join` for F1, `list_len` for F5, flat file
layouts for F4, nothing needed for F3. Fixes are being developed in
parallel; re-measure the affected tests when the adjacent binary changes.

Shipped upstream this session and available if needed, none required by the
CLI plus `run_script` path this agent uses today: `include` over the wire
(an includes map on the eval request), a `--fs-root` server sandbox for byte
file builtins, `args` on the eval request, `reset_optimizer()`, and clean
errors instead of panics on shape, label, matmul, and out-of-range `take`
mistakes in generated programs.

## Findings queue

Bugs, blocking gaps, and justified improvement suggestions for sw-MLPL are
recorded here as they are met, each with: a minimal probe, expected versus
observed behavior, the affected agent, and an honest "unavailable" status
until upstream ships a change. Do not work around a bug silently.

### F1. `+` on two strings fails with an array diagnostic (bug, diagnostic)

Probe: `x = "a" + "b"` gives `error: expected an array value, got a string`.
Expected: either string concatenation or a message that names strings and
points at `str_concat`. Affects every agent: prompt building must use
`str_concat`/`str_join`, and the research transcript's `+` pseudo-code does
not run. Suggested improvement: make `+` concatenate strings, or at least
emit `strings do not support +; use str_concat`.

### F2. Calling an undefined function reports the same array diagnostic (bug)

Probe: `nope_fn("a")` gives `error: expected an array value, got a string`
instead of an unknown-function error. Cost measured directly: it hid the fact
that `str_starts_with` and `str_trim` do not exist. Expected: `unknown
function: nope_fn`. Affects every step that guesses a builtin name.

### F3. Inside-target symlinks are readable despite "never followed" (docs)

Probe: `tests/fixtures/link-inside -> ../../LICENSE` reads successfully;
`link-outside -> /etc/hosts` is `err(outside the sandbox)`. The behavior is
the useful one; the lang-reference wording "symlinks are never followed"
should say "symlinks that resolve outside the sandbox are refused".

### F4. `write_text` does not create parent directories (awkward)

Probe: `write_text("tests/scratch/no-such-dir/x.txt", "x")` is
`err(No such file or directory)`. An agent creating a new module in a new
directory needs a `make_dir` builtin or a documented `write_text` option.
Suggested improvement: add `make_dir(path)` (sandboxed, `ok(1)`/`err`).

### F5. `len` rejects string lists (awkward)

Probe: `len(str_split("a b", " "))` is an error; `list_len` is required.
Expected by an array-language reader: `len` is total over lists. Minor;
recorded so agents use `list_len`.

### F6. `include` resolves differently under mlpl-repl and mlplunit (tooling)

Probe: `mlpl-repl -f agents/run_v0.mlpl` resolves `include "v0_read_think.mlpl"`
relative to the including file and refuses `../` that leaves `--source-dir`.
mlplunit copies each test to a temporary file, so a file-relative
`include "../agents/x.mlpl"` fails with "no such source file" and the test
must write `include "agents/x.mlpl"` relative to `source_root`. Both are
defensible; the asymmetry means an agent file cannot be included the same
way from a test and from a runner. Suggested improvement: have mlplunit run
tests in place, or document source-root-relative includes as the contract.
Consequence measured in Saga 2: `examples/tiny-mlpl-project/tests/test_add.mlpl`
includes the vendored library file-relatively so `run_script` (the agent's
`RUN`) can execute it; the same file cannot be run by mlplunit, which copies
it to a temp directory and would also prepend its own copy of the library.

### F7. Dispatch on a record tag needs nested if/else chains (awkward, improvement)

Probe: `u:parse_action` in `agents/protocol.mlpl` chooses among five verbs
with five nested `if ... { } else { if ... }` levels because there is no
`match`, no `else if`, and no early return. The same shape will appear in
the loop's dispatch on `action.tool`. The record-plus-`Result` encoding
itself is fine: `ok({tool: "write", path, body})` is readable and tests
read fields directly. Justified upstream suggestion, in order of value:
`else if` chaining (small parser change, removes the nesting), then a
`match value { "read" => ..., _ => ... }` form over strings and record tags.
A full tagged-sum type is not needed for this demo.

### F8. No boolean `&&`/`||` operators and no `str_trim` or list filter (awkward)

Probe: `a && b` is `UnexpectedCharacter '&'`. The parser uses nested `if`
and `break` inside `while` instead, and hand-writes `u:rtrim`, `u:ltrim`,
and `u:split_words` (which collapses repeated spaces by rejoining through a
string because string lists have no filter or append). Suggested
improvements: `str_trim`, `str_starts_with`, and either `list_filter` /
`list_push` or a whole-list `each` over string lists. `&&`/`||` on 0/1
scalars would read better than nested conditionals but are not blocking.
