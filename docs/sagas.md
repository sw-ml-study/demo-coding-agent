# Saga queue

## Completed: agent loop foundation

Install Agentrail process rules; match peer copyright and MIT license; write
the README, architecture, permissions, progression, capability ledger, and
cross-repository handoffs; add a documentation-only `just check` gate. Then
probe the builtins an agent depends on, build the one-tool v0 agent, the
action protocol parser, and the bounded read/write loop, all proven by
mlplunit with a scripted fake model.

Accepted: `just check` passes from a clean checkout with 51 mlplunit tests;
`agents/loop.mlpl` reads the fixture crate and writes a file under test with
no Rust, driven by a scripted transcript model; `done`, `denied`, and
`budget` stop reasons, the ask decision, parse-error recovery, and read
errors are all proven offline; `just v0` and `just loop` ran live against
qwen2.5-coder:7b; the capability ledger classifies every used builtin with
evidence and queues eight sw-MLPL findings.

## Active: the agent codes in MLPL, then mechanisms for Rust

Progress: step 001 added `examples/tiny-mlpl-project` (runs under
`run_script` with captured PASS events via the vendored mlplunit library)
and hardened the protocol against fenced WRITE bodies with tests for each
habit and for the strict rejections. Step 002 made `RUN mlpl <path>` real in
pure MLPL: an allow-list function, `run_script` with captured events, and a
rendered observation with one line per finished test. Step 003 closed the
loop live: after six attempts (fabricated observations, fence habits, false
DONE) the loop gained verified completion and `qwen2.5-coder:7b` added
`u:mul` with a passing test in six steps; the transcript is a fixture. Step
004 recorded that run with VHS into `assets/demo/` and linked the GIF from
the README: `demos/loop.tape` replays the saved transcript through the real
loop (`just replay`, no model server), `demos/loop-live.tape` records a
fresh live run. Fixed sleeps replaced VHS `Wait`, which matched the typed
command or saw a stale screen; `just replay-check` in the gate proves the
recorded run without VHS; live recipes warm the model first. Step 005 made
`just loop` interactive: at a terminal the agent asks before each write
through a FIFO forwarder because sw-MLPL's stdin builtins refuse a TTY
(ledger F9); the demo script now restores the example on Ctrl-C and keeps
live transcripts out of the committed fixture unless asked.

First target an MLPL example project: `RUN` maps to `run_script` in pure
MLPL, the protocol is hardened from live transcripts (fences, END-less
bodies), and the agent adds a function plus its test and iterates until the
tests pass, recorded as a VHS demo. Then add the Rust `agent-tools`
extension for ripgrep search, allow-listed `cargo`/`git`, and exact patching
to reach the same result on the Rust example.

## Future

Planner, builder, and reviewer agents sharing one model with different
prompts and permission sets. Iteration policy: budgets, compaction, loop
detection, retry, cancellation. Finally the OpenCode comparison write-up and
a `demo-mlpl-libraries` handoff for the domain-neutral agent core.
