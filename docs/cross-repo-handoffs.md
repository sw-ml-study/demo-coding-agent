# Cross-repository handoffs

Sibling repositories are read-only from this project. Work that belongs
elsewhere is written here as an actionable request with acceptance criteria.

## `../demo-extensions` (read-only reference)

This repository reuses, but does not modify, the extension authoring path:
`mlpl-extension-abi`, `mlpl-extension-sdk`, the `extension.toml` package
manifest, the private `_namespace` plus public `module.mlpl` facade pattern,
and stock-CLI `load_extension`.

Open decision for Saga 2: the SDK crates are unpublished (`version = 0.0.0`),
so `extensions/agent-tools/Cargo.toml` must either use a path dependency on
the adjacent checkout, pinned by documented revision, or a git dependency on
the demo-extensions repository at a pinned commit. The git dependency is the
cleaner external-crate story and is the default unless build time or offline
policy argues otherwise. Record the chosen pin in `docs/architecture.md`.

No request is open.

## `../demo-mlpl-libraries` (future consumer)

After three agents (single loop, planner/builder/reviewer, budgeted loop) use
the same action parser, policy evaluator, and bounded history helpers, write a
producer contract and extraction handoff there, following the revision-pinned
vendoring convention that repository already proves. Not before Saga 5.

## `../sw-mlpl` (last resort)

Request a language change only when `docs/sw-mlpl-capabilities.md` records a
blocked entry with an executable probe. Candidate awkwardness worth reporting
without requesting anything: tagged sum values and a `match` form would make
action dispatch read better than nested `if str_eq(action.tool, ...)`.

No request is open.
