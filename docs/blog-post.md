# A coding agent small enough to read

*Draft. Numbers are from the repository on 2026-09-16; the author will edit
before publishing.*

![mlplcode adding a function and a passing test to a small MLPL project](../assets/demo/loop.gif)

Coding agents are usually described from the outside: a product with a
terminal UI, a permission system, a dozen tools, and a model behind it all.
I wanted to see them from the inside, and I wanted the inside to be small.
So I built one in sw-MLPL, an array language I use for machine-learning
study, and gave myself one rule: MLPL owns every decision, Rust owns only
the mechanisms MLPL cannot express, and a local model owns inference.

The question was not "can MLPL call a model" but "can an array language
express the control plane of an autonomous coding agent." The answer is
yes, in about 1,200 lines and 75 functions, and the interesting part is
what those lines had to contain.

## The loop

The whole agent is one record flowing through pure functions:

```text
task -> build_context -> ask (injected) -> parse_action -> guards
     -> authorize (allow | ask | deny) -> execute -> next state
```

The model speaks a six-verb text protocol instead of native tool calling:
`READ`, `SEARCH`, `WRITE ... END`, `PATCH ... END`, `RUN`, `DONE`. A reply
is one string; the parser turns it into a record such as
`{tool: "read", path: "src/lib.mlpl"}` or an error naming the reason.
Nothing downstream ever sees the raw text again.

The model is a function passed in, not a client wired in. A live run binds
the Ollama host, model name, and system prompt into a one-argument
function; a test binds a fixed reply, or an echo function that returns the
prompt so the test can assert on exactly what the model would have seen. Of
the 84 tests, none needs a model server.

Mechanisms came first from the language itself. sw-MLPL already ships
sandboxed `read_text`, `write_atomic`, and `run_script`, confined to a root
directory with symlinks never followed, and `run_script` runs an MLPL test
file and returns its outcome as data. That meant the agent could read,
edit, run tests, and repeat on an MLPL project with no Rust at all. The Rust
extension arrived later and stayed narrow: ripgrep-backed search that
honours `.gitignore`, and a process runner whose argv must begin with
`cargo test`, `cargo check`, `cargo clippy`, `cargo fmt`, `git diff`, or
`git status`. There is no shell anywhere. That runner is about 250 lines.

## What the model actually did

The first live task was: add `u:mul` and a test to a one-function MLPL
project, run the tests, finish when they pass. With `qwen2.5-coder:7b`, a
model that fits a 12 GB card, it took six attempts to get a clean run, and
each attempt changed exactly one thing.

The model wrote a fabricated `OBSERVATION:` block inside its own reply,
then declared `DONE` claiming the tests passed when nothing had run. Three
of the six attempts failed that way. The fix was not a prompt tweak. The
loop now takes an injected verify function, and `DONE` is accepted only
when the transcript contains a successful edit followed by a test run that
passed with no failure named. Otherwise the model is told `NOT VERIFIED`
and keeps working. Without this the demo would lie.

It wrapped file bodies in markdown fences and dropped the `END` line. It
copied a concrete example path from the prompt verbatim into every action.
It sent the same failing write three times in a row. Each of those got a
tolerance or a guard with a test written first: fence lines are stripped, a
write to a file the run has not read is refused with "read it first," and
an identical reply is flagged once and stops the run as `stuck` the third
time. The system prompt lost its worked example and gained placeholders.

With those in place the 7B model did the task in six or seven steps. Then
I tried larger models. Devstral Small, the 24B first generation, did it in
the minimum six steps once the parser learned to drop the `ACTION:` header
it echoed. Devstral Small 2 did the same, and was the first model to reach
for `PATCH` rather than rewriting the file. On a Rust crate, asked to add a
unit test and make `cargo test` pass through the extension, Devstral Small
2 took four steps; the 7B model omitted `use super::add`, hit the compile
error, and thrashed on stale patches until the budget ended, with no false
`DONE`.

One run is worth describing. The demo script's cleanup had reverted a
fixture fix of mine, so the model's first `cargo test` failed with cargo's
own message about a stray workspace member. It read `Cargo.toml`, added
the empty `[workspace]` table cargo suggested, reran, and passed. A correct
repair of a fault that was mine.

## A kept example

Every demo restores the files it touches, so I also made one that keeps
them. Asked to write a hello-world module and its test from scratch,
Devstral Small 2 first invented `//` comments and left `def` off every
function, because it had never seen MLPL. Told to read the example
project's two files first, it wrote this and the test passed, six steps:

```text
# A simple greeting module.

def u:hello(name) {
  "Return a greeting string.";
  str_concat("hello, ", name)
}
```

That is the honest shape of a small model on an unfamiliar language: it
imitates what it reads. Give it something to read.

## What this cost the language

Building an agent stress-tests a language differently from building a
matrix routine. Nine findings went to the sw-MLPL maintainer, with probes.
Five shipped the same day: `+` now concatenates strings, `len` works on
string lists, an undefined function says so instead of producing an array
error, `make_dir` exists, and a doc line about symlinks is right. The one
that hurt most was the undefined-function diagnostic, because it hid the
fact that `str_trim` and `str_starts_with` do not exist; the parser has
its own trim. Still open: `include` resolves differently under the test
runner and the CLI, a six-way choice needs six nested `if`s because there
is no `else if` or `match`, and every stdin builtin refuses a terminal, so
`just loop` asks for write approval through a FIFO that the wrapper fills
from `/dev/tty`.

## What is deliberately not here

No TUI, no MCP, no multiple providers, no streaming, no session
persistence, no embeddings, no LSP, no GitHub integration, no automatic
context compaction. The plan's front end is an org-mode document rather
than a terminal UI: sw-MLPL ships an org-babel backend, and the whole agent
is already written up as a literate document whose source blocks tangle
back to the committed files, checked in the gate so the prose cannot drift.

The README GIF is a replay of a real transcript through the real loop, not
a recording of the model, so it plays in seconds on any machine and shows
the same six steps every time. That, too, was a lesson: the first
recording attempts waited on a terminal regex that never matched, and the
sibling repository's tapes had used fixed sleeps all along.

## Where it stands

The agent codes. Two example projects, one MLPL and one Rust, get a
function and a passing test added by a local model through a loop you can
read in an afternoon. Everything the model does passes through one
authorization function and one verifier, and every guard exists because a
transcript showed it was needed.

Next is splitting the loop into a planner, a builder, and a reviewer that
share one model and differ only by prompt and permissions. I expect that to
be a hundred lines. The loop is the agent; the rest is policy, and policy
is data.

*Source, transcripts, tests, and the literate document:*
*https://github.com/sw-ml-study/demo-coding-agent*
