# sw-MLPL capability ledger

Classification of what the coding agent needs from the language, measured
against sw-MLPL 0.22.0 (`../sw-mlpl` release build, commit e6070964). Entries
move from "documented" to "measured" once a probe or mlplunit test in this
repository exercises them; Saga 1 step 2 owns that measurement.

Categories: **supported** (works as documented), **awkward** (expressible but
clumsy, worth noting upstream), **extension** (not in the language, belongs
in this repository's Rust `agent-tools` extension), **blocked** (neither a
builtin nor an extension can express it; needs a probe before any request).

| need                                  | primitive                                  | status     | evidence |
|---------------------------------------|--------------------------------------------|------------|----------|
| ask a local model                     | `llm_call(url, prompt, model[, system])`   | supported  | documented in `docs/using-llm-tool.md`; live probe pending |
| read a project file                   | `read_text(path)` -> `ok(text)`/`err`      | supported  | documented in `lang-reference.md`; probe pending |
| list project files                    | `fs_walk(root, {recursive, kind, pattern})`| supported  | documented; probe pending |
| write a whole file                    | `write_text`, `write_atomic`               | supported  | documented; probe pending |
| file metadata                         | `file_metadata(path)`                      | supported  | documented |
| sandbox confinement                   | `--source-dir` root, symlinks not followed | supported  | documented; escape probe pending |
| run an MLPL script as a child         | `run_script(path, opts)`                   | supported  | MLPL only; not a general process runner |
| prefix-parse an action line           | `str_find`, `str_slice`, `str_split`, `str_eq`, `str_len` | supported | documented |
| join observations                     | `str_join(parts, sep)`                     | supported  | documented |
| inject a fake model                   | function references, `call`                | supported  | used by mlplunit itself |
| search text across files              | none                                       | extension  | ripgrep crates (`grep-searcher`, `grep-regex`, `ignore`) in `agent-tools`; `rg` subprocess fallback. Pure MLPL `fs_walk`+`read_text`+`str_find` is possible but ignores `.gitignore` and binaries |
| tagged action values                  | records + `Result`                         | awkward    | `{tool: ..}` records work; no `match` on a tag |
| run `cargo test` / `git`              | none                                       | extension  | allow-listed argv runner in `agent-tools` |
| git diff / status                     | none                                       | extension  | `agent-tools` |
| exact old/new patch                   | `read_text` + `str_find` + `write_atomic`  | supported  | pure MLPL; no extension planned |
| streaming or native tool calling      | none                                       | non-goal   | text protocol by design |
| OpenAI-compatible chat endpoint       | none; `llm_call` speaks Ollama `/api/generate` only (`contracts/eval-contract/llm-call.md`) | extension or upstream | planned so any hosted model can drive the loop; an HTTP POST from the `agent-tools` extension is the first candidate, an `llm_call` wire-format option the upstream alternative |

## Findings queue

Bugs, blocking gaps, and justified improvement suggestions for sw-MLPL are
recorded here as they are met, each with: a minimal probe, expected versus
observed behavior, the affected agent, and an honest "unavailable" status
until upstream ships a change. Do not work around a bug silently.

None yet.
