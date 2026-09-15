# Saga queue

## Active: agent loop foundation

Install Agentrail process rules; match peer copyright and MIT license; write
the README, architecture, permissions, progression, capability ledger, and
cross-repository handoffs; add a documentation-only `just check` gate. Then
probe the builtins an agent depends on, build the one-tool v0 agent, the
action protocol parser, and the bounded read/write loop, all proven by
mlplunit with a scripted fake model.

Progress: foundation (001), measured builtins (002), the v0 read/think
agent (003), and the action protocol parser (004) are done. Forty-one
mlplunit tests pin the builtin contracts, the v0 flow, and every accepted
and rejected protocol form; `just v0` ran live against qwen2.5-coder:7b;
eight sw-MLPL findings are queued in the ledger. Step 005 remains.

Acceptance: `just check` passes from a clean checkout; the agent reads and
edits a file in `examples/tiny-rust-project` under test without Rust; the
capability ledger classifies every used builtin with evidence.

## Next: mechanisms and permissions

Add the Rust `agent-tools` extension for search, allow-listed process
execution, git diff/status, and exact patching; express allow/ask/deny
permissions as MLPL data; close the loop on "add a unit test and make it
pass".

## Future

Planner, builder, and reviewer agents sharing one model with different
prompts and permission sets. Iteration policy: budgets, compaction, loop
detection, retry, cancellation. Finally the OpenCode comparison write-up and
a `demo-mlpl-libraries` handoff for the domain-neutral agent core.
