//! Behaviour and ABI contract of the agent-tools extension.
#![allow(unsafe_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use mlpl_extension_loader::{ProviderKind, Registry};
use mlpl_extension_sdk::Value;

fn library() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("target/debug/deps");
    path.push(format!(
        "{}mlpl_extension_agent_tools{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    path
}

fn project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/lib.mlpl"), "def u:add(a, b) {\n  a + b\n}\n").unwrap();
    fs::write(dir.path().join("notes.txt"), "u:add is documented here\n").unwrap();
    fs::write(dir.path().join("ignored.log"), "u:add in an ignored file\n").unwrap();
    fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    dir
}

fn strings(values: &[&str]) -> Vec<Value> {
    values.iter().map(|value| Value::String((*value).to_owned())).collect()
}

fn field<'a>(record: &'a Value, name: &str) -> &'a Value {
    let Value::Record(map) = record else { panic!("not a record") };
    &map[name]
}

#[test]
fn search_reports_relative_path_line_and_text_and_honours_gitignore() {
    let dir = project();
    let result = mlpl_extension_agent_tools::search_value(&strings(&[dir.path().to_str().unwrap(), "u:add"])).unwrap();
    let Value::String(matches) = field(&result, "matches") else { panic!() };
    assert!(matches.contains("src/lib.mlpl:1:def u:add(a, b) {"), "{matches}");
    assert!(matches.contains("notes.txt:1:u:add is documented here"), "{matches}");
    assert!(!matches.contains("ignored.log"), "gitignored file searched: {matches}");
    assert_eq!(field(&result, "count"), &Value::I64(2));
    assert_eq!(field(&result, "truncated"), &Value::Bool(false));
}

#[test]
fn search_rejects_a_missing_root_and_a_bad_pattern() {
    let dir = project();
    assert!(mlpl_extension_agent_tools::search_value(&strings(&["/no/such/dir", "x"])).is_err());
    assert!(mlpl_extension_agent_tools::search_value(&strings(&[dir.path().to_str().unwrap(), "("])).is_err());
    assert!(mlpl_extension_agent_tools::search_value(&[Value::I64(1), Value::I64(2)]).is_err());
}

#[test]
fn run_refuses_anything_outside_the_allow_list_without_starting_it() {
    let dir = project();
    let marker = dir.path().join("should-not-exist");
    let command = format!("touch {}", marker.display());
    let error = mlpl_extension_agent_tools::run_value(&strings(&[dir.path().to_str().unwrap(), &command])).unwrap_err();
    assert!(error.message().contains("command not allowed"), "{}", error.message());
    assert!(!marker.exists());
    assert!(mlpl_extension_agent_tools::run_value(&strings(&[dir.path().to_str().unwrap(), "sh -c ls"])).is_err());
}

#[test]
fn run_executes_git_status_inside_the_root_and_captures_output() {
    let dir = project();
    assert!(Command::new("git").args(["init", "-q"]).current_dir(dir.path()).status().unwrap().success());
    let result = mlpl_extension_agent_tools::run_value(&strings(&[dir.path().to_str().unwrap(), "git status --short"])).unwrap();
    assert_eq!(field(&result, "status"), &Value::I64(0));
    assert_eq!(field(&result, "timed_out"), &Value::Bool(false));
    let Value::String(stdout) = field(&result, "stdout") else { panic!() };
    assert!(stdout.contains("?? notes.txt"), "{stdout}");
}

#[test]
fn dynamic_and_static_providers_publish_the_same_two_functions() {
    let dynamic = Registry::load(library()).unwrap();
    // SAFETY: the generated entry returns immutable process-lifetime descriptor storage.
    let statik = unsafe { Registry::load_static(mlpl_extension_agent_tools::static_entry) }.unwrap();
    assert_eq!(dynamic.provider_kind(), ProviderKind::Dynamic);
    assert_eq!(statik.provider_kind(), ProviderKind::Static);
    for registry in [dynamic, statik] {
        assert_eq!(registry.extension_name(), "_agent_tools");
        assert_eq!(registry.function_names(), ["_agent_tools.run", "_agent_tools.search"]);
    }
}
