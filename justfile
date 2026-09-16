set shell := ["sh", "-cu"]

# Show available repository tasks.
default:
    @just --list

# Print the selected sw-MLPL executable without installing anything.
mlpl-path:
    ./scripts/select-mlpl

# Print the selected mlplunit runner without installing anything.
mlplunit-path:
    ./scripts/select-mlplunit

# Check required files, peer license parity, and briefing markers.
structure:
    ./scripts/check-structure

# Check that local Markdown links resolve.
doc-links:
    ./scripts/check-doc-links

# Check that the generated Agentrail briefing block is current.
instructions:
    ./scripts/check-instructions

# Check canonical formatting and module comments for tracked MLPL source.
mlpl-style:
    ./scripts/check-mlpl-style

# Run native mlplunit probes; arguments select paths, tags, or filters.
tests *args:
    ./scripts/run-tests {{args}}

# Opt-in live llm_call probe against Ollama (OLLAMA_HOST, OLLAMA_MODEL). Not in check.
llm-probe:
    ./scripts/run-llm-probe

# Opt-in live v0 agent run: read one file, think once (V0_FILE, V0_TASK override).
v0:
    ./scripts/run-v0

# Opt-in live bounded-loop run (LOOP_TASK, LOOP_BUDGET, LOOP_APPROVE=1 override).
loop:
    ./scripts/run-loop

# Opt-in live demo: the agent adds u:mul plus a test to the MLPL example and makes the tests pass.
mlpl-demo:
    ./scripts/run-mlpl-demo

# Replay the saved live transcript through the real loop with typewriter pacing; no model server needed.
replay:
    ./scripts/run-replay

# Opt-in: record the live MLPL coding demo with VHS into assets/demo (gif, mp4, webm, webp).
demo-tape:
    ./scripts/record-demo

# Prove the committed transcript replays to a verified done and restores the example.
replay-check:
    ./scripts/check-replay

# Build and prove the Rust agent-tools extension (cargo tests, clippy, sw-checklist, dynamic load).
agent-tools:
    ./scripts/check-agent-tools

# Run the complete precommit gate.
check:
    ./scripts/check
