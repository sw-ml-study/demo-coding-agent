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

# Run the complete precommit gate.
check:
    ./scripts/check
