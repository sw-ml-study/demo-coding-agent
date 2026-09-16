# Cross-repository handoffs

Sibling repositories are read-only from this project. Work that belongs
elsewhere is written here as an actionable request with acceptance criteria.

## `../demo-extensions` (read-only reference)

This repository reuses, but does not modify, the extension authoring path:
`mlpl-extension-abi`, `mlpl-extension-sdk`, the `extension.toml` package
manifest, the private `_namespace` plus public `module.mlpl` facade pattern,
and stock-CLI `load_extension`.

Decision taken in Saga 2: `extensions/agent-tools/Cargo.toml` depends on
`mlpl-extension-sdk` (and, for tests, `mlpl-extension-loader`) as git
dependencies on `https://github.com/sw-ml-study/demo-extensions` at
revision `2b2ae48e483284ddc83956f464a7481d170367b8`. Move the pin
deliberately: bump both `rev` values together and re-run
`just agent-tools`. The adjacent checkout is never referenced by path.

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
