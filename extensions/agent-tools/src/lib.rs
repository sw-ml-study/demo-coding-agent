//! Narrow OS mechanisms for the mlplcode coding agent. MLPL owns policy;
//! this crate only searches files under a root and runs allow-listed
//! commands inside it.

mod allow;
mod run;
mod search;

pub use allow::allowed;
pub use run::run_value;
pub use search::search_value;

const METADATA: &str = r#"
[[functions]]
name = "search"
documentation = "Search files under root for a regex, honouring .gitignore; bounded path:line:text output."
returns = "record"
[[functions.arguments]]
name = "root"
type = "string"
[[functions.arguments]]
name = "pattern"
type = "string"

[[functions]]
name = "run"
documentation = "Run one allow-listed command (cargo test/check/clippy/fmt, git diff/status) inside root with a timeout."
returns = "record"
[[functions.arguments]]
name = "root"
type = "string"
[[functions.arguments]]
name = "command"
type = "string"
"#;

mlpl_extension_sdk::export_extension! {
    module: generated_export,
    entry: sw_mlpl_extension_v1,
    name: "_agent_tools",
    version: "0.1.0",
    metadata: crate::METADATA,
    functions: [
        (search_trampoline, "search", 2, crate::search_value),
        (run_trampoline, "run", 2, crate::run_value),
    ]
}

pub use sw_mlpl_extension_v1 as static_entry;
